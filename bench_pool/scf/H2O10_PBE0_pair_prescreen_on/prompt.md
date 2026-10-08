# H2O10_PBE0_pair_prescreen_on

## 1. 被测功能

存储级 AO 对剪枝的**构建期预筛**（a-priori pre-screen）。

`[ctrl.ri_jk] pair_screen_threshold > 0` 时，`rimatr` 的存储剪枝本来只能作用于**建好的**张量，
所以三中心积分与度量变换仍要付全部 AO 对的钱。预筛从**任何三中心积分之前**可得的
壳对 Schwarz 界出发，先挑出一个**保守超集**，只在这个更小的行空间上做度量变换与剪枝。

## 2. 触发条件

```toml
[ctrl.ri_jk]
pair_screen_threshold = 1.0e-12
```

预筛在 `pair_screen_threshold > 0`、`use_ri_symm = true` 且**非 MPI** 时自动启用。
安全系数为 10：预筛用 `pair_screen_threshold / 10` 作阈值，因为 Schwarz 界只约束
**未变换**的三中心块，而剪枝作用在**度量变换后**的行最大值上，两者排序不同。

体系与基组：water10、PBE0/def2-SVP + def2-universal-jkfit、`eri_type = "ri_v"`、
`use_ri_symm = true`、格点缺省 level 3。

## 3. 覆盖点

- 预筛被走到，日志出现：
  `In-core RI pair pre-screen: 20219 of 28920 AO pairs kept for the build (threshold 1.0e-12, safety factor 10).`
- 行空间确实被压缩：后续 `[rimatr-time]` 的 `nao_tp` 是 `20219` 而不是 `28920`。
- 剪枝仍在压缩行空间上运行，并给出预期的保留集：
  `Storage-level AO-pair pruning: 18584 of 28920 AO-pair rows stored (64.26%)`。
- 最终 SCF 能量落在参考值上。

## 4. 与相邻用例的分工

| 用例 | 阈值 | 覆盖 |
| --- | --- | --- |
| `H2O10_PBE0_pair_prune_on` | 1.0e-6 | 存储剪枝本身，行空间全量构建 |
| `H2O10_PBE0_pair_prescreen_on`（本用例） | 1.0e-12 | 预筛 + 在**压缩**行空间上剪枝 |
| `H2O_PBE0_pair_prune_on` | 1.0e-6 | 预筛保留全部 AO 对的边界情形 |
| `H2O_PBE0_pair_prune_off` | 0.0 | 剪枝关闭 |

## 5. 注意

预筛只改变**构建路径**，不改变最终保留的行集与 `PairMap`：预筛集合是剪枝集合的超集，
且行数从 28920 变成 20219 只影响 BLAS 三角求解的分块，末位可差 ~1e-10 Ha。
回归容差 1e-5 Ha 远大于此。
