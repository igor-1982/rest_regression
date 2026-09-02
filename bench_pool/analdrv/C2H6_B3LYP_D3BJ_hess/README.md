# C2H6_B3LYP_D3BJ_hess

Analytical restricted B3LYP-D3(BJ) Hessian (`analdrv_tasks = ["hessian"]`) of
ethane at a fixed hand-built D3d (staggered) geometry, with RRHO
thermochemistry at 298.15 K / 1 atm, compared in json-reference mode
(auto-detected `reference.json`; `config.toml` pins the required fields, see
`docs/config-toml.md`). The dispersion suffix `-d3bj` in `xc` requires
`xc_parser = "parse_xc"`.

The empirical dispersion correction enters the Hessian as a nuclear-like term
(zeroth order in the density matrix, no CP-KS response). The dftd libraries
provide no analytical Hessian, so REST evaluates this contribution numerically
by central finite differences of the **analytic** dispersion gradient
(`analdrv` `dftd_hess_step`, default 3e-4 Bohr); all other contributions
(RI-JK, DFT skeleton, overlap) remain analytical.

## Cross-validation against Gaussian 16

Reference job: Gaussian 16 Rev. B.01,
`freq b3lyp/def2svp empiricaldispersion=GD3BJ`, same fixed geometry (Angstrom)
as `ctrl.in`. REST uses RI-JK and its own DFT grids; g16 uses exact 4-center
ERIs and its default grid, so a small systematic offset is expected.

| Quantity | REST | g16 | diff |
|---|---:|---:|---:|
| SCF energy (Ha) | -79.77671370 | -79.77666519 | -4.9e-05 |
| D3(BJ) energy (Ha) | -0.00541148 | -0.00541245 | +1.0e-06 |
| Freq. 1, torsion (cm-1) | 178.437 | 178.736 | -0.30 |
| Freq. 2 (cm-1) | 779.674 | 779.712 | -0.04 |
| Freq. 3 (cm-1) | 780.085 | 779.961 | +0.12 |
| Freq. 4 (cm-1) | 997.584 | 997.941 | -0.36 |
| Freq. 5 (cm-1) | 1185.300 | 1185.789 | -0.49 |
| Freq. 6 (cm-1) | 1185.579 | 1185.828 | -0.25 |
| Freq. 7 (cm-1) | 1361.514 | 1361.645 | -0.13 |
| Freq. 8 (cm-1) | 1385.448 | 1385.345 | +0.10 |
| Freq. 9 (cm-1) | 1465.674 | 1465.857 | -0.18 |
| Freq. 10 (cm-1) | 1466.132 | 1465.889 | +0.24 |
| Freq. 11 (cm-1) | 1466.985 | 1467.250 | -0.27 |
| Freq. 12 (cm-1) | 1467.340 | 1467.256 | +0.08 |
| Freq. 13 (cm-1) | 3103.170 | 3104.285 | -1.11 |
| Freq. 14 (cm-1) | 3104.132 | 3105.114 | -0.98 |
| Freq. 15 (cm-1) | 3180.830 | 3181.835 | -1.01 |
| Freq. 16 (cm-1) | 3181.281 | 3182.266 | -0.99 |
| Freq. 17 (cm-1) | 3200.992 | 3201.905 | -0.91 |
| Freq. 18 (cm-1) | 3201.402 | 3202.332 | -0.93 |
| ZPE (J/mol) | 195538.97 | 195590.6 | -51.6 |

Frequencies agree within 1.2 cm-1 and the D3(BJ) energy to 1e-6 Ha
(Bohr-conversion level). The g16 `Low frequencies` printout is the unprojected
spectrum (the fixed geometry is not exactly stationary); the tabulated
`Frequencies --` values are the projected true vibrations compared here.

## Effect of the D3(BJ) correction (vs plain B3LYP)

Same geometry and basis; the B3LYP reference values are from an identical
calculation with `xc = "b3lyp"` (no dispersion suffix).
Dispersion stabilizes the staggered conformation, so the methyl torsion is
**stiffened** by about +2.5 cm-1, while the C-H stretches are slightly
**softened** (about -0.2 to -0.3 cm-1); the skeletal/CH bending modes move by
less than +1 cm-1. The ZPE rises by about +37 J/mol.

| Mode | B3LYP | B3LYP-D3BJ | shift |
|---|---:|---:|---:|
| Torsion | 175.942 | 178.437 | +2.49 |
| CH3 rock / skeletal (780-1185, 5 modes) | 778.698 - 1185.278 | 779.674 - 1185.579 | +0.30 ... +0.98 |
| CH3 deformation (1361-1467, 6 modes) | 1361.281 - 1467.068 | 1361.514 - 1467.340 | +0.23 ... +0.48 |
| CH stretch (3103-3202, 6 modes) | 3103.350 - 3201.644 | 3103.170 - 3201.402 | -0.18 ... -0.32 |
| ZPE (J/mol) | 195502.18 | 195538.97 | +36.8 |
| SCF energy (Ha) | -79.77130222 | -79.77671370 | -0.00541148 (= E_D3BJ) |

The same comparison in g16 (`freq b3lyp/def2svp` +/- `empiricaldispersion=GD3BJ`)
gives the same shifts to within 0.02 cm-1 (torsion) and 0.003 cm-1 (all other
modes): taking the with/without difference cancels the shared electronic
Hessian, so this is essentially a direct check of REST's numerical D3(BJ)
Hessian against the analytic one in g16.

## Gaussian input (reference)

Deck used for the g16 run quoted above (Gaussian 16 Rev. B.01).

```
%chk=ethane_b3lyp_d3bj.chk
%mem=2GB
%nprocshared=8
#p freq b3lyp/def2svp empiricaldispersion=GD3BJ

ethane B3LYP-D3BJ/def2SVP freq, validation for rest_regression analdrv/C2H6_B3LYP_D3BJ_hess

0 1
C    0.00000000    0.00000000    0.76750000
C    0.00000000    0.00000000   -0.76750000
H    1.02040000    0.00000000    1.15920000
H   -0.51020000    0.88378700    1.15920000
H   -0.51020000   -0.88378700    1.15920000
H    0.51020000    0.88378700   -1.15920000
H   -1.02040000    0.00000000   -1.15920000
H    0.51020000   -0.88378700   -1.15920000
```
