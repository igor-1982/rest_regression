# MPI 测试范围声明（MPI Test Scope）

> **本文件是 rest_regression 批处理验证中 MPI 模式的测试范围声明。**
> 任何 MPI 相关功能测试与验证命令（包括 `validate.sh` 与手工执行的 `cargo run`）在运行前**必须先阅读本文件**。
> 一旦 MPI 功能测试超出本文件声明的范围，执行方必须给出 warning，不得静默运行。

## 当前 MPI 测试范围（Temporary Scope）

MPI 模式（`--n_mpi >= 2`，即 `mpirun -n N rest`）的回归测试**仅限**以下目录：

| 目录 | 内容 |
|---|---|
| `bench_pool/scf/` | SCF 相关任务（单点能、偶极、BSSE、开壳层、RSH、meta-GGA 等） |
| `bench_pool/gradient/` | 解析梯度任务（RHF/UHF/UKS/RMP2/RSH 梯度、外场梯度、数值梯度优化等） |
| `bench_pool/dispersion/` | 色散校正任务（D3BJ 能量与解析梯度，2026-09 验证通过） |
| `bench_pool/solvent/` | 溶剂模型任务（COSMO/CPCM 梯度，2026-09 验证通过） |

## 范围外目录（Out of Scope，暂不纳入 MPI 批处理验证）

以下目录中的任务**暂不**纳入 MPI 模式的批处理验证：

- `bench_pool/dh/`：双杂化/后 SCF 任务（2026-09 从目录级范围**移出**，改为按用例纳入，理由见下）
- `bench_pool/gw_bse/` — GW/BSE 任务
- `bench_pool/hessian/` — Hessian 与频率任务
- `bench_pool/pbc/` — 周期边界任务
- `bench_pool/tddft/` — TDDFT 任务
- 其他未在“当前 MPI 测试范围”表格中列出的目录

这些任务在 MPI 模式下可能尚未验证或不支持 MPI（例如 `hessian/CO_HF_OPT_TRIC_ANALYTIC_HESS` 在 `-n 2` 下会触发
`ri_jk/pure_decompose.rs` 的 naux 断言失败）。它们仍应通过 **rayon-only（`-n 1`）** 模式验证。

### 为什么把 dh 目录降级为按用例范围（2026-09）

1. `bench_pool/dh/` 里除后 SCF 能量用例外，还加入了新实现的 rayon 版 dh multipole 用例
   （`NH3_RMP2_NUMDIPOLE`、`NH3_RMP2_EXT_FIELD_DIPOLE`）。整目录纳入 MPI 范围后，任何
   新增的 dh 用例会**自动**进入 MPI 批处理，等于绕过了"逐个验证后再纳入"的门槛。
2. 目录名 `dh` 同时被用作过滤词与用例 tag。`validate.sh` 的过滤匹配同时看路径分量与
   `tests.toml` 的 tags，于是 `bench_pool/analdrv/H2O_XYG3_multipole`（tags 含 `dh`，
   范围外且未验证 MPI）被一起选中，并在 `analdrv/response/rresp_interface.rs` 的 rimatr
   `expect` 上 panic，使 `--mpi` 在第一个任务就失败。
3. 因此把 `dh` 从目录级范围移出，改为对**已逐个验证通过**的用例打 `tags=["mpi"]`。
   当前 dh 目录下 22 个用例全部在 `mpirun -n 2` 下验证通过，全部已打 tag，MPI 覆盖不减少。
   新增 dh 用例默认不进 MPI 模式，验证通过并打 tag 后才进入。

## 按用例扩展范围（Per-Test Scope，tags=["mpi"]）

除目录级范围外，允许**目录未整体纳入、但个别用例已验证支持 MPI** 的情形（如未来 gw_bse/tddft 等目录下的单例）。
这类用例通过在 `tests.toml` 中给该测试条目加 `tags = ["mpi"]` 声明：

```toml
[某个范围外目录用例]
tags = ["mpi"]
```

- **语义**：`tags=["mpi"]` = 该用例支持 MPI **且已在目标 MPI 配置下全量验证通过**（数值与串行一致）。
- **生效方式**：`validate.sh --mpi`（自动限定模式）会把此类用例与声明目录一并纳入；定向 `--mpi -f <用例名>` 不再告警。
- **目录级优先原则**：当某目录内按用例纳入的用例达到全部（或明确策略约定）时，通常应将目录整体提升为目录级范围（移入上表、删除冗余 tag），避免长期维护逐例 tag。
  例外是 dh 目录（2026-09 降级）：目录内会持续加入只做 rayon 的新用例，整目录纳入等于让新用例未经 MPI 验证就自动进批处理，故保持按用例纳入。
- 范围外目录（gw_bse/hessian/pbc/tddft/dh 等）下的用例**即使打上 mpi tag 也不得静默运行**。必须先经人工核验，再按「更新规则」把它明确列为按用例范围（在下面登记）。
- 打 tag 不锁定线程/进程数：若用例需特殊并行参数，仍按 DEVELOPMENT_STANDARD §2.6 在 `tests.toml` 中配置 `num_mpi`/`num_threads`。

### 当前按用例纳入的用例（dh 目录，2026-09 全部在 `mpirun -n 2` 下验证通过）

`C6H6p_R-xDH7_ROHF`、`C6H6p_R-xDH7_UHF`、`C6H6p_ZRPS_ROHF`、`C6H6p_ZRPS_UHF`、
`C6H6_RPA`、`C6H6_R-xDH7`、`C6H6_ZRPS`、`Cu2_MP2`、`H2_MP2`、`HI_ECP_MP2`、
`N2_roR-xDH7`、`N2_roXYG3`、`NH3_DSDDH_PARSEXC`、`NH3_RMP2_EXT_FIELD_DIPOLE`、
`NH3_RMP2_NUMDIPOLE`、`NH3_RPA`、`NH3_R-xDH7`、`NH3_R-xDH7_SCC15`、`NH3_ZRPS`、
`O2_MP2`、`O2_R-xDH7`、`O2_ZRPS`。

这 22 个用例覆盖 dh 目录当前的全部查询，`tests.toml` 中逐个带 `tags=["mpi"]`。

## 更新规则

- 只有当一个目录下的任务在 MPI 模式下经过完整验证并全部通过后，才允许将其加入“当前 MPI 测试范围”表格。
- 按用例纳入：只有当一个用例在目标 MPI 配置（如 `-n 2`）下与串行结果一致并通过验证后，才允许在 `tests.toml` 中加 `tags=["mpi"]`，并须在本文件记录。
- 更新本文件时，必须同步更新 `validate.sh` 中的范围检查逻辑（`SCOPE_DIRS` 与 mpi tag 解析），保持两者一致。
- 目录级降级为按用例（如 dh）：先把该目录下已通过 MPI 验证的用例逐个打 `tags=["mpi"]`，再从表格移除该目录行，并在本文件登记用例名单与理由。
- 缩小范围（移除目录或移除 tag）无需修改 `validate.sh`，但应更新本文件并说明原因。

## 标准验证命令速查

```bash
# rayon-only 全量（90 个任务，MPI 范围声明不影响本模式）
./validate.sh

# MPI 模式（自动限定在声明范围内：scf + gradient + dispersion + solvent，再加 tests.toml 中带 tags=["mpi"] 的 22 个 dh 用例）
./validate.sh --mpi

# MPI + 强制 ScaLAPACK 分布式路径（自动限定在声明范围内）
# 向每个任务的 ctrl.in 注入 hamiltonian_distributed = "on"（[ctrl]）与
# distributed = "on"（[ctrl.j2c_decomp]），强制走 ScaLAPACK 分布式求解。
./validate.sh --scalapack

# MPI 定向测试（若超出声明范围，脚本会打印 warning）
./validate.sh --mpi -f "NH3_SCAN"
./validate.sh --mpi -f "C6H6_ZRPS"   # dh 按用例范围内，不告警
```
