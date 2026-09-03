# NH3_RHF_hess

Analytical RHF Hessian (`analdrv_tasks = ["hessian"]`) with RRHO thermochemistry at
298.15 K / 1 atm, compared in json-reference mode (auto-detected `reference.json`;
`config.toml` pins the required fields, see `docs/config-toml.md`).

## Cross-validation against Gaussian 16

Reference job: Gaussian 16 Rev. B.01, `freq symm=loose rhf/def2svp`, same fixed
geometry (Angstrom) as `ctrl.in`. `symm=loose` lets g16 detect the C3v point group
(rotational symmetry number 3), matching REST's own point-group detection; with
default settings g16 finds C1 and its `G_corr` shifts by about `T·R·ln3 ≈ 2.7 kJ/mol`.
`symm=loose` also symmetrizes the near-C3v input geometry (deviation ~1e-5 Angstrom),
which is negligible here. g16 files are kept outside the repo.

REST uses RI-JK (`def2-universal-jkfit`), g16 uses exact 4-center ERIs, so a small
systematic offset is expected.

| Quantity | REST | g16 | diff |
|---|---:|---:|---:|
| SCF energy (Ha) | -56.14774032 | -56.14776201 | +2.2e-05 |
| Freq. 1 (cm-1) | 1080.766 | 1080.667 | +0.10 |
| Freq. 2 (cm-1) | 1790.620 | 1790.580 | +0.04 |
| Freq. 3 (cm-1) | 1790.623 | 1790.580 | +0.04 |
| Freq. 4 (cm-1) | 3520.248 | 3520.689 | -0.44 |
| Freq. 5 (cm-1) | 3665.769 | 3666.217 | -0.45 |
| Freq. 6 (cm-1) | 3665.770 | 3666.217 | -0.45 |
| ZPE (J/mol) | 92793.11 | 92800.91 | -7.80 |
| U_corr(298K) (J/mol) | 100308.17 | 100315.09 | -6.92 |
| H_corr(298K) (J/mol) | 102787.13 | 102793.56 | -6.43 |
| G_corr(298K) (J/mol) | 45417.34 | 45426.40 | -9.05 |
| S_total (J/mol/K) | 192.419 | 192.414 | +0.005 |
| CV (J/mol/K) | 26.405 | 26.405 | +0.000 |

Frequencies agree within 0.5 cm-1 and thermochemical corrections within 10 J/mol
(< 0.01 kJ/mol). The g16 `Low frequencies` printout is the unprojected spectrum
(the fixed geometry is not exactly stationary); the tabulated `Frequencies --`
values are the projected true vibrations compared here.

## Gaussian input (reference)

Deck used for the g16 run quoted above (Gaussian 16 Rev. B.01).

```
%chk=nh3_rhf.chk
%mem=1GB
%nprocshared=4
#p freq symm=loose hf/def2svp

NH3 RHF/def2-svp freq, validation for rest_regression analdrv/NH3_RHF_hess

0 1
N  -2.1988391019      1.8973746268      0.0000000000
H  -1.1788391019      1.8973746268      0.0000000000
H  -2.5388353987      1.0925460144     -0.5263586446
H  -2.5388400276      2.7556271745     -0.4338224694
```

