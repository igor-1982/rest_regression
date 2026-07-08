# H2O BLYP 解析 Hessian 矩阵测试

- 泛函方法 blyp；
- 基组 def2-SVP，辅助基组 def2-SVP-RIFIT；
- 计算类型：解析 Hessian 矩阵。

体系：H2O 分子。

参考结果：
- Hessian total max_abs ≈ 7.0228e-1
- 与 PySCF 参考值差异 < 1e-4