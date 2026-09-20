# H2O_radii_adjust_treutler

## 1. 被测功能

DFT 格点 Becke 划分中的原子半径调整方案，由 ctrl 关键词 `radii_adjust` 选择：

- `becke`：用 Bragg 半径本身，即 Becke 原文（JCP 88, 2547 (1988)）的写法，REST 的历史行为。
- `treutler`：用 Bragg 半径的平方根，即 Treutler-Ahlrichs（JCP 102, 346 (1995)）的写法，也是
  PySCF `Grids` 与 PyFock 的缺省。
- 不写该关键词时缺省为 `becke`。

两种方案只改权重、不改格点点位与点数，且都满足每个格点上 $sum_A p_A = 1$，差别是粗格点下的
积分误差，随格点加密衰减。

## 2. 触发条件

ctrl.in 写 `radii_adjust = "treutler"`，检验平方根方案。

其余关键词固定：`grid_generation_level = 0`、PBE/def2-SVP、`eri_type = "ri_v"`。

选用 level 0 的粗格点是有意的：两种方案的 SCF 能量差在该格点下为 2.3e-3 Ha，远大于回归
工具对 SCF 能量的 1e-5 Ha 容差，因此关键词一旦失效本用例必然失败。level 3 下该差只有 2e-7 Ha，
落在容差内，无法判别。

## 3. 覆盖点

显式 `treutler` 分支：解析必须映射到平方根半径方案。若关键词被忽略、或映射到 Becke，本用例能量会回到 -76.2544076928 Ha，偏离参考值 2.3e-3 Ha 而失败。

格点点数不受关键词影响（两种方案都是 2278 点），本用例只锁定权重。

## 4. 对照

参考值由本次改动的 release 构建（关键词实现完成后）生成，REST 自身两种方案的能量在同一点位上
只差权重。外部对照：同一几何、同一泛函基组下 PySCF 2.3.0（`radi.becke_atomic_radii_adjust` 与
`radi.treutler_atomic_radii_adjust`，均带 def2-universal-jkfit 的 density fitting）在 level 3 的
(H2O)5 上给出 -381.4184303558 与 -381.4184302778 Ha，REST 对应值为 -381.4184303559 与
-381.4184302778 Ha，两种方案都对齐到 1e-10 Ha。

