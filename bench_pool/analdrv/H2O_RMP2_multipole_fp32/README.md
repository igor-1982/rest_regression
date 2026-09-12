# H2O_RMP2_multipole_fp32

RI-MP2 electric multipole moments (`analdrv_tasks = ["multipole"]`, default
relaxed density: unrelaxed correlation rdm1 plus the Z-vector response
increment) with the PT2 correlation energy evaluated by the new pair-engine
driver in FP32 (`[ctrl.ri_pt2] new_driver = true`, `fp_mode = "FP32"`),
compared in json-reference mode (auto-detected `reference.json`; `config.toml`
pins the required fields, see `docs/config-toml.md`).

The FP32 mode applies to both the PT2 energy contraction and the multipole
density increments (all reported quantities remain f64; FP32 is the working
type of the incore electronic-derivative kernel only). The example covers the
combination of the
new driver (which retains `rimatr`) with the analdrv multipole task on a
PT2-family post-SCF method, with the full per-part decomposition
(nuc/scf/corr/resp/tot) of every order.

## Cross-validation against pyscf-forge

Relaxed total dipole (a.u.), same geometry/basis/auxbasis (RI and JK both
`def2-universal-jkfit`). Run with `PYSCF_EXT_PATH=<pyscf-forge checkout>`
(pyscf-forge provides `pyscf.dh`); the `atom_charges` float64 cast works
around an einsum dtype mismatch in `pyscf.dh.resp` (pyscf-forge + pyscf
2.14):

```python
import numpy as np
from pyscf import gto
from pyscf.dh import DFDH
mol = gto.M(atom="""O 0. 0. 0.11537521; H 0.75036929 0. -0.45898737; H -0.75036929 0. -0.45898737""", unit="Angstrom", basis="def2-tzvp", verbose=0)
_charges = mol.atom_charges
mol.atom_charges = lambda: np.asarray(_charges(), dtype=np.float64)
mf = DFDH(mol, "MP2", auxbasis_jk="def2-universal-jkfit", auxbasis_ri="def2-universal-jkfit").run()
print(mf.polar_method().dipole())
# [ 3.24e-15 -1.10e-15 -8.24096872e-01]  REST: [ 1.15e-13 -4.43e-15 -8.24097813e-01]  max diff 9.4e-7
```

| Quantity | REST | pyscf-forge | diff |
|---|---:|---:|---:|
| MP2 total energy (Ha) | -76.33064219 | -76.33064219 | 1.8e-8 |
| Dipole Z, corr (a.u.) | 0.008565302 | 0.008565313 | 1.2e-8 |
| Dipole Z, resp (a.u.) | 0.008076118 | 0.008076398 | 2.8e-7 |
| Dipole Z, total (a.u.) | -0.824097813 | -0.824096872 | 9.4e-7 |

REST evaluates the moments at the center of nuclear mass, pyscf at the
coordinate origin, so only the net-charge-zero parts (corr, resp) and the
total dipole (origin-invariant for the neutral molecule) are compared
directly. The residual difference sits at the known cross-code floor (~1e-6
a.u.): the Z-vector response part is limited by libcint-version/threshold
differences and the different CP-SCF stagnation points of the two solvers.
