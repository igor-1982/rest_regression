A tool to perform regression tests for REST

Usage: rest_regression [OPTIONS]

Options:
  -r, --regression_directory <regression_directory>
          The directory that contains selected regression tasks [default: ./bench_pool, then $REST_HOME/rest_regression/bench_pool]
  -w, --working_directory <working_directory>
          The working directory to store the regression output files [default: ./work_pool]
  -c, --rest_mode <rest_mode>
          The version of REST binary to invoke: "release" or "debug" [default: release]
  -p, --rest_path <rest_path>
          The absolute path to find the `rest` binary [default: $REST_HOME/target/{mode}/rest]
          When -p is provided, REST_HOME does not need to be set.
  -n, --n_mpi <n_mpi>
          The number of MPI tasks [default: 1]
  -t, --num_threads <num_threads>
          The number of OpenMP/Rayon threads. If not provided, each test keeps its own num_threads from ctrl.in.
          Overrides all normal tests; ignored for special tests listed in special_tests.toml.
  -f, --filter <filter>
          Comma-separated list of test names, folder names, or tags to run. Also matches parent folders
          (e.g. "gradient" runs all gradient tests) and tags from test_tags.toml (e.g. "mp2" runs all
          MP2 tests across folders). Panics if any name matches no test, folder, or tag.
  --skip <skip>
          Comma-separated list of test names, folder names, or tags to skip (e.g. "NH3_SCAN" or
          "solvent"). Same matching logic as --filter. Panics if any name matches nothing.
  --durations [<N>]
          Show the slowest N tests at the end [default: 5, use 0/off/false to disable]
  --dry-run
          List tests that would be run without executing them
  --timeout <timeout>
          Per-job timeout in seconds (0 = no timeout, default: no timeout).
          When set, each REST job is killed if it exceeds this duration.
          Tests can override the global timeout via tests.toml (see below).
  -h, --help
          Print help
  -V, --version
          Print version

Special tests and tags:
  tests.toml stores per-test configuration: num_mpi, num_threads (locked values, cannot be
  overridden by CLI), timeout (per-test override in seconds; 0 = no timeout), tags
  (cross-folder filter labels), and requires (Python packages needed to run the test).
  Available tags: mp2, ecp, dipole, ext-field, rsh, lda, ro, gga, meta-gga, parsexc, chkfile,
  hessian, hf, frequencies, opt, analdrv, thermo
  Usage: rest_regression -f mp2,ext-field

Dependency auto-skip:
  A test may declare requires = ["ase", "openmm"] in tests.toml. When a listed package
  cannot be imported, the test is skipped automatically with a WARNING line (counted as
  skipped in the summary; neither passed nor failed). Importability is probed once per
  package via a subprocess `python3 -c "import ..."` (falling back to `python`); set the
  REST_REGRESSION_PYTHON environment variable to probe a specific interpreter.

Reference mode (config.toml):
  Each test is compared either by the default log scraping or in json mode, against the json
  result file dumped by REST (reference.json vs the freshly generated one). The default mode
  is "auto": json comparison when the test folder contains reference.json, log scraping
  otherwise, so most tests need no config.toml at all. An optional config.toml pins the mode
  explicitly and adds required fields / tolerance overrides:

    [config]
    reference = "auto"          # "auto" (default), "json" or "log"; explicit values override detection
    fields = [ "scf_energy", "thermo/cv_vib" ]   # required field paths, "/" = json nesting

    [tolerance]                   # optional user overrides of the default tolerance table
    "thermo/zpe" = 1.0

  Only fields present in BOTH json files are compared (new fields of future REST versions are
  ignored); fields and tolerance keys must be present in both or the test fails. Numbers compare
  numpy-isclose style: |out - ref| <= atol + rtol * |ref|, where a single tolerance value sets
  atol (default 1e-8) and a pair [atol, rtol] sets both (default rtol 1e-5); arrays are compared
  elementwise under the same rule and must match in length, strings case-insensitively.
  The input card must set outname = "reference" so that REST writes reference.json; generating
  reference.log (rest -i ctrl.in > reference.log) then leaves reference.json in the folder too,
  which auto-detection picks up without any config.toml.
  See docs/config-toml.md for the full rules (Chinese).

MD / QM-MM tests (bench_pool/md, bench_pool/qmmm_md):
  Tests with job_type = "md" (AIMD, QM/MM, pure MM) require the Python runtime used by the
  REST binary to provide numpy, ase and openmm (including openmm.app). Pure-QM AIMD only needs
  ASE/numpy; QM/MM and pure-MM additionally need OpenMM. Tested with Python 3.10, numpy 2.2,
  ASE 3.29, OpenMM 8.6.
  These tests declare their needs in tests.toml (requires = [...]); when ase/openmm are not
  importable they are skipped with a warning instead of failing (see "Dependency auto-skip").
