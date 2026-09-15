# H2O_XYG3_multipole

XYG3 electric multipole moments with pure default options
(`analdrv_tasks = ["multipole"]`; no `[analdrv]` section: all orders 1-4,
center-of-nuclear-mass origin, relaxed density — unrelaxed correlation rdm1
plus the Z-vector response increment), compared in json-reference mode
(auto-detected `reference.json`; `config.toml` pins the required fields, see
`docs/config-toml.md`).

The example covers the full fifth-DFA (double-hybrid) multipole machinery on
the NCMC XYG3 functional: the SCF part from the non-self-consistent B3LYP
density, the RI-PT2 correlation increment, and the relaxed Z-vector increment
solved against the SCF-iteration functional, with the per-part decomposition
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
mf = DFDH(mol, "XYG3", auxbasis_jk="def2-universal-jkfit", auxbasis_ri="def2-universal-jkfit").run()
print(mf.polar_method().dipole())
# [ 3.62e-15 -8.13e-16 -8.22217491e-01]  REST: [ 1.13e-13 -4.74e-15 -8.22217215e-01]  max diff 2.8e-7
```

| Quantity | REST | pyscf-forge | diff |
|---|---:|---:|---:|
| XYG3 total energy (Ha) | -76.42320926 | -76.42320865 | 6.1e-7 |
| Dipole Z, corr (a.u.) | 0.008929100 | 0.008929099 | 1e-9 |
| Dipole Z, resp (a.u.) | -0.022015992 | -0.022017037 | 1.0e-6 |
| Dipole Z, total (a.u.) | -0.822217215 | -0.822217491 | 2.8e-7 |

REST evaluates the moments at the center of nuclear mass, pyscf at the
coordinate origin, so only the net-charge-zero parts (corr, resp) and the
total dipole (origin-invariant for the neutral molecule) are compared
directly. The energy and response-part differences reflect the different DFT
grids and CP-SCF stagnation points of the two codes.
