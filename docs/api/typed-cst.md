# Typed CST API

**Status:** Experimental
**Spec:** ADZE-SPEC-0004
**Proof:** `cargo test -p adze --features pure-rust --test typed_cst_generated_document -- --nocapture`

Typed CST wrappers are generated over `AdzeDocument` nodes. They do not own a
second tree.

## Usage

```rust
let doc = grammar::parse_document("1 + 2")?;
let syntax: syntax::SourceFile = doc.syntax()?;
let root = syntax.root();

// Access typed fields
if let Some(binary) = root.as_binary_expr()? {
    assert_eq!(binary.left()?.text(), "1");
    assert_eq!(binary.operator()?, "+");
    assert_eq!(binary.right()?.text(), "2");
}
```

## Generated wrappers

Tablegen generates typed wrapper types for each named rule. Each wrapper holds
a document reference and a node ID:

```rust
pub struct BinaryExpr<'a> {
    doc: &'a AdzeDocument,
    node_id: NodeId,
}
```

Accessors resolve fields through native edge metadata. No source text is cloned.

## Not promised

- Visitor or rewriter API.
- Typed CST JSON schema.
- Broad typed CST / generic CST parity matrix.
- Stable API.
