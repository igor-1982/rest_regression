# NH3_UCAM-B3LYP_hess

Analytical unrestricted CAM-B3LYP (UKS) Hessian (`analdrv_tasks = ["hessian"]`) of the
NH3+ cation (charge +1, spin multiplicity 2, doublet) with RRHO thermochemistry at
298.15 K / 1 atm, compared in json-reference mode (auto-detected `reference.json`;
`config.toml` pins the required fields, see `docs/config-toml.md`).

CAM-B3LYP is a range-separated hybrid. The exact exchange is decomposed as
`K = alpha * K_full - (alpha - hyb) * K_erfc` (long-range `alpha` = 0.65,
short-range `hyb` = 0.19, `omega` = 0.33), mirroring the SCF Fock assembly. The
Hessian adds the short-range correction as a second RI-JK Hessian object evaluated
on erfc(omega r) integrals (the incore `rimatr_sr`), reusing the same RI-JK
implementation as the full-range exchange; the Coulomb part remains full-range only.

## Cross-validation against Gaussian 16

Reference job: Gaussian 16 Rev. B.01, `freq symm=loose cam-b3lyp/def2svp`, charge 1,
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
| SCF energy (Ha) | -56.09443579 | -56.09441384 | -2.2e-05 |
| Freq. 1 (cm-1) | 1210.126 | 1210.292 | -0.17 |
| Freq. 2 (cm-1) | 1438.289 | 1438.369 | -0.08 |
| Freq. 3 (cm-1) | 1438.531 | 1438.369 | +0.16 |
| Freq. 4, NH stretch (cm-1) | 3537.122 | 3537.886 | -0.76 |
| Freq. 5 (cm-1) | 3647.354 | 3648.126 | -0.77 |
| Freq. 6 (cm-1) | 3647.358 | 3648.126 | -0.77 |
| ZPE (J/mol) | 89234.12 | 89248.61 | -14.49 |
| U_corr(298K) (J/mol) | 96746.55 | 96760.16 | -13.61 |
| H_corr(298K) (J/mol) | 99225.51 | 99238.64 | -13.13 |
| G_corr(298K) (J/mol) | 40142.43 | 40159.64 | -17.21 |
| S_total (J/mol/K) | 198.166 | 198.158 | +0.007 |
| CV (J/mol/K) | 26.550 | 26.547 | +0.003 |

Both codes include the doublet electronic contribution (`S_ele = R·ln2 = 5.763`
J/mol/K). Frequencies agree within 1 cm-1 and thermochemical corrections within
18 J/mol (< 0.02 kJ/mol). The g16 `Low frequencies` printout is the unprojected
spectrum (the neutral NH3 test geometry is far from the cation's stationary point);
the tabulated `Frequencies --` values are the projected true vibrations compared
here.

## Gaussian input (reference)

Deck used for the g16 run quoted above (Gaussian 16 Rev. B.01).

```
%chk=nh3p_camb3lyp.chk
%mem=2GB
%nprocshared=8
#p freq symm=loose cam-b3lyp/def2svp

NH3+ CAM-B3LYP/def2SVP freq (charge +1, doublet), validation for rest_regression analdrv/NH3_UCAM-B3LYP_hess

1 2
N  -2.1988391019      1.8973746268      0.0000000000
H  -1.1788391019      1.8973746268      0.0000000000
H  -2.5388353987      1.0925460144     -0.5263586446
H  -2.5388400276      2.7556271745     -0.4338224694
```
