# Current Failure Map

- Date: 2026-04-26
- Live board snapshot: 106 open PRs
- First actionable failure: `#420` (`fix(pure-rust): avoid SymbolId alias construction in external scanner test`)
- Current blocker: `runtime/tests/test_table_invariants.rs::test_dense_column_mapping` (`Non-dense mapping at column 0 after decode`)
- Current merge order:
  1. `#420`
  2. one GLR conflict-semantics PR (`#388/#389/#390`)
  3. one parser_v4 no-fallback PR (`#404/#405/#406/#411`)
  4. one field-ID preservation PR (`#400/#401/#402/#403`)
  5. one pure-Rust diagnostics PR (`#391/#392/#393/#394`)
  6. one typed AST contract PR (`#412/#414/#415/#416`)
  7. product-proof PR (`#395`)
  8. one Criterion/bincode cleanup PR (`#396/#397/#398/#413`)
