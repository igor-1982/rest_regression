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
          Comma-separated list of test names to run (e.g. "H2O_guessfile,NH3_SCAN"). Panics if any name
          matches no test directory.
  --skip <skip>
          Comma-separated list of test names to skip (e.g. "NH3_SCAN,NH3_SVWN"). Panics if any name
          matches no test directory.
  --durations [<N>]
          Show the slowest N tests at the end [default: 5, use 0/off/false to disable]
  -h, --help
          Print help
  -V, --version
          Print version

Special tests:
  Tests listed in special_tests.toml have locked -n/-t values that cannot be overridden by CLI flags.
  Currently: C4H4_mix_yamaguchi (num_mpi=1, num_threads=1)
