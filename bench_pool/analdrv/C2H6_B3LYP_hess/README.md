# C2H6_B3LYP_hess

Analytical restricted B3LYP Hessian (`analdrv_tasks = ["hessian"]`) of ethane at
a fixed hand-built D3d (staggered) geometry, with RRHO thermochemistry at
298.15 K / 1 atm, compared in json-reference mode (auto-detected
`reference.json`; `config.toml` pins the required fields, see
`docs/config-toml.md`). `xc_parser = "parse_xc"` so that this case and
`C2H6_B3LYP_D3BJ_hess` differ only by the dispersion suffix of `xc`.

Ethane is 8 atoms, slightly above the usual <=6-atom guideline: it is the
smallest covalent molecule whose harmonic frequencies respond visibly, and with
both signs, to an empirical dispersion correction (the methyl torsion is
stiffened, the C-H stretches are softened; see `C2H6_B3LYP_D3BJ_hess`).

## Cross-validation against Gaussian 16

Reference job: Gaussian 16 Rev. B.01, `freq b3lyp/def2svp`, same fixed geometry
(Angstrom) as `ctrl.in`. REST uses RI-JK (`def2-universal-jkfit`) and its own
DFT grids; g16 uses exact 4-center ERIs and its default grid, so a small
systematic offset is expected.

| Quantity | REST | g16 | diff |
|---|---:|---:|---:|
| SCF energy (Ha) | -79.77130222 | -79.77125274 | -4.9e-05 |
| Freq. 1, torsion (cm-1) | 175.942 | 176.259 | -0.32 |
| Freq. 2 (cm-1) | 778.698 | 778.738 | -0.04 |
| Freq. 3 (cm-1) | 779.109 | 778.987 | +0.12 |
| Freq. 4 (cm-1) | 997.093 | 997.451 | -0.36 |
| Freq. 5 (cm-1) | 1185.000 | 1185.488 | -0.49 |
| Freq. 6 (cm-1) | 1185.278 | 1185.528 | -0.25 |
| Freq. 7 (cm-1) | 1361.281 | 1361.413 | -0.13 |
| Freq. 8 (cm-1) | 1384.967 | 1384.864 | +0.10 |
| Freq. 9 (cm-1) | 1465.231 | 1465.415 | -0.18 |
| Freq. 10 (cm-1) | 1465.689 | 1465.447 | +0.24 |
| Freq. 11 (cm-1) | 1466.713 | 1466.980 | -0.27 |
| Freq. 12 (cm-1) | 1467.068 | 1466.986 | +0.08 |
| Freq. 13 (cm-1) | 3103.350 | 3104.466 | -1.12 |
| Freq. 14 (cm-1) | 3104.360 | 3105.344 | -0.98 |
| Freq. 15 (cm-1) | 3181.149 | 3182.154 | -1.01 |
| Freq. 16 (cm-1) | 3181.600 | 3182.585 | -0.99 |
| Freq. 17 (cm-1) | 3201.235 | 3202.149 | -0.91 |
| Freq. 18 (cm-1) | 3201.644 | 3202.575 | -0.93 |
| ZPE (J/mol) | 195502.18 | 195553.7 | -51.5 |

Frequencies agree within 1.2 cm-1 (RMS 0.6 cm-1), the C-H stretches showing the
largest offset, and ZPE within 52 J/mol, both consistent with the RI-JK/grid
differences. The g16 `Low frequencies` printout is the unprojected spectrum
(the fixed geometry is not exactly stationary); the tabulated `Frequencies --`
values are the projected true vibrations compared here.

## Gaussian input (reference)

Deck used for the g16 run quoted above (Gaussian 16 Rev. B.01).

```
%chk=ethane_b3lyp.chk
%mem=2GB
%nprocshared=8
#p freq b3lyp/def2svp

ethane B3LYP/def2SVP freq (no dispersion), validation for rest_regression analdrv/C2H6_B3LYP_hess

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
