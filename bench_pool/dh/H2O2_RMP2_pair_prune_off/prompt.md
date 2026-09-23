# H2O2_RMP2_pair_prune_off

## 1. 被测功能

存储级（storage-level）AO 对剪枝的**关闭**路径，配上后自洽场相关能（S2 之后的关键词语义）。

`[ctrl.ri_jk] pair_screen_threshold` 取 0.0 时 `rimatr` 按全部 `nbaspar` 行构建与存储，
`rimatr_pair_map` 为 `None`，AO→MO 变换走未映射的 packed 上三角顺序。

## 2. 触发条件

```toml
[ctrl.ri_jk]
pair_screen_threshold = 0.0
```

关键词**存在但取零值**，与完全不写等价。体系与方法：水二聚体（O···O 2.910 A，H···O 1.953 A），
`xc = "mp2"`（PT2 family，后自洽场相关能）、def2-SVP + def2-universal-jkfit、`eri_type = "ri_v"`、
`use_ri_symm = true`。

## 3. 覆盖点

- 分派由**值**决定：写成 0.0 必须走缺省路径，日志里不出现 "Storage-level AO-pair pruning"。
- 该档锁住未剪枝的 AO→MO 变换（`map = None`）与 streaming PT2 驱动的参考数值。
- 与 `H2O2_RMP2_pair_prune_on` 成对：两档的 MP2 相关能相差 1.1e-7 Ha，SCF 能量相差 7.5e-5 Ha
  （剪枝档的缩并期判据也随阈值收紧）。

## 4. 对照

| 用例 | 阈值 | 存储行 | Ec[PT2] (Ha) | SCF 能量 (Ha) |
| --- | --- | --- | --- | --- |
| 本用例 | 0.0 | 1176/1176 | -0.40898538 | -151.9300026261 |
| `H2O2_RMP2_pair_prune_on` | 1.0e-6 | 939/1176 | -0.40898549 | -151.9299274319 |
