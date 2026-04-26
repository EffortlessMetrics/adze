#!/usr/bin/env bash
set -u

# Advisory product-surface canaries.
# This script intentionally runs bounded compile/smoke checks and is designed
# to be used from a non-blocking CI workflow.

run_check() {
  local name="$1"
  local proof_type="$2"
  shift 2

  echo ""
  echo "==> ${name} [${proof_type}]"
  echo "+ $*"

  if "$@"; then
    RESULTS+=("PASS|${name}|${proof_type}")
    return 0
  fi

  RESULTS+=("FAIL|${name}|${proof_type}")
  return 1
}

RESULTS=()
FAILURES=0

# Global sanity check.
run_check "workspace metadata" "compile-only" cargo metadata --format-version 1 >/dev/null || FAILURES=$((FAILURES + 1))

# 1) Main runtime surface (adze) canary.
run_check "adze runtime smoke" "smoke-test" cargo test -p adze --lib -- --list || FAILURES=$((FAILURES + 1))

# 2) CLI canary.
run_check "adze-cli" "compile-only" cargo check -p adze-cli --locked || FAILURES=$((FAILURES + 1))

# 3) Golden tests surface canary.
run_check "adze-golden-tests" "compile-only" cargo test -p adze-golden-tests --no-run --locked || FAILURES=$((FAILURES + 1))

# 4) Benchmarks surface canary.
run_check "adze-benchmarks" "compile-only" cargo bench -p adze-benchmarks --no-run --locked || FAILURES=$((FAILURES + 1))

# 5) Wasm demo canary.
run_check "adze-wasm-demo" "compile-only" cargo check -p adze-wasm-demo --target wasm32-unknown-unknown --locked || FAILURES=$((FAILURES + 1))

# 6) One grammar surface canary.
run_check "grammar smoke (adze-python-simple)" "compile-only" cargo check -p adze-python-simple --locked || FAILURES=$((FAILURES + 1))

# 7) runtime2 surface canary.
run_check "runtime2 smoke (adze-runtime)" "compile-only" cargo test -p adze-runtime --no-run --locked || FAILURES=$((FAILURES + 1))

# 8) One governance/BDD microcrate canary.
run_check "microcrate smoke (adze-bdd-contract)" "compile-only" cargo check -p adze-bdd-contract --locked || FAILURES=$((FAILURES + 1))

echo ""
echo "Product proof summary:"
echo "status|canary|proof"
for row in "${RESULTS[@]}"; do
  echo "$row"
done

if [[ "$FAILURES" -gt 0 ]]; then
  echo ""
  echo "Advisory canary failures: ${FAILURES}"
  exit 1
fi

echo ""
echo "All advisory product canaries passed."
