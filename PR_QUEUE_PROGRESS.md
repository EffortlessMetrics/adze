# PR Queue Progress

- Current ladder:
  1. `#420` — current-red blocker (`test_dense_column_mapping` block persists)
  2. GLR conflict semantics canonical family (`#388/#389/#390`)
  3. parser_v4 no-fallback canonical family (`#404/#405/#406/#411`)
  4. field-ID preservation family (`#400/#401/#402/#403`)
  5. pure-Rust diagnostics family (`#391/#392/#393/#394`)
  6. typed AST contract family (`#412/#414/#415/#416`)
  7. product-proof CI (`#395`)
  8. Criterion / bincode cleanup (`#396/#397/#398/#413`)
- Completed / merged / superseded:
  - No canonical landings yet in this local branch.
  - All local PR families are currently in compare/select mode except `#420`.
- Next action only: resolve #420 gate (or land dedicated table-invariant blocker fix) before advancing canonical merges.
