# Grammar Definition

This chapter covers how to define grammars in Rust-Sitter using Rust's type system.

## Grammar Module

Every Rust-Sitter grammar starts with a module annotated with `#[rust_sitter::grammar]`:

```rust
#[rust_sitter::grammar("my_language")]
mod grammar {
    // Grammar definitions go here
}
```

The string parameter becomes the language name used by Tree-sitter.

## Language Root

Mark the entry point of your grammar with `#[rust_sitter::language]`:

```rust
#[rust_sitter::language]
pub struct Program {
    pub statements: Vec<Statement>,
}
```

Only one type should be marked as the language root.

## Node Types

### Structs

Use structs for nodes with a fixed structure:

```rust
#[rust_sitter::language]
pub struct BinaryOp {
    pub left: Expression,
    pub operator: Operator,
    pub right: Expression,
}
```

### Enums

Use enums for nodes with alternatives:

```rust
#[rust_sitter::language]
pub enum Statement {
    Assignment(Assignment),
    Expression(Expression),
    Return(ReturnStatement),
}
```

## Leaf Nodes

Leaf nodes represent terminal symbols (tokens) in your grammar.

### Pattern Matching

Use regular expressions to match tokens:

```rust
pub struct Identifier {
    #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
    pub name: (),
}
```

### Exact Text

Match specific strings:

```rust
pub struct Plus {
    #[rust_sitter::leaf(text = "+")]
    _plus: (),
}
```

### Transformations

Transform matched text into Rust types:

```rust
pub struct Number {
    #[rust_sitter::leaf(
        pattern = r"\d+", 
        transform = |s| s.parse().unwrap()
    )]
    pub value: u32,
}
```

## Repetitions

### Vectors

Use `Vec` with `#[rust_sitter::repeat]` for zero or more items:

```rust
pub struct Block {
    #[rust_sitter::repeat]
    pub statements: Vec<Statement>,
}
```

### Non-Empty Vectors

For one or more items:

```rust
pub struct ParameterList {
    #[rust_sitter::repeat(non_empty = true)]
    pub params: Vec<Parameter>,
}
```

### Separators

Add separators between repeated items:

```rust
pub struct ArgumentList {
    #[rust_sitter::repeat(separator = ",")]
    pub args: Vec<Expression>,
}
```

## Optional Fields

Use `Option` for optional elements:

```rust
pub struct Function {
    pub name: Identifier,
    pub params: Option<ParameterList>,
    pub return_type: Option<Type>,
}
```

## Precedence and Associativity

Control how ambiguous grammars are parsed:

### Left Associativity

```rust
#[rust_sitter::prec_left(1)]
pub struct Add {
    pub left: Box<Expression>,
    #[rust_sitter::leaf(text = "+")] _op: (),
    pub right: Box<Expression>,
}
```

### Right Associativity

```rust
#[rust_sitter::prec_right(1)]
pub struct Power {
    pub base: Box<Expression>,
    #[rust_sitter::leaf(text = "^")] _op: (),
    pub exponent: Box<Expression>,
}
```

### Non-Associative

```rust
#[rust_sitter::prec(1)]
pub struct Compare {
    pub left: Box<Expression>,
    pub op: CompareOp,
    pub right: Box<Expression>,
}
```

Higher precedence numbers bind more tightly.

## Extra Tokens

Define tokens that are automatically skipped:

```rust
#[rust_sitter::extra]
pub enum Extra {
    Whitespace(Whitespace),
    Comment(Comment),
}

pub struct Whitespace {
    #[rust_sitter::leaf(pattern = r"\s+")]
    _ws: (),
}

pub struct Comment {
    #[rust_sitter::leaf(pattern = r"//[^\n]*")]
    _comment: (),
}
```

## Field Names

Named fields in the generated Tree-sitter grammar:

```rust
pub struct Assignment {
    #[rust_sitter::field("left")]
    pub target: Identifier,
    
    #[rust_sitter::leaf(text = "=")] 
    _eq: (),
    
    #[rust_sitter::field("right")]
    pub value: Expression,
}
```

## Advanced Patterns

### Inline Rules

For simple alternatives without creating a separate type:

```rust
pub struct Statement {
    #[rust_sitter::leaf(pattern = r"(let|const|var)")]
    pub keyword: (),
    pub declaration: Declaration,
}
```

### Complex Tokens

For tokens that need more complex matching:

```rust
pub struct StringLiteral {
    #[rust_sitter::leaf(
        pattern = r#""([^"\\]|\\.)*""#,
        transform = |s| {
            s[1..s.len()-1]
                .replace("\\n", "\n")
                .replace("\\t", "\t")
                .replace("\\\"", "\"")
        }
    )]
    pub value: String,
}
```

## Best Practices

1. **Start Simple**: Begin with basic tokens and build up
2. **Test Incrementally**: Add tests for each new rule
3. **Avoid Deep Nesting**: Use separate types for clarity
4. **Document Complex Rules**: Add comments explaining regex patterns
5. **Use Meaningful Names**: Field names appear in the AST

## Common Pitfalls

### Left Recursion

Avoid direct left recursion:

```rust
// ❌ Bad - causes infinite recursion
pub enum List {
    Cons(Box<List>, Item),  // Left recursive!
    Nil,
}

// ✅ Good - use Vec instead
pub struct List {
    #[rust_sitter::repeat]
    pub items: Vec<Item>,
}
```

### Ambiguity

Be explicit about precedence:

```rust
// ❌ Bad - ambiguous for "1 + 2 * 3"
pub enum Expr {
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

// ✅ Good - explicit precedence
#[rust_sitter::prec_left(1)]
Add(...),
#[rust_sitter::prec_left(2)]
Mul(...),
```

## Next Steps

- Learn about [Parser Generation](parser-generation.md) to understand how grammars become parsers
- Explore [Query and Pattern Matching](query-patterns.md) for analyzing parsed trees
- Read about [Performance Optimization](performance.md) for large grammars