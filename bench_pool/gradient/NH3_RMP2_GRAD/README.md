# NH3_RMP2_GRAD

RI-MP2 analytic gradient of distorted (C1, non-equilibrium) NH3 in def2-SVP;
`def2-universal-jkfit` is the auxiliary basis of both the DF-SCF and the RI-PT2
part, and `xc = "mp2"` makes the SCF a DF-HF. The case runs in json-reference
mode (auto-detected `reference.json`; `config.toml` pins the required fields,
see `docs/config-toml.md`), with `job_type = "force"`, `use_ri_symm = true`,
`auxbasis_response = true` and `[ctrl.j2c_decomp] policy = "cd"` pinned.
`num_mpi = 1` is pinned in `tests.toml`, since the post-SCF analytic gradient
is not MPI-parallelized.

The total gradient is the sum of the existing `"SCF"` gradient entry (the DF-HF
term here) and this branch's `"DH"` entry, which carries the PT2 correlation
into the relaxed density: the hcore-derivative and overlap-Pulay terms, the JK
and RI-derivative terms with the correlation weights, the metric-derivative
integral contribution and the SCF (Z-vector) response upon the relaxed
correlation density. For the pure-MP2 family the final-functional difference
terms vanish (no post-SCF DFA components, equal hybrid coefficients).
`reference.json` carries `scf_energy` (DF-HF), `energies/pt2` (total / OS / SS
correlation) and `energies/xdh_energy` (the total energy). Both the `cd`
(Cholesky) and the `eig` (eigen) J^(-1/2) decompositions of the RI metric are
supported by the gradient code; this case pins `cd`.

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
mf = dft.RKS(mol, xc="HF").density_fit(auxbasis="def2-universal-jkfit")   # DF-HF SCF (xc="mp2")
mf.conv_tol, mf.conv_tol_grad = 1e-14, 1e-9
mf.kernel()
dh = DFDH(mf, "MP2", auxbasis_jk="def2-universal-jkfit", auxbasis_ri="def2-universal-jkfit")
e = dh.kernel()
g = np.asarray(dh.nuc_grad_method().kernel())            # Ha/bohr, (natm, 3)
print(f"E(HF)  = {mf.e_tot:.12f}")                       # -56.133286460251  REST -56.133286460251  diff 5e-14
print(f"E(PT2) = {dh.eng_os + dh.eng_ss:.12f}")          #  -0.191795257798  REST  -0.191795257820  diff 2.3e-11
print(f"E(MP2) = {e:.12f}")                              # -56.325081718049  REST -56.325081718071  diff 2.3e-11
print(np.array2string(g, precision=9))
# [[ 0.019121636  0.005521114 -0.076449807]
#  [-0.001609882 -0.004340014  0.081319501]
#  [-0.004474798 -0.008007459  0.000716947]
#  [-0.013036956  0.006826359 -0.005586641]]
# pyscf-forge gradient (Ha/bohr); against the REST gradient max|Δ| = 1.4e-8 (largest at N,y)
```

| Quantity | REST | pyscf-forge | diff |
|---|---:|---:|---:|
| HF energy (Ha) | -56.13328646025118 | -56.13328646025113 | 5e-14 |
| RI-PT2 correlation (Ha) | -0.191795257820 | -0.191795257798 | 2.3e-11 |
| MP2 total energy (Ha) | -56.325081718071 | -56.325081718049 | 2.3e-11 |
| Gradient at N,y (largest \|Δ\|) (Ha/bohr) | 0.005521127385 | 0.005521113838 | 1.4e-8 |

The energies agree to ~1e-11 (the HF part to ~1e-13). The gradient residual
(1.4e-8 Ha/bohr, on components of the order of 0.08 Ha/bohr) comes from the
different RI/JK and RI-PT2 conventions of the two codes — REST's `cd` metric
decomposition of J^(-1/2) and its integral-derivative thresholds against
pyscf-forge's — and from the stagnation points of the two orbital-response
(CP-SCF) solvers. No DFT grid is involved in this case.
