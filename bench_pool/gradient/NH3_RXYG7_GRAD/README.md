# NH3_RXYG7_GRAD

Restricted XYG7 analytic gradient of distorted (C1, non-equilibrium) NH3 in
def2-SVP; `def2-universal-jkfit` is the auxiliary basis of both the DF-SCF and
the RI part, and the final functional fixes the SCF-iteration functional
(B3LYPg). The case runs in json-reference mode (auto-detected `reference.json`;
`config.toml` pins the required fields, see `docs/config-toml.md`), with
`job_type = "force"`, `use_ri_symm = true`, `auxbasis_response = true` and
`[ctrl.j2c_decomp] policy = "cd"` pinned. `num_mpi = 1` is pinned in
`tests.toml`, since the post-SCF analytic gradient is not MPI-parallelized.

The total gradient is the sum of the existing `"SCF"` gradient entry (the
B3LYPg Kohn-Sham term) and this branch's `"DH"` entry: the relaxed-density
hcore-derivative, overlap-Pulay, JK and RI-derivative terms with the PT2
correlation, the metric-derivative contribution and the SCF (Z-vector) response
upon the relaxed correlation density, plus the final-functional difference
terms — the exchange-hybridization difference upon the SCF density, the
skeleton XC difference between the final and the SCF-iteration functional, and
the occupied-Pulay term of the final functional's energy-weighted density (the
final functional's Brillouin-violation term enters the Z-vector Lagrangian).
`reference.json` carries `scf_energy` (DF-B3LYPg), `energies/pt2` (total / OS /
SS correlation, before the final functional's 0.4052 / 0.2589 factors) and
`energies/xdh_energy` (the total energy). Both the `cd` (Cholesky) and the
`eig` (eigen) J^(-1/2) decompositions of the RI metric are supported by the
gradient code; this case pins `cd`.

## Cross-validation against pyscf-forge

Same geometry/basis/auxbasis (RI and JK both `def2-universal-jkfit`). Run with
`PYSCF_EXT_PATH=<pyscf-forge checkout>` (pyscf-forge provides `pyscf.dh`):

```python
import numpy as np
from pyscf import gto, dft
from pyscf.dh import DFDH

mol = gto.M(atom="""N 0.0000000000 0.0000000000 0.0000000000
H 0.0000000000 0.0000000000 1.1500000000
H 0.9674007198 0.0000000000 -0.2902341250
H -0.4917018117 0.8516525201 -0.3062961203""",
    unit="Angstrom", basis="def2-SVP", verbose=0)
mf = dft.RKS(mol, xc="B3LYPG").density_fit(auxbasis="def2-universal-jkfit")   # DF-B3LYPg SCF
mf.conv_tol, mf.conv_tol_grad = 1e-14, 1e-9
mf.kernel()
dh = DFDH(mf, "XYG7", auxbasis_jk="def2-universal-jkfit", auxbasis_ri="def2-universal-jkfit")
e = dh.kernel()
g = np.asarray(dh.nuc_grad_method().kernel())            # Ha/bohr, (natm, 3)
print(f"E(SCF)  = {mf.e_tot:.12f}")                      # -56.497928052344  REST -56.497928360262  diff 3.1e-7
print(f"E(PT2)  = {dh.eng_os + dh.eng_ss:.12f}")         #  -0.271041040112  REST  -0.271041066006  diff 2.6e-8
print(f"E(XYG7) = {e:.12f}")                             # -56.151313321555  REST -56.151313441983  diff 1.2e-7
print(np.array2string(g, precision=9))
# [[ 0.020145095  0.007159664 -0.075638746]
#  [-0.001273626 -0.003784027  0.077399273]
#  [-0.008250278 -0.007337033  0.002273714]
#  [-0.010621345  0.003961456 -0.004033902]]
# pyscf-forge gradient (Ha/bohr); against the REST gradient max|Δ| = 2.6e-7 (largest at H3,x)
```

| Quantity | REST | pyscf-forge | diff |
|---|---:|---:|---:|
| B3LYPg SCF energy (Ha) | -56.497928360262 | -56.497928052344 | 3.1e-7 |
| RI-PT2 correlation (Ha) | -0.271041066006 | -0.271041040112 | 2.6e-8 |
| XYG7 total energy (Ha) | -56.151313441983 | -56.151313321555 | 1.2e-7 |
| Gradient at H3,x (largest \|Δ\|) (Ha/bohr) | -0.010621601703 | -0.010621344646 | 2.6e-7 |

The residual is dominated by the different DFT grids of the two codes: the
B3LYPg SCF and the final-functional XC parts are evaluated on REST's default
SCF grid against pyscf-forge's own quadrature, and REST evaluates the Z-vector
response XC part on its dedicated coarser response grid (`grid_level_cpscf`) by
default; the SCF and total energy differences (3.1e-7 and 1.2e-7) propagate
into the gradient. On top of that sit the RI/JK and RI-PT2 conventions (REST's
`cd` metric decomposition, integral thresholds) and the different CP-SCF
stagnation points. The gradient agreement (2.6e-7 Ha/bohr) is at the expected
cross-code floor for a double hybrid.
