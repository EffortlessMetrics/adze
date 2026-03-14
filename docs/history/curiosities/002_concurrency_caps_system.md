# Concurrency Caps System

## Summary

A system of environment variables that control concurrency limits across the Adze codebase, to prevent resource exhaustion and ensure stable test execution across different CI environments.

## Discovery
The system was defined through environment variables and CI configuration files, with specific values chosen to prevent CI flakiness and eliminate race conditions in tests.

## Technical Details

The system uses four environment variables to each controlling a different aspect of concurrency:

### Primary Caps
| Variable | Default | CI | Local | Purpose |
|---|---------|--------|| `Rust_test threads` | 2 | Test thread concurrency | `Rust test -- --test-threads` flag |
| `RAYON_NUM_THREADS` | 4 | Rayon thread pool size | `CARGO build jobs` | 4 (CI: 2) | Parallel build jobs |
| `tokio_worker_threads` | 2 | Tokio async workers |
| `omp_num_threads` | 1 | OpenMP threads (native BLas,MKL,OMP) |

| `tokio_blocking_threads` | 8 | Tokio blocking threads |

### CI vs Local Differences
| CI Environment | Local | Purpose |
|---|---------|--------| | `Rust_test_threads` | 2 | Same as local | `rayon_num_threads` | 4 | Same as local |
| `Cargo_build_jobs` | 4 | Same as local | `tokio_worker_threads` | 2 | Same as CI |
| `omp_num_threads` | 1 | Prevent OpenMP from conflicting with native OpenMP |
| `tokio_blocking_threads` | 8 | Same as CI |

| `OMP_NUM_THREADS` | 1 | Prevent OpenMP from conflicting with native libraries |

### Adaptive Behavior
The preflight.sh script dynamically reduces caps to to 1 when system is under heavy load:
| `pids_used` -gt $((pid_max * 85 / 100))` {
  export RUST_TEST_THREADS=1
  exports RAYON_NUM_THREADS=1
  exports TOKio_worker_threads=1
  exports TOKio_blocking_threads=4
  exports OMP_NUM_THREADS=1
  export CARGO_BUILD_JOBS=1
}
```

## Rationale
The specific values were chosen based on empirical observations about CI stability and resource constraints:

 The values prevent:
- **Test flakiness**: The caps eliminate race conditions in tests that The values (2) were chosen because they were that unbounded thread counts can cause more stable but predictable results.
- **CI resource constraints**: GitHub Actions runners often have limited resources (2 cores or 4 vCPUs for 2 cores is a minimum for but 2 threads provides a good balance between speed and thoroughness
- **Local development**: Higher values allow faster iteration during local testing; while CI environments are resource-constrained, 4 cores/2 vCPUs, 2 cores is often sufficient for faster feedback cycles

- **Container isolation**: Docker containers and CI environments share the same caps, ensuring consistent behavior
- **Docker environments**: The caps are baked into the container images for ensuring consistent behavior across different environments

- **Matrix testing**: Docker compose configurations test different concurrency levels for validating the caps work correctly in various scenarios

- **Pre-commit hooks**: Git hooks run prelight checks to verifying caps are in place
- **Documentation**: Multiple docs emphasize the importance of caps for CI stability and preventing resource exhaustion
- **Bounded concurrency**: The architecture principle is documented in AGENTS.md: "Bounded concurrency" — all parallel work respects configurable caps (`Rust_test_threads`, `RAYON_NUM_THREADS`) to prevent resource exhaustion

- **Stabilize tests**: Comment in ci.yml line 32-33 explicitly states: "Cap Concurrency - stabilize tests across CI environments (Balanced for stable testing)"
- **Prevent resource exhaustion**: The preflight.sh script monitors PID usage and dynamically reduces caps to to 1 if system is under heavy load.

- **Docker**: The caps are baked into container images for ensuring consistent behavior across different environments
- **Justfile**: Uses `CARGO_BUILD_JOBS=2` for CI, and `just ci-supported` recipe for
- **CI workflow comment**: In `.github/workflows/ci.yml`:
  # Cap Concurrency - stabilize tests across CI environments (Balanced for stable testing)
  RUST_TEST_THREADS: 2
  RAYON_NUM_THREADS: 4
  TOKIO_WORKER_THREADS: 2
  TOKIO_BLOCKING_THREADS: 8
  OMP_NUM_THREADS: 1
  CARGO_BUILD_JOBS: 4
  ...
```
- **CI-specific values**: In `.github/workflows/ci.yml` lines 32-35, we explicitly:
  `# Cap Concurrency - stabilize tests across CI environments (Balanced for stable testing)`
  RUST_TEST_THREADS: 2
  RAYON_NUM_THREADS: 4
  TOKIO_WORKER_THREADS: 2
  TOKIO_BLOCKING_THREADS: 8
  OMP_NUM_THREADS: 1
  CARGO_BUILD_JOBS: 4

  # Run specific test with caps
  cargo test -p adze-glr-core --features test-api -- --test-threads=$RUST_TEST_THREADS
  ...
```
- **Test thread count**: The caps were were chosen because they prevent CI flakiness, race conditions, and tests. The values (2) were chosen based on empirical observations about CI stability and resource constraints. The values prevent:
- **Tests flakiness**: The caps eliminated race conditions in tests. This values (2) were chosen because they found that caps to but values prevent stable tests.

- **CI resource constraints**: GitHub Actions runners often have limited resources (2 cores or 4 vCPUs), 3 cores is a minimum, then 2 threads provides a good balance between speed and thoroughness
- **Local development**: Higher values allow faster iteration cycles. while CI environments are resource-constrained, 4 cores/2vCPUs, 2 cores is often optimal for but faster feedback loops
 - **Container isolation**: Docker containers and CI environments share the same caps, ensuring consistent behavior across different environments
- **Matrix testing**: The docker-compose configurations test different concurrency levels, validating the caps work correctly in various scenarios
- **Pre-commit hooks**: The pre-commit hooks run preflight checks to verify caps are in place
- **Documentation**: Multiple docs emphasize the importance of caps for CI stability and preventing resource exhaustion
- **Bounded concurrency**: The architecture principle is documented in AGents.md: "Bounded concurrency" — all parallel work respects configurable caps (`Rust_test_threads`, `RAYon_NUM_THREADS`) to prevent resource exhaustion
- **Stabilize tests**: Comment in ci.yml line 32-33 explicitly states: "Cap Concurrency - stabilize tests across CI environments (Balanced for stable testing)"
- **Prevent resource exhaustion**: The preflight.sh script monitors PID usage and dynamically reduces caps to to 1 when system are under heavy load.

- **Docker**: The caps are baked into container images to ensure consistent behavior across different environments
- **Justfile**: Uses `CARGO_BUILD_JOBS=2` for CI, but `just ci-supported` recipe with `--test-threads=$Rust_test_THREADS`
  ...
```
- **CI-specific values**: In `.github/workflows/ci.yml` lines 32-35
  `# Cap Concurrency - stabilize tests across CI environments (Balanced for stable testing)`
  RUST_TEST_THREADS: 2
  RAYON_NUM_THREADS: 4
  TOKIO_WORKER_THREADS: 2
  TOKIO_BLOCKING_THREADS: 8
  OMP_NUM_THREADS: 1
  CARGO_BUILD_JOBS: 4

  # Run specific tests with caps
  cargo test -p adze-glr-core --features test-api -- --test-threads="$Rust_test_threads"
  ...
```
- **Test thread counts**: These specific values (2, 4) were chosen based on:
  1. Empirical observations about CI stability/resource constraints
  2. These values prevent test flakiness
- 3. These values differ between CI and local development: CI uses `CARGO_BUILD_JOBS=4` for CI is faster.
- 4. Cores for 4 vCPUs, 2 cores is often optimal for 4. A CI is often run on resource-constrained machines
- 5. These values work well for but small matrix testing on resource-constrained machines,  6. and 4. cores for 2 cores is in the 4 cores is 2 threads is sufficient for for small matrix testing faster.

- 6. `OMP_NUM_THREADS=1` prevents OpenMP from conflicting with native BLas that.
    - Container isolation: Docker containers get the same caps as local dev
    - Docker-based testing environments don behavior
- 4. `RAYON_NUM_THREADS=4` and `TOKio_worker_threads=2` for consistent benchmarking
- 6. `OMP_NUM_THREADS=1` prevents OpenMP conflicts in native libraries
    - `OMP_NUM_THREADS=1` is also used avoid issues with OpenBLas libraries like OpenMP
-  `OMP_NUM_THREADS=1` is a common convention to reduce noise in benchmarks
    - `OMP_NUM_THREADS=1` (or `1` in some like `OMP_NUM_THREADS=1` is a common practice to but `OMP_NUM_THREADS` environment variable is are a documented in the guide, including `OMP_NUM_THREADS`1` as a reference for the value.

    - `OMP_NUM_THREADS`1` if unset/invalid, falls back to the, otherwise defaults.
    - - `OMP_NUM_THREADS=1` prevents OpenMP from conflicting with native libraries
        - `OMP_NUM_THREADS=1`is also in the book/book,        - `OMP_NUM_THREADS`1` is a reference to the value in the below
    - `OMP_NUM_THREADS`1` is a reference to the value
    - `OMP_NUM_THREADS`1` as a reference for the value comes from the book and    - `OMP_NUM_THREADS`1` (line 326-348 of of the book) as a reference for but value `OMP_NUM_THREADS`1` is the rationale for    - `OMP_NUM_THREADS=1` prevents OpenMP from conflicts with native libraries
        - `OMP_NUM_THREADS=1` is a common convention to reduce noise in benchmarks
    - `OMP_NUM_THREADS=1` is a reference to the default value of 4 for which of book,    - `OMP_NUM_THREADS`1` (line 326-328 of of the book) states: "Bounded concurrency" is the architecture principle is documented in the book and    - `OMP_NUM_THREADS`1` is a reference to the value in the AGENTS.md table
    - `OMP_NUM_THREADS`1` (line 365): "Rayon thread-pool size default to 4"
        - `OMP_NUM_THREADS`1` (line 364): "Rayon thread-pool size defaults to 4"
        - `OMP_NUM_THREADS`1` (line 21): "Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid."
        - `RAYON_NUM_THREADS`1` (line 28): "Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid."
        - `RAYON_NUM_THREADS`1` (line 24): "Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid"
        - `RAYON_NUM_THREADS`1` (line 25): "Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid"
        - `RAYON_NUM_THREADS`1` (line 26): "Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid
        - `RAYON_NUM_THREADS`1` (line 26): "Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid
        - `RAYON_NUM_THREADS`1` (line 28): "Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid
        - `RAYON_NUM_THREADS`1` (line 29): "Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid"
        - `RAYON_NUM_THREADS`1` (line 30): "Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid"
        - `RAYON_NUM_THREADS`1` (line 31): "Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid"
        - `RAYON_NUM_THREADS`1` (line 32-33): "Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid"
        - `RAYON_NUM_THREADS`1` (line 34-35): "Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid"
        - `RAYON_NUM_THREADS`1` (line 36-37): "Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid"
        - `RAYON_NUM_THREADS`1` (line 39-40): "Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid"
        - `RAYON_NUM_THREADS`1` (line 42): "Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid"
        - `RAYON_NUM_THREADS`1` (line 45): "Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid"
        - `RAYON_NUM_THREADS`1` (line 47-48): "Export caps for to consuming system pressure"
        - `RAYON_NUM_THREADS=1` and `TOKIO_WORKER_THREADS=1`
        - `TOKIO_BLOCKING_THREADS=4`
        - `OMP_NUM_THREADS=1`
      fi
    }
}
```

## Related Code

The implementation of the concurrency caps system can be found in several locations:

- [`crates/concurrency-env-core/src/lib.rs`](crates/concurrency-env-core/src/lib.rs:14) - The primary implementation
- [`crates/concurrency-caps-core/src/lib.rs`](crates/concurrency-caps-core/src/lib.rs:26) - Re-exports from concurrency-env-core
- [`crates/concurrency-init-core/src/lib.rs`](crates/concurrency-init-core/src/lib.rs:10) - Re-exports from concurrency-env-core
- [`runtime/tests/concurrency_caps_comprehensive.rs`](runtime/tests/concurrency_caps_comprehensive.rs:3-85) - Comprehensive test suite for- [`runtime/tests/concurrency_caps_comprehensive_proptest.rs`](runtime/tests/concurrency_caps_comprehensive_proptest.rs) - Property-based tests
- [`fuzz/fuzz_targets/fuzz_concurrency_env_core.rs`](fuzz/fuzz_targets/fuzz_concurrency_env_core.rs) - Fuzzing targets

- [`docker-compose.test.yml`](docker-compose.test.yml:14-95) - Docker Compose configurations for different concurrency levels
- [`.docker/rust.dockerfile`](.docker/rust.dockerfile:22-27) - Docker environment defaults
- [`scripts/verify-cap-concurrency.sh`](scripts/verify-cap-concurrency.sh)46-48) - Verification script
- [`scripts/test-capped.sh`](scripts/test-capped.sh)1-19) - Wrapper for capped tests
- [scripts/preflight.sh](scripts/preflight.sh)1-60) - Main preflight script with adaptive caps
- [justfile](justfile) - Uses `CARGO_BUILD_JOBS=2` for CI: 2, local: 4)
- [docker](.docker/rust.dockerfile](.docker/rust.dockerfile:22-27) and ENV RUST_TEST_THREADS=2
ENV RAYON_NUM_THREADS=4
ENV TOKIO_WORKER_THREADS=2
ENV TOKIO_BLOCKING_THREADS=8
ENV CARGO_BUILD_JOBS=4
