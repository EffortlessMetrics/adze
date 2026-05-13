# ADZE-SPEC-0005: Diagnostics and recovery

Status: proposed
Owner: Adze maintainers
Created: 2026-05-13
Linked proposal: ../proposals/ADZE-PROP-0002-api-foundation.md
Linked ADRs: ADZE-ADR-0001 AdzeDocument one parse truth
Linked plan:
Linked issues:
Linked PRs:
Support-tier impact: stabilizing
Policy impact: none

## Problem

Parse diagnostics must be document facts — structured data attached to document
nodes and ranges — not separate error vectors that can drift from the native
tree. Without a clear diagnostic contract, error messages, byte spans, source
excerpts, and recovery state can be inconsistent across projections.

## Behavior

### Diagnostics are document facts

Each diagnostic is a structured record attached to the document, not a separate
error list produced by a different code path.

```rust
pub struct ParseDiagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub byte_range: ByteRange,
    pub point_range: PointRange,
    pub source_excerpt: Option<String>,
    pub expected: Vec<String>,
    pub found: Option<String>,
    pub node_id: Option<NodeId>,
}
```

### Rendered source excerpts are views

Source excerpts in diagnostics are views over document source text, not copied
strings. They are rendered on demand.

### Byte/point spans are the data contract

Diagnostics communicate location through byte ranges and zero-based point
ranges (row, column). These are the stable data contract. Display formatting
(human-readable line/column) is a view over these.

### Recovery creates tree facts

Error recovery inserts nodes into the document tree (e.g. missing children,
`ERROR` wrapper nodes). These are tree facts with `is_missing()` or `is_error()`
flags. They are not separate recovery state.

### Typed AST may reject recovered documents by default

By default, `doc.ast()` rejects documents where the root or critical children
are error or missing nodes. Users who want best-effort extraction must opt in.
Typed CST is lenient — it exposes whatever the document tree contains.

### Diagnostic lookup

The document supports diagnostic lookup by node ID and by range, enabling
editor and tooling integration.

### Diagnostic display

Diagnostics implement `Display` to produce human-readable output with source
context:

```
error: unexpected token
  --> input:3:5
   |
 3 | let x = 1 + ;
   |             ^ expected expression
```

### Severity levels

Diagnostics use a small severity vocabulary:

- **Error** — syntax error that prevented a complete parse.
- **Warning** — recovered construct that may indicate a user mistake.
- **Note** — contextual information attached to another diagnostic.

### EOF and truncated input

Missing children at EOF produce zero-width diagnostics at the end-of-input
position. The document still has a root node; the root's error flag reflects
the diagnostic state.

### Multibyte input

Byte ranges are UTF-8 byte offsets, not character offsets. Point ranges use
UTF-16 column counts (matching Tree-sitter convention) or UTF-8 byte columns
for native API. The data contract specifies which.

## Non-Goals

- Full Tree-sitter error-tree parity.
- Non-EOF recovery shapes (insertion, deletion, substitution).
- Diagnostic codes or structured error taxonomy.
- LSP diagnostic output (that is a projection, not a document fact).
- Imported grammar corpus compatibility.

## Required Evidence

- Diagnostics have byte ranges and point ranges for multiline input.
- Diagnostics display includes source excerpts.
- Multibyte input produces correct UTF-8 byte spans.
- EOF missing children produce zero-width diagnostics.
- Error documents still have root nodes with error flags.
- `doc.ast()` rejects error documents by default.
- Named-child filtering excludes anonymous aliases.

## Acceptance Examples

### Multibyte diagnostic

```rust
let doc = grammar::parse_document("x = \"hello\u{00e9}\u{0300}\"")?;
// ... or invalid input near multibyte chars
assert!(doc.diagnostics().iter().any(|d| d.byte_range.start > 0));
```

### EOF diagnostic

```rust
let doc = grammar::parse_document("1 + ")?;
let diag = doc.diagnostics().first().unwrap();
assert_eq!(diag.byte_range.end - diag.byte_range.start, 0); // zero-width
```

### Error document has root

```rust
let doc = grammar::parse_document("!!!invalid!!!")?;
assert!(doc.root().has_error());
```

## Test Mapping

- `cargo test -p adze --test error_display_tests`
- `cargo test -p adze --features "pure-rust,glr" --test generated_parse_errors`
- `cargo test -p adze --features pure-rust --test generated_parse_errors`

## Implementation Mapping

| Component | Owner |
| --- | --- |
| `ParseDiagnostic` | `runtime/src/document.rs` |
| `ByteRange`, `PointRange` | `runtime/src/document.rs` |
| Error display | `runtime/src/error.rs` |
| Recovery insertion | `runtime/src/parser_v4.rs` |

## CI Proof

```bash
cargo test -p adze --test error_display_tests --features "pure-rust,glr" -- --nocapture
cargo test -p adze --features "pure-rust,glr" --test generated_parse_errors -- --nocapture
just ci-supported
```

## Metrics And Promotion Rule

Promotion from stabilizing to stable requires:

- All supported grammars produce structured diagnostics for bad input.
- Byte spans are correct for multibyte input across all grammars.
- Display output includes source excerpts for all error types.
- No known diagnostic drift between `parse()` and `parse_document()`.
