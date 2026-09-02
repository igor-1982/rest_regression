# NH3_UB3LYP_hess

Analytical unrestricted B3LYP (UKS) Hessian (`analdrv_tasks = ["hessian"]`) of the
NH3+ cation (charge +1, spin multiplicity 2, doublet) with RRHO thermochemistry at
298.15 K / 1 atm, compared in json-reference mode (auto-detected `reference.json`;
`config.toml` pins the required fields, see `docs/config-toml.md`).

## Cross-validation against Gaussian 16

Reference job: Gaussian 16 Rev. B.01, `freq symm=loose ub3lyp/def2svp`, charge 1,
multiplicity 2, same fixed geometry (Angstrom) as `ctrl.in`. `symm=loose` lets g16
detect the C3v point group (rotational symmetry number 3), matching REST's own
point-group detection; with default settings g16 finds C1 and its `G_corr` shifts by
about `T·R·ln3 ≈ 2.7 kJ/mol`. `symm=loose` also symmetrizes the near-C3v input
geometry (deviation ~1e-5 Angstrom), which is negligible here. g16 files are kept
outside the repo.

REST uses RI-JK (`def2-universal-jkfit`) and its own DFT grids; g16 uses exact
4-center ERIs and its default (UltraFine) grid, so small systematic offsets are
expected.

| Quantity | REST | g16 | diff |
|---|---:|---:|---:|
| SCF energy (Ha) | -56.12503571 | -56.12501384 | -2.2e-05 |
| Freq. 1 (cm-1) | 1207.505 | 1207.710 | -0.20 |
| Freq. 2 (cm-1) | 1437.652 | 1437.722 | -0.07 |
| Freq. 3 (cm-1) | 1437.947 | 1437.722 | +0.23 |
| Freq. 4 (cm-1) | 3523.100 | 3523.868 | -0.77 |
| Freq. 5 (cm-1) | 3635.200 | 3635.962 | -0.76 |
| Freq. 6 (cm-1) | 3635.210 | 3635.963 | -0.75 |
| ZPE (J/mol) | 88981.92 | 88996.56 | -14.64 |
| U_corr(298K) (J/mol) | 96494.88 | 96508.12 | -13.23 |
| H_corr(298K) (J/mol) | 98973.84 | 98986.59 | -12.75 |
| G_corr(298K) (J/mol) | 39890.12 | 39904.97 | -14.85 |
| S_total (J/mol/K) | 198.168 | 198.163 | +0.005 |
| CV (J/mol/K) | 26.558 | 26.560 | -0.002 |

Both codes include the doublet electronic contribution (`S_ele = R·ln2 = 5.763`
J/mol/K). Frequencies agree within 1 cm-1 and thermochemical corrections within
15 J/mol (< 0.015 kJ/mol). The g16 `Low frequencies` printout is the unprojected
spectrum (the neutral NH3 test geometry is far from the cation's stationary point);
the tabulated `Frequencies --` values are the projected true vibrations compared
here.

## Gaussian input (reference)

Deck used for the g16 run quoted above (Gaussian 16 Rev. B.01).

```
%chk=nh3_ub3lyp.chk
%mem=1GB
%nprocshared=4
#p freq symm=loose ub3lyp/def2svp

NH3+ UB3LYP/def2-svp freq (charge +1, doublet), validation for rest_regression analdrv/NH3_UB3LYP_hess

1 2
N  -2.1988391019      1.8973746268      0.0000000000
H  -1.1788391019      1.8973746268      0.0000000000
H  -2.5388353987      1.0925460144     -0.5263586446
H  -2.5388400276      2.7556271745     -0.4338224694
```

