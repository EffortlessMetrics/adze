#!/usr/bin/env bash
set -euo pipefail

# Cap Concurrency - System Pressure Monitor and Caps
# Fail-safe caps if the machine is "hot"

# Show current system pressure
pids_used=$(ps -e --no-headers | wc -l | xargs)
pid_max=$(cat /proc/sys/kernel/pid_max)
files_used=$(awk '{print $2}' /proc/sys/fs/file-nr)
files_max=$(cat /proc/sys/fs/file-max)
echo "PIDs: $pids_used / $pid_max | Open files: $files_used / $files_max"

# Conservative defaults (overridable via env in CI)
export RUST_TEST_THREADS="${RUST_TEST_THREADS:-2}"
export RAYON_NUM_THREADS="${RAYON_NUM_THREADS:-4}"

# Tokio runtime caps (used by async tests)
export TOKIO_WORKER_THREADS="${TOKIO_WORKER_THREADS:-2}"
export TOKIO_BLOCKING_THREADS="${TOKIO_BLOCKING_THREADS:-8}"

# Numeric library thread caps (for ML/scientific computing deps)
export OMP_NUM_THREADS="${OMP_NUM_THREADS:-1}"
export OPENBLAS_NUM_THREADS="${OPENBLAS_NUM_THREADS:-1}"
export MKL_NUM_THREADS="${MKL_NUM_THREADS:-1}"
export NUMEXPR_NUM_THREADS="${NUMEXPR_NUM_THREADS:-1}"

# Rust-specific caps
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}"

# If near PID limit, drop to ultra-safe mode
if [ "$pids_used" -gt $((pid_max * 85 / 100)) ]; then
  export RUST_TEST_THREADS=1
  export RAYON_NUM_THREADS=1
  export TOKIO_WORKER_THREADS=1
  export TOKIO_BLOCKING_THREADS=4
  export CARGO_BUILD_JOBS=1
  export OMP_NUM_THREADS=1 
  export OPENBLAS_NUM_THREADS=1 
  export MKL_NUM_THREADS=1 
  export NUMEXPR_NUM_THREADS=1
  echo "System hot → auto-degraded workers (RUST_TEST=1, RAYON=1, TOKIO=1, CARGO=1)"
fi

# Export caps for consumption by other scripts
echo "Concurrency caps: RUST_TEST_THREADS=$RUST_TEST_THREADS RAYON_NUM_THREADS=$RAYON_NUM_THREADS"
echo "Tokio caps: WORKER_THREADS=$TOKIO_WORKER_THREADS BLOCKING_THREADS=$TOKIO_BLOCKING_THREADS"