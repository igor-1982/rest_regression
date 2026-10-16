# H2O10_PBE0_grid_prune_on

## 1. 被测功能

初猜密度驱动的**分子格点一次式裁剪**（grid pruning by initial-guess density）。

`[ctrl] grid_prune_threshold > 0` 时，`initialize_scf` 在初猜密度矩阵就位后调用
`Grids::prune_by_density`：逐点计算判据 `Σ_s |ρ_s(p)| · |w_p|`，低于阈值的格点被
永久删除（判据含积分权重，避免误删远尾大权重点），已制表的 AO/AOP 值按存活点
**就地压缩**（`grid_prune_compact = true` 缺省）而不是重新制表。裁剪后所有消费者
（SCF/梯度/Hessian）看到的是同一张更小的格点表。

体系与基组：water10、PBE0/def2-SVP + def2-universal-jkfit、`eri_type = "ri_v"`、
`use_ri_symm = true`、格点缺省 level 3，`pair_screen_threshold = 0.0`（隔离 RI 对
预筛，本用例只考察格点裁剪）。

## 2. 触发条件

```toml
[ctrl]
grid_prune_threshold = 1.0e-11
```

## 3. 覆盖点

- 裁剪被走到且走了压缩路径，日志出现：
  ` [grid-prune] threshold 1.0e-11: dropped 71316 of 343100 grid points (20.8%), they carried 2.764e-8 of the 1.022e2 total density contribution (2.70e-10), largest dropped term 9.999e-12; tabulated AO/AOP compacted in place`
  （dropped 计数与判据是逐点确定性结果，与线程数无关；改动判据或 ρ 评估的实现会
  反映在这行数字上，属于**必须人工复核**的变化。）
- `print_level ≥ 2` 时的分段计时行：
  ` [grid-prune] timing: rho ... s (N threads), decide + gather ... s, compact ... s`
  ——ρ 评估应为多线程（线程数 ≈ num_threads），是 2026-10-17 修复 1016 版本
  InitGuess 回归（串行 ρ + 被丢弃的 ∇ρ）的守护指标。
- 压缩后 SCF 正常收敛，能量落在参考值上（与 `grid_prune_off` 的差 ~4e-9 Ha，
  是文档化的裁剪截断效应，容差 1e-5 Ha 内）。

## 4. 与相邻用例的分工

| 用例 | grid_prune_threshold | 覆盖 |
| --- | --- | --- |
| `H2O10_PBE0_grid_prune_on`（本用例） | 1.0e-11 | 裁剪 + 就地压缩 + 裁剪后 SCF |
| `H2O10_PBE0_grid_prune_off` | （缺省 0.0） | 同体系不裁剪基线 |
| `H2O10_PBE0_pair_prescreen_on` | 0.0 | RI 对构建期预筛（另一维度） |

## 5. 注意

- 判据用的 ρ 来自**初猜密度**（SAD），不是收敛密度：裁剪决策发生在 SCF 之前。
- 大体系上阈值不应放宽到 1e-9 以上（见 `GRID_PRUNE_AUDIT.md` §7：误差窗口随体系
  增大左移）；1e-11 在所有已测体系上误差 ≤ 1e-8 Ha 量级。
