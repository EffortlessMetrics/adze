# ADZE-SPEC-0010: Language metadata and node-types

Status: proposed
Owner: Adze maintainers
Created: 2026-05-13
Linked proposal: ../proposals/ADZE-PROP-0002-api-foundation.md
Linked ADRs: ADZE-ADR-0001 AdzeDocument one parse truth
Linked plan:
Linked issues:
Linked PRs:
Support-tier impact: advisory
Policy impact: none

## Problem

Language metadata — node types, field names, symbol IDs, and alias mappings —
must be projections of generated grammar data, not separate registries. Without
a clear contract, `node_types_json()` could drift from actual parse behavior,
and alias metadata could be inconsistent between the adapter and the native
document.

## Behavior

### LanguageSchema is first-class

Generated language metadata is collected into a `LanguageSchema` structure that
is available at runtime. It is not a separate build artifact.

```rust
pub struct LanguageSchema {
    pub name: &'static str,
    pub node_types: Vec<NodeTypeInfo>,
    pub fields: Vec<FieldInfo>,
    pub aliases: Vec<AliasMapping>,
}
```

### Generated constants exist

Node kind IDs, field IDs, and symbol constants are generated at build time and
available as `const` values in the generated parser module. They do not require
runtime computation.

### node-types JSON is a projection

`Language::node_types_json()` is an advisory projection of `LanguageSchema`
into Tree-sitter-compatible JSON format. It is useful for editor integration
but is not a stable contract.

The projection:

- Lists named node types with their fields and children.
- Lists anonymous types (tokens) separately.
- Includes alias-visible names where applicable.
- Excludes hidden rules.

### Alias metadata is explicit

Alias mappings are recorded explicitly in the language schema. Each alias maps
a grammar identity to a visible identity:

```rust
pub struct AliasMapping {
    pub grammar_name: &'static str,
    pub grammar_id: u16,
    pub visible_name: &'static str,
    pub visible_id: u16,
    pub is_named: bool,
}
```

Alias metadata is used by:

- `ts_compat` to project alias-visible identity.
- `node_types_json()` to list visible types.
- S-expression output to render alias-visible names.

### node-types JSON is advisory

`node_types_json()` output is not guaranteed to be fully compatible with
Tree-sitter's `node-types.json`. Known gaps include:

- Alias-visible node-types/query-compatible alias metadata.
- Full field parity with Tree-sitter output.
- Supertype/subtype relationships.

These gaps are explicitly documented and not promised for 0.9.

### Metadata is grammar-local

Language metadata is tied to a specific grammar fingerprint. Different grammar
versions produce different metadata. Metadata does not persist across grammar
changes.

## Non-Goals

- Full Tree-sitter node-types.json parity.
- Query-compatible alias metadata.
- Supertype/subtype declarations.
- Stable JSON schema for metadata output.
- Cross-grammar metadata compatibility.

## Required Evidence

- `node_types_json()` output includes named node types and their fields.
- Alias mappings are present in the language schema.
- Generated node kind IDs are stable for a given grammar.
- `node_types_json()` excludes hidden rules.

## Acceptance Examples

### node-types JSON output

```rust
let lang = grammar::language();
let json = lang.node_types_json();
assert!(json.contains("\"type\":"));
assert!(json.contains("\"named\":"));
```

### Alias mapping present

```rust
let schema = grammar::language_schema();
assert!(!schema.aliases.is_empty() || schema.aliases.is_empty()); // depends on grammar
```

## Test Mapping

- `cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_node_metadata`
- `cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_language_fields`
- `cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_language_metadata`

## Implementation Mapping

| Component | Owner |
| --- | --- |
| `LanguageSchema` | `runtime/src/document.rs` |
| `node_types_json()` | `runtime/src/language.rs` |
| Alias mappings | `tablegen/src/` |
| Generated constants | `tool/src/` |

## CI Proof

```bash
cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_language_metadata -- --nocapture
cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_node_metadata -- --nocapture
just ci-supported
```

## Metrics And Promotion Rule

Promotion from advisory to stabilizing requires:

- `node_types_json()` output is validated against at least one editor
  integration.
- Alias mappings cover all known production alias sequences.
- Field metadata matches native edge metadata for all supported grammars.
