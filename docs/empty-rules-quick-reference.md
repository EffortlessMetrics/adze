# Empty Rules Quick Reference

## ❌ DON'T: Structs with only Vec fields

```rust
// This will fail!
pub struct Module {
    #[rust_sitter::repeat]
    pub statements: Vec<Statement>,
}
```

## ✅ DO: Add non_empty constraint

```rust
pub struct Module {
    #[rust_sitter::repeat(non_empty = true)]
    pub statements: Vec<Statement>,
}
```

## ✅ DO: Add whitespace tokens for containers

```rust
pub struct ListExpression {
    #[rust_sitter::leaf(text = "[")]
    _open: (),
    #[rust_sitter::leaf(pattern = r"\s*")]
    #[rust_sitter::skip]
    _ws: (),
    #[rust_sitter::repeat]
    pub elements: Vec<Expression>,
    #[rust_sitter::leaf(text = "]")]
    _close: (),
}
```

## ✅ DO: Use enums for optional parts

```rust
pub enum Name {
    Simple(Identifier),
    Qualified {
        first: Identifier,
        #[rust_sitter::repeat(non_empty = true)]
        rest: Vec<NamePart>,
    }
}
```

## 🔍 Debug with:

```bash
RUST_SITTER_EMIT_ARTIFACTS=true cargo build
# Check target/debug/build/*/out/grammar.json
```

## 💡 Remember:
- Every grammar rule must consume at least one token
- Empty Vecs create empty rules
- Tree-sitter will reject grammars with empty rules at generation time