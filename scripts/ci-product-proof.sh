#!/usr/bin/env bash
set -euo pipefail

# Advisory product-surface proof lane.
# Intentionally bounded to compile/smoke checks so it stays fast and non-blocking.

if [[ "${1:-}" == "--list" ]]; then
  cat <<'LIST'
canary,type,command
adze-runtime,smoke-test,cargo test -p adze --lib -- --test-threads=2
adze-cli,compile-only,cargo check -p adze-cli
adze-golden-tests,compile-only,cargo test -p adze-golden-tests --no-run
adze-benchmarks,compile-only,cargo test -p adze-benchmarks --no-run
wasm-demo,compile-only,cargo check -p adze-wasm-demo
grammar-python-simple,compile-only,cargo check -p adze-python-simple
runtime2,compile-only,cargo test --manifest-path runtime2/Cargo.toml --no-run
microcrate-governance,compile-only,cargo check -p adze-runtime2-governance
LIST
  exit 0
fi

export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
export RUST_TEST_THREADS="${RUST_TEST_THREADS:-2}"

failures=0

run_canary() {
  local name="$1"
  local proof_type="$2"
  shift 2

  echo
  echo "=== ${name} (${proof_type}) ==="
  if "$@"; then
    echo "PASS: ${name}"
  else
    echo "FAIL: ${name}"
    failures=$((failures + 1))
  fi
}

run_canary "adze-runtime" "smoke-test" cargo test -p adze --lib -- --test-threads="$RUST_TEST_THREADS"
run_canary "adze-cli" "compile-only" cargo check -p adze-cli
run_canary "adze-golden-tests" "compile-only" cargo test -p adze-golden-tests --no-run
run_canary "adze-benchmarks" "compile-only" cargo test -p adze-benchmarks --no-run
run_canary "wasm-demo" "compile-only" cargo check -p adze-wasm-demo
run_canary "grammar-python-simple" "compile-only" cargo check -p adze-python-simple
run_canary "runtime2" "compile-only" cargo test --manifest-path runtime2/Cargo.toml --no-run
run_canary "microcrate-governance" "compile-only" cargo check -p adze-runtime2-governance

echo
if [[ "$failures" -eq 0 ]]; then
  echo "All product-proof canaries passed."
else
  echo "Product-proof canaries failed: ${failures}"
  exit 1
fi
