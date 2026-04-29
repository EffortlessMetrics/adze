#!/usr/bin/env bash
set -euo pipefail

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN=true
fi

# Canary definitions: "label|proof_type|command"
CANARIES=(
  "adze runtime pure-rust smoke|compile-only|cargo check -p adze --features pure-rust"
  "adze-cli smoke|compile-only|cargo check -p adze-cli"
  "golden-tests smoke|compile-only|cargo test -p adze-golden-tests --features python-grammar --no-run"
  "benchmarks canary|compile-only|cargo bench -p adze-benchmarks --no-run"
  "wasm-demo canary|compile-only|cargo check --manifest-path wasm-demo/Cargo.toml --target wasm32-unknown-unknown"
  "grammar smoke (python)|compile-only|cargo check -p adze-python"
  "runtime2 canary|compile-only|cargo test --manifest-path runtime2/Cargo.toml --no-run"
  "governance/BDD microcrate smoke|compile-only|cargo test -p adze-bdd-grid-core --lib --no-run"
)

printf '== ci-product advisory canaries ==\n'
printf 'Mode: %s\n\n' "$([[ "$DRY_RUN" == true ]] && echo dry-run || echo execute)"

failures=0
for entry in "${CANARIES[@]}"; do
  IFS='|' read -r label proof_type cmd <<<"$entry"
  printf '\n[%s] %s\n' "$proof_type" "$label"
  printf '  $ %s\n' "$cmd"

  if [[ "$DRY_RUN" == true ]]; then
    continue
  fi

  if eval "$cmd"; then
    printf '  -> PASS\n'
  else
    printf '  -> FAIL\n'
    failures=$((failures + 1))
  fi
done

if [[ "$DRY_RUN" == true ]]; then
  printf '\nDry run complete.\n'
  exit 0
fi

if [[ $failures -gt 0 ]]; then
  printf '\nci-product completed with %d failing canary(s).\n' "$failures"
  exit 1
fi

printf '\nci-product completed successfully.\n'
