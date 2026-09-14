# C6H6p_ZRPS_ROHF: 2.5D MPI SBGE2 (ZRPS) 验证用例
被测功能: DFAFamily::SBGE2 的 2.5D MPI 路径（rest/src/ri_pt2/sbge2_25d.rs），use_ri_symm=true 且 n_occ>=20 时触发。
小配置: basis=3-21g; aux=def2-svpd-rifit（注：def2-SV(P)-JKFIT 是 def2-universal-jkfit 的别名，并非更小）; grid_generation_level=2。
覆盖点: serial vs 25d mpi(n=2/3/4) 能量一致(<=1e-9)。release 串行计时: C6H6 5.5s / UHF 11.5s / ROHF 11.2s。
