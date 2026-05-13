# Product Proof Map

**Last updated:** 2026-05-13
**Source of truth for:** README feature claims and release notes.

This document is a release-readable companion to `SUPPORT_TIERS.md`. It maps
each product claim to its spec, proof command, and release status.

## Core thesis

Adze has one canonical parse product: `AdzeDocument`. Everything else is a
projection.

```
parse::<T>()           stable typed AST front door
parse_document()       experimental native document boundary
  |
  +-- doc.ast::<T>()         experimental typed AST projection
  +-- doc.syntax()           experimental typed CST projection
  +-- doc.diagnostics()      stabilizing structured diagnostics
  +-- doc.as_tree_sitter()   advisory ts_compat adapter
  +-- doc.ambiguities()      experimental GLR ambiguity summary
  +-- doc.to_json_value()    experimental JSON projection
  +-- LanguageSchema          advisory metadata projection
```

## Proof map

| Product claim | 0.9 status | Source of truth | Proof command | Release note |
|---|---|---|---|---|
| Typed extraction works for supported generated grammars | Stable | ADZE-SPEC-0004, SUPPORT_TIERS | `cargo test -p adze --features pure-rust --test typed_ast_contract` | 0.8+ stable |
| Pure-Rust parser is the supported path | Stable | SUPPORT_TIERS | `just ci-supported` | 0.8+ stable |
| Operator precedence resolves correctly | Stable | SUPPORT_TIERS | `cargo test -p adze-cli readme_arithmetic_quickstart_builds_and_runs -- --exact --nocapture` | 0.8+ stable |
| GLR conflict routing works for ambiguous grammars | Stabilizing | ADZE-SPEC-0007, SUPPORT_TIERS | `cargo test -p adze-glr-core conflict ambiguity -- --nocapture` | 0.8+ stabilizing |
| Tablegen ABI is correct | Stabilizing | SUPPORT_TIERS | `cargo test -p adze-tablegen --all-features` | 0.8+ stabilizing |
| Structured parse errors are useful | Stabilizing | ADZE-SPEC-0005, SUPPORT_TIERS | `cargo test -p adze --test error_display_tests --features "pure-rust,glr" -- --nocapture` | 0.8+ stabilizing |
| Core table serialization round-trips | Stable | SUPPORT_TIERS | `cargo test -p adze-glr-core --features serialization` | 0.8+ stable |
| `parse_document()` exposes native document facts | Experimental | ADZE-SPEC-0003, SUPPORT_TIERS | `cargo test -p adze --features "pure-rust,ts-compat" --test adze_document_alpha -- --nocapture` | 0.9 foundation |
| Typed CST is generated over document nodes | Experimental | ADZE-SPEC-0004, SUPPORT_TIERS | `cargo test -p adze --features pure-rust --test typed_cst_generated_document -- --nocapture` | 0.9 foundation |
| Diagnostics are structured document facts | Stabilizing | ADZE-SPEC-0005, SUPPORT_TIERS | `cargo test -p adze --features "pure-rust,glr" --test generated_parse_errors -- --nocapture` | 0.9 stabilizing |
| `ts_compat` is a subset adapter | Advisory | ADZE-SPEC-0006, SUPPORT_TIERS | `cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_node_metadata -- --nocapture` | 0.9 advisory |
| `node_types_json()` is an advisory projection | Advisory | ADZE-SPEC-0010, SUPPORT_TIERS | `cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_language_metadata -- --nocapture` | 0.9 advisory |
| GLR ambiguity summaries are document facts | Experimental | ADZE-SPEC-0007, SUPPORT_TIERS | `cargo test -p adze --features "pure-rust,glr,runtime-e2e" --test test_e2e_ambiguous_grammar_glr -- --nocapture` | 0.9 experimental |
| JSON projection is schema-versioned | Experimental | ADZE-SPEC-0008, SUPPORT_TIERS | `cargo test -p adze --features "pure-rust,serialization" --test adze_document_json -- --nocapture` | 0.9 experimental |
| Incremental parsing exists but is not stable | Experimental | ADZE-SPEC-0009 | `cargo test --workspace --features incremental_glr` | future |
| WASM builds for core crates | Advisory | ADZE-SPEC-0008, SUPPORT_TIERS | `cargo check -p adze --target wasm32-unknown-unknown --features pure-rust` | advisory |

## What is not promised in 0.9

- Full GLR forest / raw ambiguity API
- Full Tree-sitter compatibility
- Stable AdzeDocument ABI
- Stable typed CST API
- Query predicate parity
- Production benchmark claims
- Stable incremental parsing
- Full error-tree parity beyond EOF
