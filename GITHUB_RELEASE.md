# rust-sitter v0.6.1-beta

[![Crates.io](https://img.shields.io/crates/v/rust-sitter.svg)](https://crates.io/crates/rust-sitter)
[![Build Status](https://github.com/hydro-project/rust-sitter/workflows/CI/badge.svg)](https://github.com/hydro-project/rust-sitter/actions)

## 🎯 Algorithmically Correct GLR Parser

This beta lands six end-game correctness fixes and brings the core GLR suites to **100% pass**. Ambiguous grammars now behave with true GLR semantics (multi-action cells, real fork/merge) while queries produce stable counts.

## ✅ What's Fixed

### Core Algorithm
* **Reduce → re-closure (same lookahead)** to expose cascaded reduces/accepts
* **Per-token accept aggregation**, no early short-circuit
* **EOF recovery loop**: close → check → *(insert|pop)*; never delete at EOF
* **ε loop guard** keyed on `(state, rule, end)` to prevent re-fires
* **Nonterminal goto** restored (no LHS via action table)

### Query & Forest
* **Query correctness**: squash exact-span unary wrappers; dedup captures by `(symbol,start,end)`
* **Fork/merge stability**: safe stack dedup removes only pointer-equal duplicates

## 🧪 Test Results

| Suite | Pass Rate | Status |
|-------|-----------|--------|
| Fork/Merge | 30/30 | ✅ |
| Integration | 5/5 | ✅ |
| Error Recovery | 5/5 | ✅ |
| GLR Parsing | 6/6 | ✅ |
| Regression Guards | 5/5 | ✅ |

## ⚠️ Known Limitations (Beta)

* **Queries**: predicates & advanced APIs pending
* **Incremental GLR**: experimental; heuristics & equivalence suite WIP
* **CLI**: runtime loading & corpus runner WIP
* **External scanners**: linking workflow/docs to finish
* **Performance**: baselines next; safe-dedup heuristics by threshold

## 📦 Installation

```toml
# Cargo.toml
[dependencies]
rust-sitter = "0.6.1-beta"

[build-dependencies]
rust-sitter-tool = "0.6.1-beta"
```

## 🔧 What Changed

<details>
<summary>Technical Details</summary>

### Fixes Applied
1. Phase-2 reductions now re-saturate with lookahead to find cascaded reduces
2. Accept states aggregate per-token instead of returning early
3. EOF recovery implements close→check→recover pattern without deletion
4. Epsilon reduction stamps include position to prevent loops
5. Wrapper nodes with identical spans collapse in queries
6. Stack deduplication uses pointer equality to preserve ambiguities

### Test Infrastructure
- Replaced hand-crafted parse tables with proper LR(1) automaton builder
- Adjusted fork depth expectations (LR(1) ambiguity surfaces at ≥3 tokens)
- Added regression guard tests for all critical fixes

</details>

## 🚀 Next Steps

This release achieves algorithmic correctness. Coming next:
- Performance optimization and benchmarking
- Query predicate support
- Incremental parsing improvements
- External scanner documentation
- CLI runtime loader

---

**Full Changelog**: https://github.com/hydro-project/rust-sitter/compare/v0.6.0...v0.6.1-beta