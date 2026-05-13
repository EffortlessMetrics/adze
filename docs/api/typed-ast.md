# Typed AST Extraction

**Status:** Stable (via `parse()`), Experimental (via `doc.ast()`)
**Spec:** ADZE-SPEC-0004
**Proof:** `cargo test -p adze --features pure-rust --test typed_ast_contract -- --nocapture`

## Stable entry point

```rust
let ast: Expr = grammar::parse("1 + 2")?;
```

This is the stable ergonomic front door. It produces typed semantic values
directly.

## Experimental projection

```rust
let doc = grammar::parse_document("1 + 2")?;
let ast: Expr = doc.ast()?;
```

Both entry points produce the same typed value. `parse()` is sugar over
`parse_document()` + AST projection.

## Provenance

```rust
let doc = grammar::parse_document("1 + 2")?;
let result: AstWithProvenance<Expr> = doc.ast_with_provenance()?;
println!("extracted from node {:?}", result.provenance);
```

## Error handling

By default, `doc.ast()` rejects documents with unrecovered errors:

```rust
let doc = grammar::parse_document("1 + ")?;
assert!(doc.ast::<Expr>().is_err()); // strict by default
```

## Not promised

- Per-AST-node provenance (document-level provenance first).
- Best-effort extraction from partial parses without explicit opt-in.
