# H2O10_PBE0_grid_prune_off

## 1. 被测功能

格点一次式裁剪**关闭**的基线（`grid_prune_threshold` 缺省 0.0）。

与 `H2O10_PBE0_grid_prune_on` 同体系、同参数（含 `pair_screen_threshold = 0.0`），
唯一差别是不设 `grid_prune_threshold`。两例的能量差即裁剪的截断效应
（water10 / 1e-11 档约 4e-9 Ha），超出此量级的变化说明裁剪实现漂移。

## 2. 触发条件

不设该键（或 `grid_prune_threshold = 0.0`）。

## 3. 覆盖点

- 日志**不出现** `[grid-prune] ... dropped` 行（`threshold <= 0.0` 直接返回）。
- 全格点 SCF 正常收敛，能量落在参考值上。

## 4. 与相邻用例的分工

见 `H2O10_PBE0_grid_prune_on/prompt.md` 第 4 节。
