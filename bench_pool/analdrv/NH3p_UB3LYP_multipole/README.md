# NH3p_UB3LYP_multipole

UKS B3LYP electric multipole moments of the NH3 cation (+1, doublet, C1
geometry; def2-TZVP / def2-universal-jkfit RI-JK) with pure default multipole
options (`analdrv_tasks = ["multipole"]`; no `[analdrv]` section: all orders
1-4, coordinate origin [0, 0, 0]), compared in json-reference mode
(auto-detected `reference.json`; `config.toml` pins the required fields, see
`docs/config-toml.md`).

The example covers the unrestricted SCF-level multipole machinery
(`UMultipoleDH`: per-spin MO contraction onto the total α+β density, per-part
decomposition nuc/scf/tot with no corr/resp entries) on a **charged** open
shell: the dipole is origin-dependent, so the default-origin convention is
part of what the reference pins. The C1 geometry keeps all off-diagonal
quadrupole components non-zero.

## Cross-validation against pyscf

Unrelaxed total moments at the coordinate origin (a.u.); pyscf 2.14.0 UKS
B3LYP with the same RI-JK setup (the charged-system translation laws
dip' = dip − Q·shift and quad' = quad − shift·dipᵀ − dip·shiftᵀ +
Q·shift·shiftᵀ were verified alongside):

```python
import numpy as np
from pyscf import gto, scf
mol = gto.M(atom="""N 0. 0. 0.; H 0.62 0.10 0.90; H -0.55 -0.25 0.90; H 0.02 0.70 -0.75""", unit="Angstrom", basis="def2-tzvp", charge=1, spin=1, verbose=0)
mf = scf.UKS(mol).density_fit(auxbasis="def2-universal-jkfit"); mf.xc = "b3lyp"
mf.conv_tol = 1e-12; mf.kernel()
dm = mf.make_rdm1()[0] + mf.make_rdm1()[1]
print(np.einsum("A,At->t", mol.atom_charges(), mol.atom_coords()) - np.einsum("tuv,uv->t", mol.intor("int1e_r"), dm))
# [0.062582058 0.353193023 0.637093486]  REST: [0.062581752 0.353192254 0.637093871]
```

| Quantity | REST | pyscf | diff |
|---|---:|---:|---:|
| UKS B3LYP energy (Ha) | -56.134700236 | -56.134700136 | 1.0e-7 |
| Dipole X/Y/Z, tot (a.u.) | 0.062581752, 0.353192254, 0.637093871 | 0.062582058, 0.353193023, 0.637093486 | 7.7e-7 |
| Quadrupole XY, raw (a.u.) | 0.175791106 | 0.175792850 | 1.7e-6 |

## Cross-validation against Gaussian 16

Gaussian 16 B.01 run of the same system with exact ERIs (a trailing blank
line after the geometry is required):

```
%chk=nh3p.chk
%mem=2GB
%nprocshared=1
#p UB3LYP/def2TZVP NoSymm Int=UltraFine

NH3+ UKS B3LYP double validation

1 2
N 0.0 0.0 0.0
H 0.62 0.10 0.90
H -0.55 -0.25 0.90
H 0.02 0.70 -0.75

```

Converting the printed "Dipole moment" section (Debye; 1 a.u. = 2.54174623 D)
and the raw "Quadrupole moment" section (Debye-Ang; 1 a.u. = 2.54174623 ×
0.529177210903 Debye-Ang), max diff vs the pinned references is 5.1e-5 a.u.
(dipole; 4-decimal print rounding + grid) and 1.8e-4 a.u. (raw quadrupole;
+ RI-JK fitting). Conventions confirmed for the charged case: multipoles
printed at the coordinate origin, raw second-moment quadrupole, and G16's
"Traceless Quadrupole" section carries no 3/2 factor.
