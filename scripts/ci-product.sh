#!/usr/bin/env bash
set -euo pipefail

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN=true
fi

# Canary definitions: "label|proof_type|command"
CANARIES=(
  "adze runtime pure-rust typed extraction|behavior|cargo test -p adze --features pure-rust --test typed_ast_contract typed_ast_contract_left_associative_addition -- --exact --nocapture"
  "adze GLR ambiguous typed extraction|behavior|cargo test -p adze --features \"pure-rust,glr,runtime-e2e\" --test test_e2e_ambiguous_grammar_glr test_ambiguous_grammar_glr_parsing -- --exact --nocapture"
  "adze GLR generated conflict preservation|behavior|cargo test -p adze --features \"pure-rust,glr,runtime-e2e\" --test test_e2e_ambiguous_grammar_glr test_ambiguous_grammar_conflict_generation -- --exact --nocapture"
  "adze structured parse diagnostics|behavior|cargo test -p adze --test error_display_tests reporting_parse_with_errors_includes_source_excerpt_after_bad_input --features \"pure-rust,glr\" -- --exact --nocapture"
  "adze multiline parse diagnostic location|behavior|cargo test -p adze --test error_display_tests reporting_parse_with_errors_tracks_multiline_bad_input_location_and_excerpt --features \"pure-rust,glr\" -- --exact --nocapture"
  "adze parse diagnostic byte spans|behavior|cargo test -p adze --test error_display_tests reporting_parse_diagnostics_include_byte_span_for_multiline_bad_input --features \"pure-rust,glr\" -- --exact --nocapture"
  "adze core parse-table serialization roundtrip|behavior|cargo test -p adze-glr-core --features serialization --test serialization_v9 sv9_complex_precedence_roundtrip -- --exact --nocapture"
  "adze tablegen ABI compressed decode roundtrip|behavior|cargo test -p adze --features \"pure-rust,glr,ts-compat\" --test tablegen_abi_decode_roundtrip compressed_tslanguage_decode_preserves_metadata_actions_and_fields -- --exact --nocapture"
  "adze tablegen ABI conflict decode preservation|behavior|cargo test -p adze --features \"pure-rust,glr,runtime-e2e,ts-compat\" --test test_e2e_ambiguous_grammar_glr tablegen_abi_decode_preserves_generated_conflict_cells -- --exact --nocapture"
  "README arithmetic quickstart clean-room|behavior|cargo test -p adze-cli readme_arithmetic_quickstart_builds_and_runs -- --exact --nocapture"
  "adze-cli clean-room init/check smoke|behavior|cargo test -p adze-cli test_init_generates_buildable_project -- --exact --nocapture"
  "adze-cli parse unsupported-mode truthfulness|behavior|cargo test -p adze-cli test_parse_static_mode_is_explicitly_unimplemented -- --exact --nocapture"
  "golden-tests javascript canary|behavior|cargo test -p adze-golden-tests javascript_canary_expression_golden --features javascript-grammar -- --nocapture"
  "benchmark arithmetic fixture validity|behavior|cargo test -p adze-benchmarks --test verify_fixture_parsing verify_arithmetic_benchmark_fixtures_parse_with_arithmetic_grammar -- --exact --nocapture"
  "benchmarks canary|compile-only|cargo bench -p adze-benchmarks --no-run"
  "wasm-demo canary|compile-only|cargo check --manifest-path wasm-demo/Cargo.toml --target wasm32-unknown-unknown"
  "grammar metadata smoke (python)|behavior|cargo test -p adze-python test_python_language_exists -- --exact --nocapture"
  "runtime2 metadata smoke|behavior|cargo test --manifest-path runtime2/Cargo.toml --features test-utils --test basic language_smoke_exposes_metadata_queries -- --exact --nocapture"
  "governance/BDD microcrate smoke|behavior|cargo test -p adze-bdd-grid-core --lib tests::progress_summary_reports_counts -- --exact --nocapture"
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
