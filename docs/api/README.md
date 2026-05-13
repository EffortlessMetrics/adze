# Adze API Documentation

This directory contains user-facing API documentation for Adze surfaces.

## Core thesis

Adze has one canonical parse product: `AdzeDocument`. Everything else is a
projection.

## Entry points

| Entry point | Status | Use when |
| --- | --- | --- |
| `parse::<T>(source)` | Stable | You want typed semantic values. |
| `parse_document(source)` | Experimental | You want the native parse product for tooling. |

## Projections

| Projection | Status | Use when |
| --- | --- | --- |
| `doc.ast::<T>()` | Experimental | You want typed AST from a document. |
| `doc.syntax()` | Experimental | You need source-preserving syntax. |
| `doc.diagnostics()` | Stabilizing | You need editor/CLI-quality errors. |
| `doc.as_tree_sitter()` | Advisory | You need ecosystem interop. |
| `doc.ambiguities()` | Experimental | GLR matters to your use case. |
| `doc.to_json_value()` | Experimental | You need serialized output. |
| `LanguageSchema` | Advisory | You need language metadata. |

## Documents

- [adze-document.md](adze-document.md) — AdzeDocument API and projections
- [typed-ast.md](typed-ast.md) — Typed AST extraction
- [typed-cst.md](typed-cst.md) — Typed CST wrappers
- [diagnostics.md](diagnostics.md) — Structured diagnostics
- [tree-sitter-compat.md](tree-sitter-compat.md) — Tree-sitter compatibility
- [glr-ambiguity.md](glr-ambiguity.md) — GLR ambiguity summaries
- [tablegen-abi.md](tablegen-abi.md) — Tablegen ABI

## Support tiers

See `docs/status/SUPPORT_TIERS.md` for the full feature-to-proof map.
See `docs/status/PRODUCT_PROOF_MAP.md` for the release-readable summary.
