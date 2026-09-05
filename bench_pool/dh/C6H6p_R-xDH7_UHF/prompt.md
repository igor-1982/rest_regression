# C6H6p_R-xDH7_UHF: 2.5D MPI SCS-RPA (R-xDH7) 验证用例
被测功能: DFAFamily::SCSRPA 的 2.5D MPI 路径（rest/src/ri_rpa/scsrpa_25d.rs），use_ri_symm=true 且 n_occ>=20 (α=21, β=20 occ) 时触发。
小配置: basis=3-21g; aux=def2-svpd-rifit; grid_generation_level=2; frequency_points=20 (默认)。
覆盖点: serial vs 25d mpi(n=2/3/4) 能量一致(<=1e-9)。
