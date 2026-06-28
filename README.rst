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
  -h, --help
          Print help
  -V, --version
          Print version

Special tests and tags:
  tests.toml stores per-test configuration: num_mpi, num_threads (locked values, cannot be
  overridden by CLI), and tags (cross-folder filter labels).
  Available tags: mp2, ecp, dipole, ext-field, rsh, lda, ro, gga, meta-gga
  Usage: rest_regression -f mp2,ext-field
