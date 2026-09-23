# H2O10_PBE0_pair_prune_on

## 1. 被测功能

存储级（storage-level）AO 对剪枝的**打开**路径：(H2O)10 水簇、PBE0/def2-SVP +
def2-universal-jkfit，相对阈值 1.0e-6。

剪枝在 `SCF::prepare_necessary_integrals` 内完成：先算每个 AO 对的静态行界
`q_p = max_P |R[p, P]|`（MPI 下先做一次长度 `nbaspar` 的 max 归约再加广播），丢掉
`q_p <= tol * max_p q_p` 的行，把保留行**原地**压紧（不额外分配第二份张量），并把
"存储行 -> (mu, nu)" 与保留行界记进 `rimatr_pair_map`。J/K 随后按该 map 取行；J 的输出仍散射回
**全量** packed 上三角，被丢的行保持 0，调用方继续用未改动的 `basbas2baspar` 组装 Fock 矩阵。

## 2. 触发条件

```toml
[ctrl.ri_jk]
pair_screen_threshold = 1.0e-6
```

日志证据（本用例的 reference.log 中可查）：

```
In-core RI pair screening enabled (threshold 1.0e-6, 28920 AO pairs).
Storage-level AO-pair pruning: 10235 of 28920 AO-pair rows stored (35.39%), the full-space pair tables are unchanged.
```

即 28920 个 AO 对里丢掉 65% 的行；缩并阶段 J 只用 4900 个、K 用 9106 个存储行
（`pair-screened RI-J/K` 调试行）。

## 3. 覆盖点

- **剪枝确实生效**：65% 的行被丢掉，访问的存储空间相应下降。
- **紧凑行布局的取数**：J 的权重按存储行取全密度 `dm[[mu, nu]]`（对角 1 倍、非对角 2 倍），
  K 经 `map.pair_of` 取活跃 AO 子集。若 map 用错（例如仍按全量号索引），能量会明显偏离。
- **J 的全量散射**：被丢的行补 0、保留行写回 `basbas2baspar[[mu, nu]]`。漏掉或错位这一步，
  Fock 组装会错位，能量会大幅偏移。
- **数值中性**：1.0e-6 下存储剪枝保留的行集合是使用期筛选保留集合的超集，所以本用例的参考能量
  `-762.8486075242` Ha 与改动前"只用使用期筛选"的同一算例逐位相同。这条不变量同时锁住
  "存储级剪枝在这个阈值上不引入任何数值变化"。
- **MPI 一致性**：本用例在 `validate.sh --mpi`（2 进程）下跑，静态行界与每轮筛选尺度都必须
  全局化，否则两个 rank 保留不同行集合，数据会缺项。

## 4. 对照

同一体系、同一方法下的实测（PBE0/def2-SVP，能量单位 Ha）：

| `pair_screen_threshold` | 存储保留行 | 参考能量 | 相对未筛选的偏差 |
| --- | --- | --- | --- |
| 0.0（未筛选） | 28920 / 28920 | -762.8544995784 | 0 |
| 1.0e-6（本用例） | 10235 / 28920 | -762.8486075242 | +5.9e-03 |
| 1.0e-4 | 6984 / 28920 | -763.4181397957 | -5.6e-01（收敛到别的解） |

1.0e-4 一行是"存储保留集合不再包含使用期筛选集合"的对照：那一档的存储级剪枝会真正删掉
使用期筛选本来还想保留的少数行，能量相对改动前的使用期筛选版移动 7.2e-04 Ha。本用例取 1.0e-6
是为了让两个筛选层级互为超集，从而把"重接线后数值中性"这条不变量固定在参考值上。

未筛选对照见 `H2O_PBE0_pair_screen_off`（H2O 体系）。
