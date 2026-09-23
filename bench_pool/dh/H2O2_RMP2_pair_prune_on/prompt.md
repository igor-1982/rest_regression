# H2O2_RMP2_pair_prune_on

## 1. 被测功能

存储级 AO 对剪枝打开后，**后自洽场相关能的 AO→MO 变换**（S2 接线）仍读同一个紧凑行布局。

`pair_screen_threshold = 1.0e-6` 时 `rimatr` 只存 `q_p > 1e-6 * max_p q_p` 的行（本用例 1176 行
存 939 行，丢掉 20.2%），行布局记入 `PairMap`。`xc = "mp2"` 走 streaming PT2 驱动，该驱动每一步
调用 `scf_io::ao2mo_rayon_m1` 把三中心张量的若干列变换到 MO 空间。内核通过
`ri_jk::fill_ao_matrix_from_column` 按 `(mu, nu)` 散射到对称矩阵，被丢掉的行留在零。

## 2. 触发条件

```toml
[ctrl.ri_jk]
pair_screen_threshold = 1.0e-6
```

外加 `xc = "mp2"`、`eri_type = "ri_v"`、`use_ri_symm = true`（后两者保证 `rimatr` 被构建），
体系为水二聚体（O···O 2.910 A，H···O 1.953 A），def2-SVP + def2-universal-jkfit。

## 3. 覆盖点

- 端到端锁住"剪枝开启时后自洽场相关能仍正确"：MP2 相关能 -0.40898549 Ha 与未剪枝的
  -0.40898538 Ha 相差 1.1e-7 Ha，与 1.0e-6 的截断尺度一致。
- 若 AO→MO 变换仍按全量 packed 顺序读紧凑张量（S1 交付时的状态），行会整体错位。实测同一
  缺陷在 `NH3_RPA` 的 1.0e-2 档把 Ec[RPA] 从 -0.1892 Ha 变成 -1.9961 Ha，远大于回归工具的
  1e-5 Ha 容差，所以本用例对这类回归敏感。
- 单元测试 `pruned_map_matches_zeroed_full_rows` 用合成张量证明"子集 map"与"全量张量把丢弃
  行清零"逐位相同，覆盖 v02 / m1 / m2 三个内核与全部块形状。
- 存储层丢弃集合非空（237 行），满足"最小验证档必须真的丢行"这条规则。

## 4. 对照

| 用例 | 阈值 | 存储行 | Ec[PT2] (Ha) | SCF 能量 (Ha) |
| --- | --- | --- | --- | --- |
| 本用例 | 1.0e-6 | 939/1176 | -0.40898549 | -151.9299274319 |
| `H2O2_RMP2_pair_prune_off` | 0.0 | 1176/1176 | -0.40898538 | -151.9300026261 |
| `scf/H2O10_PBE0_pair_prune_on` | 1.0e-6 | 10235/28920 | 不适用 | -762.8486075242 |
