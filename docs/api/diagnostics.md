# Diagnostics API

**Status:** Stabilizing
**Spec:** ADZE-SPEC-0005
**Proof:** `cargo test -p adze --test error_display_tests --features "pure-rust,glr" -- --nocapture`

Diagnostics are structured document facts attached to nodes and ranges.

## Accessing diagnostics

```rust
let doc = grammar::parse_document("1 + ")?;
for diag in doc.diagnostics() {
    println!("{}", diag);
}
```

## Diagnostic display

```
error: unexpected token
  --> input:1:5
   |
 1 | 1 +
   |     ^ expected expression
```

## Diagnostic structure

Each diagnostic carries:

- `severity` — error, warning, or note.
- `message` — human-readable description.
- `byte_range` — UTF-8 byte offsets into source.
- `point_range` — zero-based (row, column) range.
- `source_excerpt` — rendered source context.
- `expected` — list of expected tokens.
- `found` — the unexpected token.
- `node_id` — optional document node attachment.

## Error documents

Syntax errors produce a document with diagnostics and a best-effort tree:

```rust
let doc = grammar::parse_document("!!!invalid!!!")?;
assert!(doc.root().has_error());
assert!(!doc.diagnostics().is_empty());
```

## Not promised

- Full Tree-sitter error-tree parity.
- Non-EOF recovery shapes.
- Diagnostic codes or structured error taxonomy.
