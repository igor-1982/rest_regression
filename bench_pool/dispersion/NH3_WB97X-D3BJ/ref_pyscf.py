# pyscf 2.13.0
# pyscf dispersion may need modify on np.ctypeslib.load_library

from pyscf import gto, dft

mol = gto.Mole(atom = "N 0 0 0; H 1 0 0; H 0 1 0; H 0 0 1", basis="def2-svp").build()
mf = dft.UKS(mol, xc="wB97X-D3BJ").density_fit().run()
print("pyscf wB97X-D3BJ energy: {:}".format(mf.e_tot))
assert abs(mf.e_tot - -56.513581122349514) < 1e-5
