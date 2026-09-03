# NH3_CAM-B3LYP_hess

Analytical restricted CAM-B3LYP (RKS) Hessian (`analdrv_tasks = ["hessian"]`) of NH3
at the fixed near-C3v geometry shared with the other NH3 analdrv tests, with RRHO
thermochemistry at 298.15 K / 1 atm, compared in json-reference mode (auto-detected
`reference.json`; `config.toml` pins the required fields, see `docs/config-toml.md`).

CAM-B3LYP is a range-separated hybrid. The exact exchange is decomposed as
`K = alpha * K_full - (alpha - hyb) * K_erfc` (long-range `alpha` = 0.65,
short-range `hyb` = 0.19, `omega` = 0.33), mirroring the SCF Fock assembly. The
Hessian adds the short-range correction as a second RI-JK Hessian object evaluated
on erfc(omega r) integrals (the incore `rimatr_sr`), reusing the same RI-JK
implementation as the full-range exchange; the Coulomb part remains full-range only.

## Cross-validation against Gaussian 16

Reference job: Gaussian 16 Rev. B.01, `freq symm=loose cam-b3lyp/def2svp`, charge 0,
multiplicity 1, same fixed geometry (Angstrom) as `ctrl.in`. `symm=loose` lets g16
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
| SCF energy (Ha) | -56.47769738 | -56.47768138 | -1.6e-05 |
| Freq. 1, umbrella (cm-1) | 864.082 | 864.324 | -0.24 |
| Freq. 2 (cm-1) | 1636.235 | 1636.422 | -0.19 |
| Freq. 3 (cm-1) | 1636.446 | 1636.422 | +0.02 |
| Freq. 4, NH stretch (cm-1) | 3473.787 | 3474.600 | -0.81 |
| Freq. 5 (cm-1) | 3622.105 | 3622.929 | -0.82 |
| Freq. 6 (cm-1) | 3622.114 | 3622.929 | -0.82 |
| ZPE (J/mol) | 88851.25 | 88867.91 | -16.66 |
| U_corr(298K) (J/mol) | 96464.96 | 96481.86 | -16.90 |
| H_corr(298K) (J/mol) | 98943.92 | 98960.33 | -16.41 |
| G_corr(298K) (J/mol) | 41449.40 | 41467.14 | -17.74 |
| S_total (J/mol/K) | 192.838 | 192.832 | +0.005 |
| CV (J/mol/K) | 27.635 | 27.631 | +0.004 |

Frequencies agree within 1 cm-1 and thermochemical corrections within 18 J/mol
(< 0.02 kJ/mol). The g16 `Low frequencies` printout is the unprojected spectrum
(the fixed geometry is not exactly stationary); the tabulated `Frequencies --`
values are the projected true vibrations compared here.

## Gaussian input (reference)

Deck used for the g16 run quoted above (Gaussian 16 Rev. B.01).

```
%chk=nh3_camb3lyp.chk
%mem=2GB
%nprocshared=8
#p freq symm=loose cam-b3lyp/def2svp

NH3 CAM-B3LYP/def2SVP freq (charge 0, singlet), validation for rest_regression analdrv/NH3_CAM-B3LYP_hess

0 1
N  -2.1988391019      1.8973746268      0.0000000000
H  -1.1788391019      1.8973746268      0.0000000000
H  -2.5388353987      1.0925460144     -0.5263586446
H  -2.5388400276      2.7556271745     -0.4338224694
```
