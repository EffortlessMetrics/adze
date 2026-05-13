# AdzeDocument API

**Status:** Experimental
**Spec:** ADZE-SPEC-0003
**Proof:** `cargo test -p adze --features "pure-rust,ts-compat" --test adze_document_alpha -- --nocapture`

`AdzeDocument` is the canonical native parse product. All other surfaces
project from it.

## Construction

```rust
// Stable ergonomic entry point (sugar over parse_document)
let ast: Expr = grammar::parse("1 + 2")?;

// Experimental native entry point
let doc: AdzeDocument = grammar::parse_document("1 + 2")?;
```

## Projections

```rust
let doc = grammar::parse_document("1 + 2")?;

// Typed AST extraction
let ast: Expr = doc.ast()?;

// Generic CST access
let root = doc.root();
println!("root kind: {}", root.kind());

// Structured diagnostics
for diag in doc.diagnostics() {
    println!("{}: {}", diag.severity, diag.message);
}

// Tree-sitter compatibility
let ts_tree = ts_compat::Tree::from_document(&doc);

// GLR ambiguity
for site in doc.ambiguities() {
    println!("ambiguous at {:?}: {} alternatives", site.node_id, site.alternatives.len());
}

// JSON serialization (requires `serialization` feature)
let json = doc.to_json_value()?;
```

## Error handling

```rust
// Syntax errors usually produce a document with diagnostics
let doc = grammar::parse_document("1 + ")?;
assert!(!doc.diagnostics().is_empty());
assert!(doc.root().has_error());

// Default AST extraction rejects error documents
assert!(doc.ast::<Expr>().is_err());
```

## Not promised

- Stable ABI or serialization schema.
- Per-AST-node provenance.
- Full Tree-sitter node-types parity.
- Incremental document reuse.
