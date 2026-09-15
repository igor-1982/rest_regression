# H2O_RHF_multipole_hess

RHF electric multipole moments plus analytical Hessian in one `analdrv` job
(`analdrv_tasks = ["multipole", "hessian"]`, default multipole options: all
orders 1-4, center-of-nuclear-mass origin), with RRHO thermochemistry at
298.15 K / 1 atm, compared in json-reference mode (auto-detected
`reference.json`; `config.toml` pins the required fields, see
`docs/config-toml.md`). The geometry is a stationary DF-RHF/def2-SVP structure
(pyscf/geometric optimization, max grad 4e-6), so the projected vibrational
frequencies are clean.

The single job exercises the shared response object between the two tasks (the
hessian always builds it; the SCF-only multipole ignores it), the task-order
independence, and the full multipole decomposition (nuc/scf/tot; no corr/resp
entries for SCF methods).

## Cross-validation against pyscf

Total dipole (a.u.), same geometry/basis/auxbasis (RI-JK `def2-universal-jkfit`,
pyscf 2.14):

```python
import numpy as np
from pyscf import gto, scf
mol = gto.M(atom="""O 0. 0. 0.11537521; H 0.75036929 0. -0.45898737; H -0.75036929 0. -0.45898737""", unit="Angstrom", basis="def2-svp", verbose=0)
mf = scf.RHF(mol).density_fit(auxbasis="def2-universal-jkfit")
mf.conv_tol = 1e-12
mf.kernel()
h = -mol.intor("int1e_r")
print(np.einsum("A,At->t", mol.atom_charges(), mol.atom_coords()) + np.einsum("tuv,uv->t", h, mf.make_rdm1()))
# [ 1.41e-15 -3.55e-16 -8.30077004e-01]  REST: [ 2.22e-13 -2.71e-15 -8.30076977e-01]  max diff 2.8e-8
```

| Quantity | REST | pyscf | diff |
|---|---:|---:|---:|
| SCF energy (Ha) | -75.96128289 | -75.96128289 | 1.0e-11 |
| Dipole Z (a.u.) | -0.830076977 | -0.830077004 | 2.8e-8 |
| Quadrupole XX, raw (a.u.) | -3.083257432 | -3.083257363 | 7.0e-8 |
| Quadrupole ZZ, raw (a.u.) | -4.259382953 | -4.259382922 | 3.1e-8 |
| Quadrupole XX, traceless (a.u.) | 1.653391299 | 1.653391354 | 5.4e-8 |

The quadrupole rows are compared at REST's echoed center-of-mass origin
[0, 0, 0.096565314] (Bohr).

Vibrational frequencies (projected, cm-1): 1750.626, 4148.565, 4244.686
(6 TR modes < 0.08); ZPE 60673.86 J/mol.
