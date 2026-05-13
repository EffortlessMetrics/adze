# ADZE-SPEC-0010: Language metadata and node-types

Status: accepted
Owner: tablegen/runtime/ts-compat
Created: 2026-05-13
Linked proposal: ../proposals/ADZE-PROP-0002-api-foundation.md
Linked ADRs: ../adr/ADZE-ADR-0001-adze-document-one-parse-truth.md
Linked plan: ../../plans/0.9.0/api-foundation.md
Linked issues:
Linked PRs:
Support-tier impact: ../status/SUPPORT_TIERS.md
Policy impact: ../../policy/doc-artifacts.toml

## Problem

Typed CST, Tree-sitter compatibility, query compatibility, node-types JSON, and
schema-versioned output all need stable language metadata. Metadata must be a
first-class generated artifact, not reconstructed from one runtime view.

## Behavior

### B1. Language schema is first-class

Generated languages must expose enough metadata for symbols, fields, aliases,
supertypes, node types, public/grammar identity, and grammar fingerprints.

### B2. Generated constants exist

Generated modules should expose stable constants or tables for kinds, fields,
aliases, and node-types metadata where supported.

### B3. node-types JSON is a projection

`node-types.json` is generated from language metadata, not reconstructed from
runtime trees.

### B4. Alias metadata is explicit

Language metadata must carry enough information to answer visible identity and
grammar identity separately.

### B5. Metadata feeds every projection

Typed CST casts, field accessors, `ts_compat::Language`, node-types JSON, query
compatibility, and document JSON should all read language metadata from the same
source.

## Non-Goals

- No full query engine in this spec.
- No imported grammar parity guarantee.
- No Tree-sitter C ABI stability guarantee.

## Required Evidence

- Field ID/name lookup canary.
- Alias metadata canary.
- Node-types JSON snapshot or structural canary.
- Typed CST field accessors use generated field IDs.
- `ts_compat` language metadata maps to the same schema.

## Acceptance Examples

```rust
let schema = grammar::language_schema();
let left = schema.field_id_for_name("left").unwrap();
assert_eq!(schema.field_name(left), Some("left"));
```

```rust
let node_types = grammar::node_types_json();
assert!(node_types.contains("fields"));
```

## Test Mapping

- `adze-tablegen` node-types tests;
- typed CST generator tests;
- `runtime/tests/ts_compat_node_types.rs`;
- language metadata tests.

## Implementation Mapping

Primary implementation surfaces:

- `tablegen`;
- generated grammar modules;
- `runtime/src/document/language*`;
- `runtime/src/ts_compat/language*`;
- node-types JSON output.

## CI Proof

```bash
cargo test -p adze-tablegen node_types -- --nocapture
cargo test -p adze-tablegen typed_cst -- --nocapture
cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_node_types -- --nocapture
git diff --check
```

## Metrics / Promotion Rule

Language metadata and node-types remain advisory until field IDs, aliases,
node-types, typed CST, and Tree-sitter compatibility all prove they project the
same schema.
