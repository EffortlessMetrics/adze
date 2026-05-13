# ADZE-SPEC-0003: Canonical parse document

Status: proposed
Owner: Adze maintainers
Created: 2026-05-13
Linked proposal: ../proposals/ADZE-PROP-0002-api-foundation.md
Linked ADRs: ADZE-ADR-0001 AdzeDocument one parse truth
Linked plan:
Linked issues:
Linked PRs:
Support-tier impact: experimental → stabilizing when proof passes
Policy impact: none

## Problem

Adze needs a single canonical native parse product that all projections consume.
Without it, each surface (typed AST, typed CST, ts_compat, diagnostics, GLR
ambiguity, JSON) builds its own tree or error model, causing drift and proof
ambiguity.

## Behavior

### AdzeDocument is canonical

`AdzeDocument` is the single native parse product returned by
`parse_document()`. All other surfaces project from it.

```rust
pub struct AdzeDocument {
    source: String,
    root: ParseNode,
    node_index: Vec<NodeIndex>,
    diagnostics: Vec<ParseDiagnostic>,
    ambiguities: AmbiguitySet,
    metadata: ParseMetadata,
}
```

### Monomorphic

`AdzeDocument` is not generic over an AST type. Typed ASTs, typed CSTs, and
compatibility trees are extracted from it, not stored in it.

### parse() remains the stable front door

`grammar::parse::<T>(source)` is stable ergonomic sugar. Internally it calls
`parse_document()` and projects the typed AST. The public API does not change.

### parse_document() is the experimental boundary

`grammar::parse_document(source)` returns `Result<AdzeDocument>`. This is the
experimental native entry point for tool builders who need document-level
access.

### Syntax errors usually produce a document

A parse with errors should return a document containing diagnostics and a
best-effort tree, not an `Err` that discards partial results. An `Err` should
be reserved for cases where no useful document can be produced (e.g. encoding
errors, invalid language configuration).

### Tree storage

Tree storage moves toward direct node/edge storage:

- Nodes store visible identity (alias-adjusted kind, flags) and grammar
  identity (raw rule name/ID).
- Fields live on edges, not on nodes.
- `NodeId` is document-local and stable for the document lifetime.
- Parent, child, and sibling navigation uses node IDs.

### Projections are lazy where practical

Common paths (typed AST extraction via `parse()`) should not pay for typed CST,
JSON serialization, or full forest export unless explicitly requested. Lazy
projection is preferred when it does not compromise correctness.

### Document identity

Nodes expose two identity slots:

- **Visible identity** — the alias-adjusted kind name and ID used in S-expressions,
  `ts_compat` output, and user-facing queries.
- **Grammar identity** — the raw rule name and ID used for internal routing and
  tablegen ABI.

These are distinct. A node whose visible kind is `identifier` may have grammar
identity `rule_42`.

### Source text ownership

The document owns the source text. Projections borrow from it or extract text
ranges by value. Projections must not clone the entire source string.

## Non-Goals

- Stable AdzeDocument ABI or serialization schema.
- Raw GLR forest as the first native API.
- Per-AST-node provenance (document-level provenance first).
- Incremental document reuse (see ADZE-SPEC-0009).
- Full Tree-sitter node-types parity (see ADZE-SPEC-0010).

## Required Evidence

- `parse_document()` returns a document for valid input.
- `parse_document()` returns a diagnostic document for input with syntax errors.
- Document nodes have both visible and grammar identity.
- Fields are accessible on edges, not on nodes.
- `parse::<T>()` produces the same typed value as `doc.ast::<T>()`.
- `NodeId` lookups are O(1).

## Acceptance Examples

### Valid input produces a document

```rust
let doc = grammar::parse_document("1 + 2")?;
assert!(doc.diagnostics().is_empty());
assert!(doc.root().kind_id() > 0);
```

### Error input produces a diagnostic document

```rust
let doc = grammar::parse_document("1 + ")?;
assert!(!doc.diagnostics().is_empty());
assert!(doc.root().has_error());
```

### parse() is sugar over document

```rust
let ast: Expr = grammar::parse("1 + 2")?;
let doc = grammar::parse_document("1 + 2")?;
let ast2: Expr = doc.ast()?;
assert_eq!(format!("{ast:?}"), format!("{ast2:?}"));
```

## Test Mapping

- `cargo test -p adze --features "pure-rust,ts-compat" --test adze_document_alpha`
- `cargo test -p adze --features pure-rust --test typed_ast_contract`
- `cargo test -p adze --features pure-rust --test typed_cst_generated_document`

## Implementation Mapping

| Component | Owner |
| --- | --- |
| `AdzeDocument` struct | `runtime/src/document.rs` |
| `parse_document()` | `runtime/src/__private.rs`, `runtime/src/parser_v4.rs` |
| `NodeId`, `AdzeEdge`, `NodeIdentity` | `runtime/src/document.rs` |
| Tree construction | `runtime/src/parser_v4.rs` |

## CI Proof

```bash
cargo test -p adze --features "pure-rust,ts-compat" --test adze_document_alpha -- --nocapture
cargo test -p adze --features pure-rust --test typed_ast_contract -- --nocapture
cargo test -p adze --features pure-rust --test typed_cst_generated_document -- --nocapture
just ci-supported
```

## Metrics And Promotion Rule

Promotion from experimental to stabilizing requires:

- All required evidence tests pass in `ci-supported`.
- `parse()` and `parse_document()` produce consistent results for every
  generated grammar.
- Document node identity and field access have no known drift against
  `ts_compat`.
