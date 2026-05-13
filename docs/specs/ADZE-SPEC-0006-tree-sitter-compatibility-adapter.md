# ADZE-SPEC-0006: Tree-sitter compatibility adapter

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

Tree-sitter compatibility (`ts_compat`) is an adoption adapter that lets Adze
users leverage existing editor tooling, query files, and ecosystem conventions.
It must project from native document data, not invent its own semantics. Without
a clear adapter contract, `ts_compat` could drift from the native document or
expose semantics that do not exist in the source of truth.

## Behavior

### ts_compat depends on document data

`ts_compat::Tree` is constructed from `AdzeDocument`, not from a separate parse
pass. It reads node identity, field metadata, and tree structure from the
document.

```rust
let doc = grammar::parse_document(source)?;
let ts_tree = ts_compat::Tree::from_document(&doc);
```

### Adapter does not invent semantics

`ts_compat` exposes only semantics that exist in the native document. It does
not:

- Create node kinds that do not exist in the grammar.
- Invent field assignments that the document does not record.
- Report different error state than the document's diagnostics.
- Expose ambiguity information (the adapter sees the selected tree only).

### kind/kind_id and grammar_name/grammar_id are separate

Node identity in `ts_compat` exposes two slots:

- **kind() / kind_id()** — alias-visible identity (the name shown in
  S-expressions and user-facing queries).
- **grammar_name() / grammar_id()** — raw grammar identity (the rule name/ID
  before alias resolution).

These are distinct and both come from document `NodeIdentity`.

### Fields are edge-based

Field access in `ts_compat` resolves through native edge metadata, not through
a separate field registry. Field names and IDs are projected from `AdzeEdge`.

### Adapter exposes selected tree only

`ts_compat::Tree` shows the selected parse tree. It does not expose GLR
alternatives, ambiguity summaries, or forest internals. Those are native
document facts, not compatibility adapter facts.

### Node types JSON is a projection

`Language::node_types_json()` is an advisory projection of grammar metadata,
not a separate registry. It is useful for editor integration but is not a
stable contract.

### Query parity is not promised

`ts_compat` does not promise full compatibility with Tree-sitter `.scm` query
files. Query parity is explicitly out of scope for 0.9.

### Named-child filtering

Named-child iteration filters out anonymous aliases and hidden nodes, matching
Tree-sitter convention. Anonymous aliases are visible in `kind()` but excluded
from `named_child_count()` and `children_by_field_name()`.

### Missing and error nodes

Missing children (from EOF recovery) are represented as zero-width `ERROR`
nodes with `is_missing() == true` and `is_error() == true`. This matches
Tree-sitter convention for the EOF case. Broader error-tree parity is not
promised.

## Non-Goals

- Full Tree-sitter API parity.
- Query predicate compatibility.
- Parser-generated error-tree parity beyond EOF.
- Imported grammar corpus compatibility.
- node_types.json full parity.
- Alias-visible node-types/query-compatible alias metadata.

## Required Evidence

- `Tree::from_document(&doc)` produces a tree whose node count matches the
  document.
- kind() returns alias-visible names; grammar_name() returns raw names.
- Field access resolves through edge metadata.
- Named-child filtering excludes anonymous aliases.
- Missing children at EOF produce zero-width ERROR nodes.
- S-expression output uses alias-visible identity.

## Acceptance Examples

### Adapter from document

```rust
let doc = grammar::parse_document("1 + 2")?;
let ts_tree = ts_compat::Tree::from_document(&doc);
let root = ts_tree.root_node();
assert_eq!(root.child_count(), 3);
```

### Separate identity

```rust
let root = ts_tree.root_node();
assert_ne!(root.kind(), root.grammar_name()); // if aliased
```

### Named-child filtering

```rust
let root = ts_tree.root_node();
assert!(root.named_child_count() <= root.child_count());
```

## Test Mapping

- `cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_tree_children`
- `cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_node_metadata`
- `cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_to_sexp`
- `cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_node_error`

## Implementation Mapping

| Component | Owner |
| --- | --- |
| `ts_compat::Tree` | `runtime/src/ts_compat/` |
| `NodeIdentity` | `runtime/src/document.rs` |
| Field projection | `runtime/src/ts_compat/` |
| S-expression output | `runtime/src/ts_compat/` |

## CI Proof

```bash
cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_tree_children -- --nocapture
cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_node_metadata -- --nocapture
cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_to_sexp -- --nocapture
just ci-supported
```

## Metrics And Promotion Rule

Promotion from advisory to stabilizing requires:

- `from_document()` roundtrip preserves all node identity and field metadata.
- Named-child filtering matches Tree-sitter behavior for all supported grammars.
- S-expression output matches Tree-sitter output for a representative corpus.
- node_types_json() output is validated against at least one editor integration.
