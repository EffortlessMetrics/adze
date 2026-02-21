# Empty Rules Quick Reference

## ❌ DON'T: Structs with only Vec fields

```rust
// This will fail!
pub struct Module {
    #[adze::repeat]
    pub statements: Vec<Statement>,
}
```

## ✅ DO: Add non_empty constraint

```rust
pub struct Module {
    #[adze::repeat(non_empty = true)]
    pub statements: Vec<Statement>,
}
```

## ✅ DO: Add whitespace tokens for containers

```rust
pub struct ListExpression {
    #[adze::leaf(text = "[")]
    _open: (),
    #[adze::leaf(pattern = r"\s*")]
    #[adze::skip]
    _ws: (),
    #[adze::repeat]
    pub elements: Vec<Expression>,
    #[adze::leaf(text = "]")]
    _close: (),
}
```

## ✅ DO: Use enums for optional parts

```rust
pub enum Name {
    Simple(Identifier),
    Qualified {
        first: Identifier,
        #[adze::repeat(non_empty = true)]
        rest: Vec<NamePart>,
    }
}
```

## 🔍 Debug with:

```bash
ADZE_EMIT_ARTIFACTS=true cargo build
# Check target/debug/build/*/out/grammar.json
```

## 💡 Remember:
- Every grammar rule must consume at least one token
- Empty Vecs create empty rules
- Tree-sitter will reject grammars with empty rules at generation time