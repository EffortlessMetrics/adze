# rust-sitter v0.6.1-beta

[![Crates.io](https://img.shields.io/crates/v/rust-sitter.svg)](https://crates.io/crates/rust-sitter)
[![CI](https://github.com/hydro-project/rust-sitter/workflows/CI/badge.svg)](https://github.com/hydro-project/rust-sitter/actions)

## 🎯 Algorithmically Correct GLR Parser

This beta delivers six end-game correctness fixes and brings core GLR suites to **100% pass**. Ambiguous grammars behave with true GLR semantics (multi-action cells, real fork/merge) while queries produce stable counts.

### ✅ What's Fixed
- **Reduce → re-closure (same lookahead)** — cascaded reduces & accepts found.
- **Per-token accept aggregation** — prevents early short-circuit.
- **EOF recovery loop**: `close → check → (insert|pop)` — never delete at EOF.
- **ε loop guard** keyed on `(state, rule, end)` to prevent infinite loops.
- **Nonterminal goto** semantics restored (no LHS via action table).

### 🧭 Improvements
- **Query correctness**: squash unary wrapper nodes with identical spans; dedup captures by `(symbol, start, end)`.
- **Fork/merge stability**: safe stack dedup removes only pointer-equal duplicates (optionally gated by threshold).
- **Testing**: replaced hand-rolled tables with LR(1) builder; adjusted fork depth expectations (LR(1) ambiguity often ≥3 tokens).

### 🧪 Tests (core suites)
- Fork/Merge: **30/30** ✅  
- Integration (queries): **5/5** ✅  
- Error Recovery: **5/5** ✅  
- GLR Parsing: **6/6** ✅  
- Regression Guards: **5/5** ✅

### ⚠️ Known Limitations (beta)
- Query predicates & advanced APIs in development.
- Incremental-GLR heuristics & equivalence suite WIP.
- CLI runtime loading & external scanner linking docs still pending.
- Performance baseline & safe-dedup heuristics to be tuned.

### Upgrade
```toml
[dependencies]
rust-sitter = "0.6.1-beta"
rust-sitter-tool = "0.6.1-beta" # optional
```