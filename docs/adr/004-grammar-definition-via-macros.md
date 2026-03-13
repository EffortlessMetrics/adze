# ADR-004: Grammar Definition via Macros

## Status

Accepted

## Context

Parser generators traditionally use external DSLs (domain-specific languages) for grammar definition:

- **Yacc/Bison**: Separate `.y` files with custom syntax
- **ANTLR**: `.g4` files with grammar rules
- **Tree-sitter**: `grammar.js` JavaScript files

These approaches have drawbacks:

1. **External Files**: Grammar lives outside normal Rust source
2. **Build System Complexity**: External files require custom build rules
3. **Type Mismatch**: Grammar types don't match AST types—manual conversion required
4. **IDE Support**: Limited editor assistance for external DSLs
5. **Learning Curve**: Developers must learn a new syntax

### Alternatives Considered

1. **External Grammar Files**: Follow Tree-sitter's `grammar.js` pattern
2. **Builder API**: Fluent Rust API for grammar construction
3. **Derive Macros**: Generate grammar from struct/enum definitions only
4. **Attribute Macros**: Annotate Rust types with grammar metadata

## Decision

We chose **attribute macros on Rust types** as the primary grammar definition mechanism:

```rust
#[adze::grammar("arithmetic")]
pub mod grammar {
    #[adze::language]
    #[derive(Debug, PartialEq)]
    pub enum Expr {
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            i32
        ),

        #[adze::prec_left(1)]
        Sub(Box<Expr>, #[adze::leaf(text = "-")] (), Box<Expr>),

        #[adze::prec_left(2)]
        Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _ws: (),
    }
}
```

### Key Design Principles

1. **Grammar IS the AST**: The Rust type definitions become both the grammar and the output type
2. **Zero Boilerplate**: No manual tree walking or conversion code
3. **Full IDE Support**: rust-analyzer provides autocomplete, type checking, navigation
4. **Composable**: Standard Rust modules and visibility rules apply

### Macro Attributes

| Attribute | Purpose |
|-----------|---------|
| `#[adze::grammar("name")]` | Marks a module as containing grammar definitions |
| `#[adze::language]` | Marks the entry point type for parsing |
| `#[adze::leaf(pattern = "...")]` | Matches a regex pattern |
| `#[adze::leaf(text = "...")]` | Matches exact text |
| `#[adze::extra]` | Marks a type as skippable (whitespace, comments) |
| `#[adze::prec_left(n)]` | Left-associative operator with precedence n |
| `#[adze::prec_right(n)]` | Right-associative operator with precedence n |
| `#[adze::field(name)]` | Names a field for tree queries |

### Build-Time Processing

```
┌─────────────────────────────────────────────────────────────┐
│                    Source Code                               │
│         #[adze::grammar] mod grammar { ... }                │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    adze-macro                                │
│  - Parse attribute arguments                                 │
│  - Extract type definitions                                  │
│  - Generate grammar IR                                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    adze-tool (build.rs)                      │
│  - Read generated IR                                         │
│  - Generate parse tables                                     │
│  - Output optimized parser code                              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Generated Code                            │
│  - grammar::parse() function                                 │
│  - Type extraction logic                                     │
│  - Error handling                                            │
└─────────────────────────────────────────────────────────────┘
```

## Consequences

### Positive

- **Type Safety**: Parse output is guaranteed to match the grammar type
- **No DSL Learning**: Developers use familiar Rust syntax
- **IDE Integration**: Full rust-analyzer support including go-to-definition
- **Refactoring Support**: Rename refactorings update grammar automatically
- **Minimal Boilerplate**: The grammar definition IS the AST definition
- **Compile-Time Checks**: Invalid grammars fail at compile time

### Negative

- **Attribute Complexity**: Complex grammars can have deeply nested attributes
- **Macro Expansion Errors**: Error messages can be cryptic during macro expansion
- **Build Time**: Proc-macro processing adds to compilation time
- **Limited Expressiveness**: Some grammar patterns are hard to express in Rust types
- **Debugging Difficulty**: Generated code is hard to inspect

### Neutral

- **Build Script Required**: Projects need a `build.rs` to invoke `adze-tool`
- **Separate Crate**: Macro and tool are separate crates for compilation reasons
- **Incremental Compilation**: Changes to grammar trigger recompilation of dependents

## Related

- Related ADRs: [ADR-001](001-pure-rust-glr-implementation.md)
- Reference: [macro/src/lib.rs](../../macro/src/lib.rs) - Macro implementation
- Tutorial: [docs/tutorials/getting-started.md](../tutorials/getting-started.md)
