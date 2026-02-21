# Grammar Definition

This chapter covers how to define grammars in Adze using Rust's type system.

## Grammar Module

Every Adze grammar starts with a module annotated with `#[adze::grammar]`:

```rust
#[adze::grammar("my_language")]
mod grammar {
    // Grammar definitions go here
}
```

The string parameter becomes the language name used by Tree-sitter.

## Language Root

Mark the entry point of your grammar with `#[adze::language]`:

```rust
#[adze::language]
pub struct Program {
    pub statements: Vec<Statement>,
}
```

Only one type should be marked as the language root.

## Node Types

### Structs

Use structs for nodes with a fixed structure:

```rust
#[adze::language]
pub struct BinaryOp {
    pub left: Expression,
    pub operator: Operator,
    pub right: Expression,
}
```

### Enums

Use enums for nodes with alternatives:

```rust
#[adze::language]
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
    #[adze::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
    pub name: (),
}
```

### Exact Text

Match specific strings:

```rust
pub struct Plus {
    #[adze::leaf(text = "+")]
    _plus: (),
}
```

### Transformations

Transform matched text into Rust types:

```rust
pub struct Number {
    #[adze::leaf(
        pattern = r"\d+", 
        transform = |s| s.parse().unwrap()
    )]
    pub value: u32,
}
```

## Repetitions

### Vectors

Use `Vec` with `#[adze::repeat]` for zero or more items:

```rust
pub struct Block {
    #[adze::repeat]
    pub statements: Vec<Statement>,
}
```

### Non-Empty Vectors

For one or more items:

```rust
pub struct ParameterList {
    #[adze::repeat(non_empty = true)]
    pub params: Vec<Parameter>,
}
```

### Separators

Add separators between repeated items:

```rust
pub struct ArgumentList {
    #[adze::repeat(separator = ",")]
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
#[adze::prec_left(1)]
pub struct Add {
    pub left: Box<Expression>,
    #[adze::leaf(text = "+")] _op: (),
    pub right: Box<Expression>,
}
```

### Right Associativity

```rust
#[adze::prec_right(1)]
pub struct Power {
    pub base: Box<Expression>,
    #[adze::leaf(text = "^")] _op: (),
    pub exponent: Box<Expression>,
}
```

### Non-Associative

```rust
#[adze::prec(1)]
pub struct Compare {
    pub left: Box<Expression>,
    pub op: CompareOp,
    pub right: Box<Expression>,
}
```

Higher precedence numbers bind more tightly.

### Precedence Values

- **Valid Range**: `0` to `4294967295` (u32 range)
- **Zero is Valid**: `#[adze::prec(0)]` is a valid precedence level
- **Integer Literals Only**: Must use literal integers, not variables or expressions

```rust
// ✅ Valid precedence values
#[adze::prec(0)]        // Lowest precedence
#[adze::prec(100)]      // Medium precedence
#[adze::prec(4294967295)] // Highest precedence

// ❌ Invalid - will produce clear error messages
#[adze::prec("high")]   // String instead of integer
#[adze::prec(3.14)]     // Float instead of integer
#[adze::prec(HIGH_PREC)] // Variable instead of literal
#[adze::prec(-1)]       // Negative number
#[adze::prec(4294967296)] // Too large for u32
```

### Precedence Error Handling

The grammar processor provides comprehensive error messages for common precedence mistakes:

#### Multiple Precedence Attributes

Only one precedence attribute can be used per rule:

```rust
// ❌ Error: Multiple precedence attributes
#[adze::prec(1)]
#[adze::prec_left(2)]
pub struct Conflict {
    // This will produce error:
    // "only one of prec, prec_left, and prec_right can be specified, 
    //  but found: prec, prec_left"
}
```

#### Invalid Precedence Values

Non-integer or out-of-range values produce specific error messages:

```rust
// ❌ Error: String literal instead of integer
#[adze::prec("high")]
pub struct StringPrec {
    // Error: "Expected integer literal for precedence. 
    //         Use #[adze::prec(123)] with a positive integer (0 to 4294967295)."
}

// ❌ Error: Float literal instead of integer  
#[adze::prec_left(3.14)]
pub struct FloatPrec {
    // Error: "Expected integer literal for left-associative precedence. 
    //         Use #[adze::prec_left(123)] with a positive integer (0 to 4294967295)."
}
```

#### Troubleshooting Precedence Errors

When you encounter precedence errors:

1. **Check for Multiple Attributes**: Remove conflicting precedence attributes
2. **Use Integer Literals**: Replace strings, floats, or variables with integer literals  
3. **Validate Range**: Ensure values are between 0 and 4294967295
4. **Review Compiler Output**: Error messages include the specific attributes found and expected formats

## Extra Tokens

Define tokens that are automatically skipped:

```rust
#[adze::extra]
pub enum Extra {
    Whitespace(Whitespace),
    Comment(Comment),
}

pub struct Whitespace {
    #[adze::leaf(pattern = r"\s+")]
    _ws: (),
}

pub struct Comment {
    #[adze::leaf(pattern = r"//[^\n]*")]
    _comment: (),
}
```

## Field Names

Named fields in the generated Tree-sitter grammar:

```rust
pub struct Assignment {
    #[adze::field("left")]
    pub target: Identifier,
    
    #[adze::leaf(text = "=")] 
    _eq: (),
    
    #[adze::field("right")]
    pub value: Expression,
}
```

## Advanced Patterns

### Inline Rules

For simple alternatives without creating a separate type:

```rust
pub struct Statement {
    #[adze::leaf(pattern = r"(let|const|var)")]
    pub keyword: (),
    pub declaration: Declaration,
}
```

### Complex Tokens

For tokens that need more complex matching:

```rust
pub struct StringLiteral {
    #[adze::leaf(
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
    #[adze::repeat]
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
#[adze::prec_left(1)]
Add(...),
#[adze::prec_left(2)]
Mul(...),
```

## Next Steps

- Learn about [Parser Generation](parser-generation.md) to understand how grammars become parsers
- Explore [Query and Pattern Matching](query-patterns.md) for analyzing parsed trees
- Read about [Performance Optimization](performance.md) for large grammars