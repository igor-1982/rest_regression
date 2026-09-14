# C6H6_RPA: 2.5D MPI dRPA 验证用例
被测功能: DFAFamily::RPA 的 2.5D MPI 路径（rest/src/ri_rpa/rpa_25d.rs），use_ri_symm=true 且任意 n_occ（RPA 无 1D MPI 回退）。
小配置: basis=3-21g; aux=def2-svpd-rifit; grid_generation_level=2; frequency_points=20。
覆盖点: serial vs 25d mpi(n=2/3/4) 能量一致(<=1e-8)。
