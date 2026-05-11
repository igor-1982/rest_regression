# pyscf 2.13.0

from pyscf import gto, dft

mol = gto.Mole(atom = "N 0 0 0; H 1 0 0; H 0 1 0; H 0 0 1", basis="def2-svp", charge=1, spin=1).build()
mf = dft.UKS(mol, xc="CAM-B3LYP").density_fit().run()
print("pyscf CAM-B3LYP energy: {:}".format(mf.e_tot))
assert abs(mf.e_tot - -56.021955023891) < 1e-5
