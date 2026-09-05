# NH3_R-xDH7_SCC15: 闭壳层 R-xDH7 + SCC15 的 2.5D MPI 验证用例
被测功能: 闭壳层（RHF）SCSRPA 2.5D 关联能 + SCC15 MPI 路径（sbge2_25d detailed 闭壳层内核，
含 eij_01 全矩形镜像语义——该路径曾被闭壳层下三角缺失 bug 影响，本用例锁定修复）。
小配置: cc-pVDZ / def2-SV(P)-JKFIT（本地目录），frequency_points=40，post_ai_correction="scc15"。
覆盖点: serial vs 25d mpi(n=2) 总能一致(<=1e-8)，SCC_1/SCC_R/SCC_Limit 分量一致。
