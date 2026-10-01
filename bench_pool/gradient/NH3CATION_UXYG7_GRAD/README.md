# NH3CATION_UXYG7_GRAD

Unrestricted XYG7 analytic gradient of the distorted (C1, non-equilibrium) NH3+
doublet (`charge = 1`, `spin = 2` i.e. multiplicity 2 (2S = 1), `spin_polarization = true`) in
def2-SVP; `def2-universal-jkfit` is the auxiliary basis of both the DF-SCF and
the RI part, and the final functional fixes the SCF-iteration functional
(B3LYPg). The case runs in json-reference mode (auto-detected `reference.json`;
`config.toml` pins the required fields, see `docs/config-toml.md`), with
`job_type = "force"`, `use_ri_symm = true`, `auxbasis_response = true` and
`[ctrl.j2c_decomp] policy = "cd"` pinned. `num_mpi = 1` is pinned in
`tests.toml`, since the post-SCF analytic gradient is not MPI-parallelized.

The total gradient is the sum of the existing `"SCF"` gradient entry
(unrestricted B3LYPg term) and this branch's `"DH"` entry, the unrestricted
counterpart of the restricted decomposition: the PT2 correlation (OS and SS
spin channels) on the relaxed density with its hcore-derivative, overlap-Pulay,
JK and RI-derivative terms, the metric-derivative contribution and the SCF
(Z-vector) response, plus the final-functional difference terms — the
exchange-hybridization difference upon the SCF density, the skeleton XC
difference between the final and the SCF-iteration functional, and the
occupied-Pulay term of the final functional's energy-weighted density (the
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
    unit="Angstrom", basis="def2-SVP", charge=1, spin=1, verbose=0)
# DFDH rejects scf.HF/ROHF references: the reference SCF must be a dft.UKS object
# with density fitting and the XYG7 SCF functional (B3LYPg, pyscf alias "b3lyp").
mf = dft.UKS(mol, xc="B3LYPG").density_fit(auxbasis="def2-universal-jkfit")
mf.conv_tol, mf.conv_tol_grad = 1e-14, 1e-9
mf.kernel()
dh = DFDH(mf, "XYG7", auxbasis_jk="def2-universal-jkfit", auxbasis_ri="def2-universal-jkfit")
e = dh.kernel()
g = np.asarray(dh.nuc_grad_method().kernel())            # Ha/bohr, (natm, 3)
print(f"E(SCF)  = {mf.e_tot:.12f}")                      # -56.116153272815  REST -56.116153442167  diff 1.7e-7
print(f"E(PT2)  = {dh.eng_os + dh.eng_ss:.12f}")         #  -0.208587118018  REST  -0.208587126669  diff 8.7e-9
print(f"E(XYG7) = {e:.12f}")                             # -55.775680470092  REST -55.775680551671  diff 8.2e-8
print(np.array2string(g, precision=9))
# [[-0.009660425 -0.045791375 -0.102196316]
#  [ 0.011752364  0.019117794  0.062875605]
#  [-0.019403832  0.019093257  0.022739012]
#  [ 0.01731165   0.007580304  0.016582063]]
# pyscf-forge gradient (Ha/bohr); against the REST gradient max|Δ| = 4.4e-7 (largest at N,z)
```

| Quantity | REST | pyscf-forge | diff |
|---|---:|---:|---:|
| B3LYPg SCF energy (Ha) | -56.116153442167 | -56.116153272815 | 1.7e-7 |
| RI-PT2 correlation (Ha) | -0.208587126669 | -0.208587118018 | 8.7e-9 |
| XYG7 total energy (Ha) | -55.775680551671 | -55.775680470092 | 8.2e-8 |
| Gradient at N,z (largest \|Δ\|) (Ha/bohr) | -0.102196760219 | -0.102196315747 | 4.4e-7 |

As in the restricted case, the residual is dominated by the different DFT grids
of the two codes (the B3LYPg SCF, the final-functional XC part and REST's
dedicated coarser response grid `grid_level_cpscf` for the Z-vector response),
which shows up in the spin-density XC terms of the unrestricted solve. On top
of that sit the RI/JK and RI-PT2 conventions (REST's `cd` metric decomposition,
integral thresholds) and the different CP-SCF stagnation points. The gradient
agreement (4.4e-7 Ha/bohr) is at the expected cross-code floor for an
unrestricted double hybrid.
