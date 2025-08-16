#!/usr/bin/env bash
set -euo pipefail

# Quick benchmarking for development loops
# Usage: ./scripts/bench-quick.sh [additional cargo bench args]

echo "Running quick benchmarks with perf-counters..."
env BENCH_QUICK=1 cargo bench -p rust-sitter-glr-core --features perf-counters "$@"