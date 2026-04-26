#!/usr/bin/env bash
set -u -o pipefail

# Advisory canary lane for product surfaces outside ci-supported.
# This script is intentionally bounded to compile/smoke checks.

declare -i failures=0
dry_run=false

declare -a labels=()
declare -a proof_types=()
declare -a commands=()
declare -a statuses=()
declare -a notes=()

run_canary() {
  local label="$1"
  local proof_type="$2"
  local command="$3"
  local note="$4"

  echo
  echo "==> ${label} (${proof_type})"
  echo "    $command"

  local status
  if [[ "$dry_run" == "true" ]]; then
    status="DRY-RUN"
  elif eval "$command"; then
    status="PASS"
  else
    status="FAIL"
    failures+=1
  fi

  labels+=("$label")
  proof_types+=("$proof_type")
  commands+=("$command")
  statuses+=("$status")
  notes+=("$note")
}

summary_file=""
for arg in "$@"; do
  case "$arg" in
    --dry-run)
      dry_run=true
      ;;
    *)
      if [[ -z "$summary_file" ]]; then
        summary_file="$arg"
      else
        echo "unexpected argument: $arg" >&2
        exit 2
      fi
      ;;
  esac
done

run_canary \
  "adze runtime typed parse surface" \
  "compile-only" \
  "cargo check -p adze --locked" \
  "Confirms runtime crate compiles in the advisory lane."

run_canary \
  "adze-cli" \
  "compile-only" \
  "cargo check -p adze-cli --locked" \
  "CLI wiring compiles without executing commands."

run_canary \
  "adze-golden-tests" \
  "smoke-test (no-run)" \
  "cargo test -p adze-golden-tests --features all-grammars --no-run --locked" \
  "Builds golden-test harness without downloading external references or running parity suite."

run_canary \
  "adze-benchmarks" \
  "compile-only (bench --no-run)" \
  "cargo bench -p adze-benchmarks --no-run --locked" \
  "Ensures benchmark targets compile; no perf assertions are executed."

run_canary \
  "adze-wasm-demo" \
  "compile-only" \
  "cargo check -p adze-wasm-demo --target wasm32-unknown-unknown --locked" \
  "Requires wasm32 target; validates demo crate compiles for web target."

run_canary \
  "grammar canary (adze-python)" \
  "compile-only" \
  "cargo check -p adze-python --locked" \
  "One representative grammar surface canary."

run_canary \
  "runtime2 canary (adze-runtime)" \
  "compile-only" \
  "cargo check -p adze-runtime --no-default-features --features glr --locked" \
  "runtime2 crate/package is named adze-runtime."

run_canary \
  "governance/BDD microcrate canary (adze-bdd-governance-core)" \
  "compile-only" \
  "cargo check -p adze-bdd-governance-core --locked" \
  "Representative governance/BDD microcrate smoke canary."

if [[ -z "$summary_file" && -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
  summary_file="$GITHUB_STEP_SUMMARY"
fi

{
  echo "## Product proof canary summary"
  echo
  echo "| Surface | Proof type | Status | Command | Notes |"
  echo "|---|---|---|---|---|"
  for i in "${!labels[@]}"; do
    echo "| ${labels[$i]} | ${proof_types[$i]} | ${statuses[$i]} | ${commands[$i]} | ${notes[$i]} |"
  done
  echo
  if [[ "$dry_run" == "true" ]]; then
    echo "> Advisory result: dry-run completed; no cargo commands were executed."
  elif (( failures > 0 )); then
    echo "> Advisory result: ${failures} canary check(s) failed. This lane is non-blocking by design."
  else
    echo "> Advisory result: all canary checks passed."
  fi
} | tee /tmp/ci-product-proof-summary.md

if [[ -n "$summary_file" ]]; then
  cat /tmp/ci-product-proof-summary.md >> "$summary_file"
fi

# Always exit 0 so the advisory lane can publish a full matrix of PASS/FAIL signals.
exit 0
