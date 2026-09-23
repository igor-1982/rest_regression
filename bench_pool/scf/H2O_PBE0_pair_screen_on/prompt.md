# H2O_PBE0_pair_screen_on

## 1. 被测功能

incore RI-J/RI-K 内核的 AO 对（basis pair）筛选，判据与实现见 `H2O_PBE0_pair_screen_off`
的说明。本用例打开筛选，检验筛选路径能被实际走到、且数值落在参考值上。

## 2. 触发条件

```toml
[ctrl.ri_jk]
pair_screen_threshold = 1.0e-4
```

该阈值是**相对**阈值：对界值低于“本次迭代最大对界值 × 1.0e-4”的对被丢弃。

体系与基组：H2O、PBE0/def2-SVP + def2-universal-jkfit、`eri_type = "ri_v"`、
`use_ri_symm = true`（走 `rimatr` 的 incore 路径）、格点取缺省 level 3。
日志里会出现 "In-core RI pair screening enabled (threshold 1.0e-4, 300 AO pairs)."。

## 3. 覆盖点

- 阈值分支：`pair_screen_threshold > 0` 时分派到筛选内核，J 与 K 同时生效。
- 相对阈值 1.0e-4 在该体系上丢掉相当一部分对，能量相对未筛选路径移动 1.57e-02 Ha，远大于
  回归工具 1e-5 Ha 的容差。若分派失效（回到未筛选内核），本用例会读到 -76.2762703477 Ha
  而失败，所以它同时锁定“筛选路径确实被使用”。

## 4. 对照

未筛选对照见 `H2O_PBE0_pair_screen_off`（-76.2762703477 Ha）。阈值扫描（H2O，PBE0/def2-SVP）
实测：1e-10、1e-8、1e-6 相对阈值与未筛选逐位相同，1e-4 移动 -1.57e-02 Ha，1e-2 移动
-1.21e+00 Ha。本用例取 1e-4 是为了让两条路径在回归容差下可区分，不代表生产推荐阈值。

## 5. 与存储级剪枝的关系（2026-09 起）

同一个关键词现在驱动**两级**筛选：

1. 存储级（构建期）：算完 `rimatr` 后丢掉静态行界 `q_p = max_P |R[p,P]|` 低于
   `tol * max_p q_p` 的行，把张量原地压紧，J/K 之后按 `rimatr_pair_map` 取存储行；
2. 使用期（缩并期）：在存储行里再按密度/轨道权重丢掉本轮不重要的行。

H2O 这个紧凑体系的 300 个 AO 对行界彼此都在 1.0e-4 以内，所以 1.0e-4 下存储级一行都不丢
（日志 "300 of 300 AO-pair rows stored"），本用例的参考能量因此与改动前逐位相同：
-76.2919377798 Ha。这条恰好锁住了重接线后的**数值中性**：当存储保留集合包含使用期保留集合时，
两条实现必须给出一模一样的数。存储级真正生效的对照见 `H2O10_PBE0_pair_prune_on`。
