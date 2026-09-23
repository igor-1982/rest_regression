# H2O_PBE0_pair_prune_off

## 1. 被测功能

存储级（storage-level）AO 对剪枝的**关闭**路径。

`[ctrl.ri_jk] pair_screen_threshold` 取 0.0 时，`rimatr` 按全部 `nbaspar` 行构建与存储，
J/K 走未筛选内核，映射表 `basbas2baspar` / `baspar2basbas` 保持全量语义，`rimatr_pair_map` 为 `None`。

## 2. 触发条件

```toml
[ctrl.ri_jk]
pair_screen_threshold = 0.0
```

关键词**存在但取零值**，与完全不写该关键词等价。体系与基组：H2O、PBE0/def2-SVP +
def2-universal-jkfit、`eri_type = "ri_v"`、`use_ri_symm = true`。

## 3. 覆盖点

- 关键词的值（而不是它的存在）决定分派：写成 0.0 必须走缺省路径，日志里不出现
  "In-core RI pair screening enabled" 与 "Storage-level AO-pair pruning"。
- 参考能量 -76.2762703477 Ha 与 `H2O_PBE0_pair_screen_off` 相同（该用例不写关键词）。
  若分派条件写成"关键词是否存在"，本用例会读到剪枝路径的结果（该体系相差 1e-2 Ha 量级），
  远大于回归工具 1e-5 Ha 的容差。

## 4. 对照

| 用例 | 关键词 | 参考能量 (Ha) |
| --- | --- | --- |
| 本用例 | `pair_screen_threshold = 0.0` | -76.2762703477 |
| `H2O_PBE0_pair_screen_off` | 不写 | -76.2762703477 |
| `H2O_PBE0_pair_screen_on` | `1.0e-4` | -76.2919377798 |
| `H2O10_PBE0_pair_prune_on` | `1.0e-6` | 见该用例 |
