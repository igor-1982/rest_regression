# H2O_PBE0_pair_prune_on

## 1. 被测功能

存储级 AO 对剪枝的**构建期预筛**在**边界情形**下的行为：预筛算出的保守超集就是全部 AO 对。

## 2. 触发条件

```toml
[ctrl.ri_jk]
pair_screen_threshold = 1.0e-6
```

体系只有 300 个 AO 对，壳对 Schwarz 界在 `threshold / 10 = 1.0e-7` 的相对cut 下
一个也筛不掉，所以：

- 预筛保留 `300 of 300`，行空间没有被压缩；
- 剪枝也保留 `300 of 300`。

这正是 `H2O_PBE0_pair_prune_off`（阈值 0.0）的对照：两条路径都必须给出同一个能量。

## 3. 覆盖点

- 预筛在"无行可筛"时不报错、不误删：
  `In-core RI pair pre-screen: 300 of 300 AO pairs kept for the build (threshold 1.0e-6, safety factor 10).`
- 压缩路径在 `kept == all` 时退化为原行为：
  `Storage-level AO-pair pruning: 300 of 300 AO-pair rows stored (100.00%)`。
- 能量与 `H2O_PBE0_pair_prune_off` 一致。

## 4. 注意

本用例的系统太小，`H2O10_PBE0_pair_prescreen_on` 才是预筛真正被触发的例证；
本用例负责的是"预筛保留全部"这个边界不被改坏。
