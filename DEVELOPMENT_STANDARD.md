# REST 程序开发标准（Development Standard）

> **本文件是 REST 程序开发的正确性验证标准，是所有开发者（含 AI 编程代理）必须遵守的工程阀门。**
> 任何功能开发（bug 修复、新方法、性能优化、并行化改造等）**完成后，必须通过 rest_regression 批处理验证**，
> 全部通过才能视为开发成功；未通过验证的代码不得提交、不得标记为完成。

---

## 1. 验证流程（Development Completion Gate）

### 1.1 功能开发完成后必须执行

```bash
# 第一步：rayon-only 全量验证（覆盖 bench_pool 全部任务）
cd $REST_HOME/rest_regression
./validate.sh

# 第二步：MPI 验证（范围由 MPI_TEST_SCOPE.md 声明，当前为 scf + gradient）
./validate.sh --mpi

# 第三步（改动涉及 ScaLAPACK/分布式计算路径时）：强制分布式验证
./validate.sh --scalapack
```

- **rayon-only（`-n 1`）**：验证串行 + rayon 线程并行的正确性，覆盖 `bench_pool/` 下全部任务；
- **MPI（`--mpi`，`mpirun -n 2`）**：验证 MPI 并行正确性，**范围暂时仅限 `bench_pool/scf/` 与 `bench_pool/gradient/`**
  （见 [MPI_TEST_SCOPE.md](./MPI_TEST_SCOPE.md)）。执行前必须先阅读该文件；任何超出声明范围的 MPI 测试必须给出 warning；
- **ScaLAPACK（`--scalapack`）**：在 MPI 基础上**强制**走 ScaLAPACK 分布式路径（向每个任务的 `ctrl.in`
  注入 `hamiltonian_distributed = "on"` 到 `[ctrl]`、`distributed = "on"` 到 `[ctrl.j2c_decomp]`），
  用于验证 MPI+ScaLAPACK 组合功能；同样受 `MPI_TEST_SCOPE.md` 范围约束。

### 1.2 定向验证（开发迭代中快速自检）

```bash
./validate.sh -f "NH3_SCAN"                  # rayon-only 单任务
./validate.sh --mpi -f "NH3_CAM-B3LYP_GRAD"  # MPI 单任务（限声明范围内）
```

### 1.3 完成标准（Definition of Done）

- [ ] `./validate.sh`（rayon-only 全量）全部通过
- [ ] `./validate.sh --mpi`（MPI 范围内全量）全部通过
- [ ] 若新功能引入了新物理量/输出格式，已选择一种方式为其建立对比：为 rest_regression 的
      `collect_results()`/`compare_results()` 添加解析与对比逻辑，**或**采用 json 参考模式
      （ctrl.in 设 `outname = "reference"` 并提供 `reference.json`，即自动按 json 比较；如需
      必查字段或容许误差覆盖，再放置 `config.toml`，见 [docs/config-toml.md](./docs/config-toml.md)），并补充测试
- [ ] 新功能已提供 rest_regression 测试例子（见第 2 节）
- [ ] 提交时附上验证结果（`Regression summary: N/N passed`）

---

## 2. 新功能测试例子规范（New-Feature Regression Test Spec）

每个新功能完成后，**必须**在 `bench_pool/` 下提供一个（或多个）回归测试例子。
测试设计原则：**体系小、逻辑简单、覆盖功能方方面面**。

### 2.1 目录结构

```
bench_pool/<category>/<TestName>/
├── ctrl.in          # [必须] REST 输入卡（TOML），最小化但完整触发被测功能
├── reference.log    # [必须] 参考输出（由通过验证的 REST 版本生成）
├── rest_jobs        # [可选] 多步测试：每行一个输入卡名（按序执行，输出合并对比）
├── prompt.md        # [可选] 中文说明：被测功能点、覆盖清单
├── README           # [可选] 英文说明
├── config.toml      # [可选] json 参考模式的必查字段与容许误差覆盖；参考模式缺省为 auto：
│                   #      目录含 reference.json 即按 json 比较（此时 ctrl.in 需设
│                   #      outname = "reference"），显式 reference = "json"/"log" 可覆盖检测
└── reference.json   # [json 比较时必须] REST 自动写出的参考结果；存在即自动选用 json 比较
```

### 2.2 命名规范

`分子_方法[_后缀]`，例如 `NH3_SCAN`、`NH3_RHF_GRAD`、`H2_PBE_TDA`、`CO_HF_OPT_TRIC`。

### 2.3 设计要求

| 原则 | 要求 |
|---|---|
| **体系小** | 原子数 ≤ 6（如 H2、H2O、NH3、CO、HI）；基组 ≤ cc-pVDZ / def2-TZVP；运行时间目标 < 60 s |
| **逻辑简单** | 单点能 / 梯度 / 一个物性为主；避免一次测试混杂过多功能 |
| **覆盖方方面面** | 被测功能的每个代码分支 / 参数组合都要有对应例子或输入卡组合 |
| **可复现** | 基组用名称（`cc-pVDZ` 等）而非绝对路径；不依赖网络（除 BSE 下载 fallback） |
| **参考值精度** | `reference.log` 浮点数 ≥ 10 位小数，确保 1e-5 容许误差对比可靠 |

### 2.4 覆盖清单（prompt.md 应写明）

每个测试例子应在 `prompt.md` 中列出：

1. **被测功能**：一句话说明（如 "RI-V 辅助基组 Cholesky 分解的 MPI 并行路径"）；
2. **触发条件**：ctrl.in 中哪些关键词激活该功能（如 `policy = "cholesky"`、`distributed = "on"`）；
3. **覆盖点**：枚举该功能涉及的分支/参数/边界（如：Eig vs Cd 策略、nproc=1/2/4、开壳层、梯度、外场等）；
4. **对照**：参考值来源（哪个已验证版本、串行结果等）。

### 2.5 多步测试（rest_jobs）

需要按序执行多个输入卡的场景（如基态 → 激发态、guess 文件传递），不建 `ctrl.in`，改建 `rest_jobs`：

```
ctrl_gs.in
ctrl_dscf.in
```

回归工具按序执行全部输入卡，收集输出后与 `reference.log` 对比。

### 2.6 特殊并行配置（tests.toml）

对并行参数有特殊要求的测试，在 `rest_regression/tests.toml` 中配置（不会被 CLI `-n`/`-t` 覆盖）：

```toml
[TestName]
num_mpi = 1        # 强制 rayon-only（如某些功能暂不支持 MPI）
num_threads = 3    # 锁定线程数
timeout = 300      # 单独超时
tags = ["mp2"]     # 供 -f 按 tag 过滤
```

---

## 3. 验证结果记录与提交

- 每次验证保留 `work_pool/<TestName>.log` 输出；
- 提交（commit）时在提交信息中注明验证结果，例如：
  `feat: ... (regression: rayon 60/60, mpi 29/29 passed)`；
- 若验证失败，先修代码或更新 reference（仅当结果因算法改进而**正确变化**时允许更新
  `reference.log`，且必须说明原因）。

---

## 4. 相关文件

| 文件 | 作用 |
|---|---|
| `rest_regression/validate.sh` | 标准验证入口（rayon-only 全量 / MPI 范围验证） |
| `rest_regression/MPI_TEST_SCOPE.md` | MPI 测试范围声明（执行 MPI 前必须阅读） |
| `rest_regression/docs/config-toml.md` | json 参考模式规则（config.toml 语法、容许误差表、比较语义） |
| `rest_regression/src/main.rs` | 回归工具主程序（collect_results / compare_results，log 参考模式） |
| `rest_regression/src/json_ref.rs` | json 参考模式实现（config.toml 解析、默认容许误差表、比较） |
| `rest_regression/src/default_tolerance.toml` | 默认容许误差表资源文件（编译期 `include_str!` 嵌入二进制） |
| `rest_regression/src/filter.rs` | 测试过滤与 tests.toml 配置 |
| `rest_regression/tests.toml` | 特殊测试并行配置 |
| `rest-dev-ai-agent/skills/rest-regression-guide/SKILL.md` | 回归测试添加/维护指南 |
