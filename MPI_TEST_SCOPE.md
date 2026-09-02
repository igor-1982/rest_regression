# MPI 测试范围声明（MPI Test Scope）

> **本文件是 rest_regression 批处理验证中 MPI 模式的测试范围声明。**
> 任何 MPI 相关功能测试与验证命令（包括 `validate.sh` 与手工执行的 `cargo run`）在运行前**必须先阅读本文件**。
> 一旦 MPI 功能测试超出本文件声明的范围，执行方必须给出 warning，不得静默运行。

## 当前 MPI 测试范围（Temporary Scope）

MPI 模式（`--n_mpi >= 2`，即 `mpirun -n N rest`）的回归测试**暂时仅限**以下两个目录：

| 目录 | 内容 |
|---|---|
| `bench_pool/scf/` | SCF 相关任务（单点能、偶极、BSSE、开壳层、RSH、meta-GGA 等） |
| `bench_pool/gradient/` | 解析梯度任务（RHF/UHF/UKS/RMP2/RSH 梯度、外场梯度、数值梯度优化等） |

## 范围外目录（Out of Scope，暂不纳入 MPI 批处理验证）

以下目录中的任务**暂不**纳入 MPI 模式的批处理验证：

- `bench_pool/gw_bse/` — GW/BSE 任务
- `bench_pool/hessian/` — Hessian 与频率任务
- `bench_pool/pbc/` — 周期边界任务
- `bench_pool/solvent/` — 溶剂模型任务
- `bench_pool/tddft/` — TDDFT 任务
- 其他未在“当前 MPI 测试范围”表格中列出的目录

这些任务在 MPI 模式下可能尚未验证或不支持 MPI（例如 `hessian/CO_HF_OPT_TRIC_ANALYTIC_HESS` 在 `-n 2` 下会触发
`ri_jk/pure_decompose.rs` 的 naux 断言失败）。它们仍应通过 **rayon-only（`-n 1`）** 模式验证。

## 更新规则

- 只有当一个目录下的任务在 MPI 模式下经过完整验证并全部通过后，才允许将其加入“当前 MPI 测试范围”表格。
- 更新本文件时，必须同步更新 `validate.sh` 中的范围检查逻辑（`SCOPE_DIRS`），保持两者一致。
- 缩小范围（移除目录）无需修改 `validate.sh`，但应更新本文件并说明原因。

## 标准验证命令速查

```bash
# rayon-only 全量（60 个任务，MPI 范围声明不影响本模式）
./validate.sh

# MPI 模式（自动限定在声明范围内：scf + gradient）
./validate.sh --mpi

# MPI + 强制 ScaLAPACK 分布式路径（自动限定在声明范围内）
# 向每个任务的 ctrl.in 注入 hamiltonian_distributed = "on"（[ctrl]）与
# distributed = "on"（[ctrl.j2c_decomp]），强制走 ScaLAPACK 分布式求解。
./validate.sh --scalapack

# MPI 定向测试（若超出声明范围，脚本会打印 warning）
./validate.sh --mpi -f "NH3_SCAN"
```
