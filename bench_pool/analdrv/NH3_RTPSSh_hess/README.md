# NH3_RTPSSh_hess

Analytical restricted TPSSh (meta-GGA hybrid, 10% exact exchange) Kohn-Sham Hessian
(`analdrv_tasks = ["hessian"]`) with RRHO thermochemistry at 298.15 K / 1 atm,
compared in json-reference mode (`config.toml`, see `docs/config-toml.md`).
`xc_parser = "parse_xc"` resolves the TPSSh functional through REST's parser.

## Cross-validation against Gaussian 16

Reference job: Gaussian 16 Rev. B.01, `freq symm=loose tpssh/def2svp`, same fixed
geometry (Angstrom) as `ctrl.in`. `symm=loose` lets g16 detect the C3v point group
(rotational symmetry number 3), matching REST's own point-group detection; with
default settings g16 finds C1 and its `G_corr` shifts by about `T·R·ln3 ≈ 2.7 kJ/mol`.
`symm=loose` also symmetrizes the near-C3v input geometry (deviation ~1e-5 Angstrom),
which is negligible here. g16 files are kept outside the repo.

REST uses RI-JK (`def2-universal-jkfit`) and its own DFT grids; g16 uses exact
4-center ERIs and its default (UltraFine) grid, so small systematic offsets are
expected, larger than in the RHF case.

| Quantity | REST | g16 | diff |
|---|---:|---:|---:|
| SCF energy (Ha) | -56.50670484 | -56.50668075 | -2.4e-05 |
| Freq. 1 (cm-1) | 844.756 | 845.133 | -0.38 |
| Freq. 2 (cm-1) | 1644.413 | 1644.698 | -0.28 |
| Freq. 3 (cm-1) | 1644.798 | 1644.699 | +0.10 |
| Freq. 4 (cm-1) | 3452.357 | 3453.265 | -0.91 |
| Freq. 5 (cm-1) | 3608.491 | 3609.421 | -0.93 |
| Freq. 6 (cm-1) | 3608.518 | 3609.421 | -0.90 |
| ZPE (J/mol) | 88543.59 | 88563.35 | -19.77 |
| U_corr(298K) (J/mol) | 96168.95 | 96187.80 | -18.85 |
| H_corr(298K) (J/mol) | 98647.91 | 98666.28 | -18.37 |
| G_corr(298K) (J/mol) | 41138.00 | 41159.96 | -21.95 |
| S_total (J/mol/K) | 192.889 | 192.882 | +0.007 |
| CV (J/mol/K) | 27.744 | 27.740 | +0.004 |

Frequencies agree within 1 cm-1 and thermochemical corrections within 25 J/mol
(< 0.025 kJ/mol). The g16 `Low frequencies` printout is the unprojected spectrum
(the fixed geometry is not exactly stationary); the tabulated `Frequencies --`
values are the projected true vibrations compared here.

## Gaussian input (reference)

Deck used for the g16 run quoted above (Gaussian 16 Rev. B.01).

```
%chk=nh3_rtpssh.chk
%mem=1GB
%nprocshared=4
#p freq symm=loose tpssh/def2svp

NH3 TPSSh/def2-svp freq (restricted), validation for rest_regression analdrv/NH3_RTPSSh_hess

0 1
N  -2.1988391019      1.8973746268      0.0000000000
H  -1.1788391019      1.8973746268      0.0000000000
H  -2.5388353987      1.0925460144     -0.5263586446
H  -2.5388400276      2.7556271745     -0.4338224694
```

