#!/usr/bin/env bash
set -euo pipefail

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN=true
fi

# Stable product canaries map to README Stable claims only.
# Broader stabilizing/advisory surfaces remain in scripts/ci-product.sh.
CANARIES=(
  "README stable proof alignment|cargo test -p adze-cli readme_stable_claims_are_in_stable_product_lane -- --exact --nocapture"
  "typed extraction exact value|cargo test -p adze --features pure-rust --test typed_ast_contract typed_ast_contract_left_associative_addition -- --exact --nocapture"
  "typed extraction repeated-parse determinism|cargo test -p adze --features pure-rust --test typed_ast_contract typed_ast_contract_repeated_parse_is_deterministic -- --exact --nocapture"
  "README quickstart clean-room parse and diagnostics|cargo test -p adze-cli readme_arithmetic_quickstart_builds_and_runs -- --exact --nocapture"
  "checked-in downstream quickstart sample|cargo test -p downstream-demo -- --nocapture"
  "operator precedence core shape|cargo test -p adze-glr-core --test ambiguity_detection_comprehensive test_precedence_resolves_add_mul -- --exact --nocapture"
  "core parse-table serialization doctests|cargo test -p adze-glr-core --features serialization --doc"
  "core parse-table serialization roundtrip|cargo test -p adze-glr-core --features serialization --test serialization_v9 sv9_complex_precedence_roundtrip -- --exact --nocapture"
)

printf '== ci-product stable canaries ==\n'
printf 'Mode: %s\n\n' "$([[ "$DRY_RUN" == true ]] && echo dry-run || echo execute)"

for entry in "${CANARIES[@]}"; do
  IFS='|' read -r label cmd <<<"$entry"
  printf '\n[stable] %s\n' "$label"
  printf '  $ %s\n' "$cmd"

  if [[ "$DRY_RUN" == true ]]; then
    continue
  fi

  eval "$cmd"
  printf '  -> PASS\n'
done

if [[ "$DRY_RUN" == true ]]; then
  printf '\nDry run complete.\n'
else
  printf '\nci-product-stable completed successfully.\n'
fi
