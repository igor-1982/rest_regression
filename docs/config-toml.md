# 测试参考模式与 `config.toml` 配置

本文档定义 rest_regression 的 **json 参考模式**：参考模式的缺省值为 **auto**——目录中存在 `reference.json` 即按 json 比较，否则按 log 比较；测试目录也可以通过一个可选的 `config.toml`，显式声明参考模式并指定必查字段与容许误差覆盖。文档面向两类读者：想为新物理量建立回归测试的测试编写者（第 1–4 节），以及需要扩展默认容许误差表或比较规则的 rest_regression 维护者（第 5 节）。

## 1. 背景与目的

默认的 log 模式下，rest_regression 用正则表达式从运行日志中抽取观测量（`src/main.rs` 的 `collect_results`），再与 `reference.log` 的抽取结果按硬编码容许误差比较（`compare_results`）。每引入一个新观测量，都要同时新增正则和容许误差分支。

REST 程序本身会在计算结束时自动写出机器可读的 json 结果文件（`rest/src/fileop/json_dump.rs`），其中包含 SCF 能量、偶极、梯度、频率、热化学等全部观测量。json 参考模式直接比较这份文件，好处是：

- 不必为每个新字段修改 rest_regression；但作为代价，测试编写者需要保证在生成 `reference.json` 中引入新字段；
- 未来 REST 版本向 json 中新增字段，不会破坏已有参考（见第 4 节的交集规则）；
- 数值、字符串、数组都有统一的比较语义。

参考模式缺省为 `auto`（见第 3 节）：`ctrl.in` 设置 `outname = "reference"` 后，按常规方式生成 `reference.log` 的同时就会得到 `reference.json`，测试即自动按 json 比较。因此**新测试通常无需 `config.toml`**。

## 2. 测试目录约定

json 参考模式的测试目录在原有文件的基础上多了两个可选文件：

| 文件 | 必需 | 说明 |
|---|---|---|
| `ctrl.in`（或 `rest_jobs`） | 是 | REST 输入卡；json 比较要求设置 `outname = "reference"`，使结果 json 输出为 `reference.json` |
| `reference.log` | 是 | 参考日志；测试发现的依据仍然是目录中存在 `reference.log` |
| `reference.json` | json 比较时必须 | 参考结果，由 REST 自动写出；auto 模式（缺省）凭它的存在自动选用 json 比较 |
| `config.toml` | 可选 | 仅在需要显式声明参考模式、必查字段或容许误差覆盖时提供（见第 3 节） |
| `prompt.md` / `README` | 可选 | 测试说明 |

注意：

- 缺省参考模式为 auto：只要 `ctrl.in` 设置了 `outname = "reference"`，按下述方式生成 `reference.log` 时 `reference.json` 会同时落在目录中，测试即自动按 json 比较；仅当需要必查字段、容许误差覆盖或强制 log 模式时才需要写 `config.toml`。
- REST 默认的 json 文件名是 `rest_results.json`；`outname = "reference"` 只是约定，程序并不强制。但若要求使用 `json` 格式的数值结果比对，则必须要这么设置：若输入卡漏写这一项，测试会带着明确的 `REST did not produce reference.json` 信息失败。
- 参考文件在测试目录内生成：

```bash
# export REST_BASIS_DIR=$REST_HOME/rest/basis-set-pool
$REST_HOME/target/release/rest -i ctrl.in > reference.log
# reference.json 由 REST 自动写出
```

- 多步任务（`rest_jobs`）中每个任务都会覆写 `reference.json`，最终只有**最后一个任务**的结果参与比较，这与 log 模式下日志被逐任务截断的行为一致。

## 3. `config.toml` 语法

```toml
[config]
# 参考类型："auto"、"json" 或 "log"。缺省为 "auto"：目录中存在
# reference.json 时按 json 比较，否则按 log 比较（后者完全等价于没有
# config.toml 的既有行为，此时本文件其余字段均被忽略）。显式 "json"/"log"
# 优先于自动检测。
reference = "auto"

# 必查字段列表（可选）。每个字段必须同时存在于 reference.json 与本次
# 运行输出的 json 中，否则测试失败。路径以 "/" 表示 json 嵌套，
# 例如 "thermo/cv_vib" 对应 { "thermo": { "cv_vib": ... } }。
fields = [
    "scf_energy",
    "analdrv/frequencies_cm",
    "thermo/zpe",
    "thermo/cv_vib",
]

[tolerance]
# 用户容许误差覆盖（可选）。数值判据与 numpy 的 isclose/allclose 相同：
#   |out - ref| <= atol + rtol * |ref|    （rtol 以参考值为基准）
# 单个值表示 atol（rtol 取默认值 1e-5）；两个值的数组表示 [atol, rtol]。
# 出现在此表中的字段自动进入必查字段列表。
# 量级远小于 atol 的小数值字段必然通过比较；如需严格检验，应设
# atol = 0，仅按 rtol 作相对比较。

# 若不作设置，则按默认判据 [1e-8, 1e-5]：绝对误差 1e-8，相对误差 1e-5

# 允许 1 J/mol 的零点能误差，相对误差 1e-5
"thermo/zpe" = 1.0
# 允许 1e-6 Hartree 的电子能量误差，相对误差 1e-4
"thermo/electronic_energy_au" = [1e-6, 1e-4]
```

规则要点：

- **大小写**：`reference` 的取值不区分大小写；json 字符串值的比较同样不区分大小写（见第 4 节）。
- **未知键**：`config.toml` 中出现无法识别的段或键会直接报错终止，避免拼写错误被静默忽略。
- 容许误差必须是非负有限值；atol 与 rtol 对标量与数组一视同仁，作用于数组的**每个元素**（见第 4 节）。

## 4. 比较规则

设参考文档为 R（`reference.json`），输出文档为 O（本次运行产生）。

1. **叶子字段**：将 R 与 O 分别展平为"路径 → 值"。叶子指任何非对象值；**数组整体视为一个叶子**；空对象不产生叶子。例如 `{"thermo": {"cv_vib": 1.5}}` 展平出 `thermo/cv_vib`。
2. **交集比较**：只比较同时在 R 与 O 中存在的叶子。因此，未来 REST 版本新增的字段（O 中有、R 中无）不参与比较，不会破坏旧参考；R 中有、O 中缺失的字段同样不比较，**除非**它是必查字段。
3. **必查字段**：即 `fields` 与 `[tolerance]` 键的并集。每个必查路径必须在 R 与 O 中**均可解析**（允许解析到对象本身，例如 `fields = ["thermo"]` 表示检查整个热化学块存在），否则测试失败。
4. **容许误差优先级**：用户 `[tolerance]` > 默认容许误差表（第 5 节）> 默认值（atol = 1e-8，rtol = 1e-5）。
5. **值比较语义**（对每对叶子）：

| 类型组合 | 规则 |
|---|---|
| 数值 × 数值 | `\|out − ref\| <= atol + rtol · \|ref\|`，与 numpy 的 isclose 相同（rtol 以**参考值**为基准；整数同样按数值处理） |
| 字符串 × 字符串 | 不区分大小写的相等 |
| 布尔 × 布尔 | 相等 |
| null × null | 相等 |
| 数组 × 数组 | 长度必须一致，且逐元素递归应用本表（每个数值元素独立套用上述 isclose 判据） |
| 其它（类型不匹配） | 失败 |

6. **路径导航**：`/` 只穿越 json 对象，不支持数组下标。

与 log 模式有两处不同，均为有意为之、且更严格：

- log 模式对偶极、梯度按差向量的二范数判据；json 模式统一为逐元素判据（对三维量约严格 √3 倍，实际影响可以忽略）。
- log 模式对频率、TDDFT 数组只比较到较短一方的长度；json 模式要求数组**长度一致**。

## 5. 默认容许误差表（维护者指南）

默认容许误差由 `src/json_ref.rs` 中的 `default_tolerance(path)` 给出，取值逐项沿用 log 模式 `compare_results`（`src/main.rs`）中已有的判据（唯一例外是 `analdrv/frequencies_cm`，atol 0.1，较 log 模式的 2.0 有意收紧）。表本身写在独立的资源文件 `src/default_tolerance.toml` 中，经 `include_str!` 在**编译期**嵌入二进制（运行时不读取磁盘），首次使用时解析一次。当前条目清单以该文件为准，本文档不再罗列，以免两处失同步。

沿用 log 模式判据的表项一律写成 `[atol, 0.0]`，即 rtol = 0 的纯绝对判据，与原先的绝对容许误差严格等价；唯一例外是 `analdrv/frequencies_cm`，只写 atol（rtol 取默认值 1e-5）。未登记的路径一律落入兜底（atol 1e-8、rtol 1e-5）。

维护约定：

- 新观测量若在 log 模式中已有判据，应把 json 路径与 `[atol, 0.0]` 一并登记进 `src/default_tolerance.toml`。
- 判据自带相对项（默认 rtol = 1e-5）：只要参考值明显超过 atol/rtol（默认判据下约 1e-3），**大数值**字段（如 `thermo/q_trans` ~1e30）就由 rtol 项控制，自动获得合适的相对容许误差，一般无需逐字段设置；只有当某字段的数值噪声天然超出相对判据（例如对频率敏感的热力学量）时，才在 `[tolerance]` 中显式给出 `[atol, rtol]`。
- 反之，**小数值字段必然通过**：当字段量级远小于 atol（兜底 1e-8，如 `thermo/q_vib_bot` ~1e-17）时，绝对项完全主导判据，比较实际不起甄别作用；若确需检验这类小量级字段，应在 `[tolerance]` 中显式设置 `atol = 0`，使判据退化为纯相对比较（注意此时若参考值恰为 0，则要求输出严格相等）。
- 修改 `src/default_tolerance.toml` 后需要**重新编译** rest_regression（表在编译期嵌入）；改完运行 `cargo test`，`json_ref` 的单元测试会核对表中的条目数与数值。

## 6. 输出与失败诊断

json 模式下，每个测试会打印与 log 模式同风格的逐字段报告，例如：

```
                                     Current Value vs      Reference Value
Pass    : scf_energy                    : Number(-56.14774031782696)  = Number(-56.14774031782704) (atol 1.0e-5, rtol 1.0e-5)
WARNING : thermo/cv_vib                 : Number(1.4716724712551368) != Number(1.4616724713067903) (atol 1.0e-8, rtol 1.0e-5)
JSON fields: 47 compared, 1 failed
```

常见失败：

| 现象 | 原因 |
|---|---|
| `required field not found in reference.json` / `in output json` | 必查字段在对应文档中不存在（REST 改名或未输出该字段，或 `fields` 拼写错误） |
| `REST did not produce reference.json in the job dir` | ctrl.in 未设置 `outname = "reference"` |
| 数组 `!=` 且长度不同 | REST 输出的数组长度与参考不一致（结构变化） |
| `config.toml: [config].reference must be "auto", "json" or "log"` | 配置取值错误，整个运行终止 |

另有一条不影响判定的提示：若 `config.toml` 写了 `fields` 或 `[tolerance]`，但测试实际运行在 log 模式（例如漏放了 `reference.json`），会打印 `WARNING : config.toml carries fields/tolerance ...`，提醒这些设置未生效。

运行结束后，本次输出的 json 保留在 `work_pool/<TestName>.json`，日志保留在 `work_pool/<TestName>.log`，便于人工比对。
