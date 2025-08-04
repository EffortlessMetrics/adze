# Quick Start

This guide will walk you through creating your first Rust-Sitter grammar and parser.

## Creating a Simple Arithmetic Parser

Let's build a parser for basic arithmetic expressions like `1 + 2 * 3`.

### Step 1: Define the Grammar

Create a new Rust project and add this to your `src/main.rs`:

```rust
#[rust_sitter::grammar("arithmetic")]
mod grammar {
    #[rust_sitter::language]
    pub enum Expr {
        Number(
            #[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32,
        ),
        
        #[rust_sitter::prec_left(1)]
        Add(
            Box<Expr>,
            #[rust_sitter::leaf(text = "+")] (),
            Box<Expr>,
        ),
        
        #[rust_sitter::prec_left(2)]
        Multiply(
            Box<Expr>,
            #[rust_sitter::leaf(text = "*")] (),
            Box<Expr>,
        ),
        
        Parenthesized(
            #[rust_sitter::leaf(text = "(")] (),
            Box<Expr>,
            #[rust_sitter::leaf(text = ")")] (),
        ),
    }
    
    // Define whitespace as extra (automatically skipped)
    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

fn main() {
    // Parse some expressions
    let expressions = vec![
        "42",
        "1 + 2",
        "1 + 2 * 3",
        "(1 + 2) * 3",
    ];
    
    for expr in expressions {
        println!("Parsing: {}", expr);
        match grammar::parse(expr) {
            Ok(ast) => println!("  Result: {:?}\n", ast),
            Err(e) => println!("  Error: {}\n", e),
        }
    }
}
```

### Step 2: Run Your Parser

```bash
cargo run
```

You should see output like:

```
Parsing: 42
  Result: Number(42)

Parsing: 1 + 2
  Result: Add(Number(1), (), Number(2))

Parsing: 1 + 2 * 3
  Result: Add(Number(1), (), Multiply(Number(2), (), Number(3)))

Parsing: (1 + 2) * 3
  Result: Multiply(Parenthesized((), Add(Number(1), (), Number(2)), ()), (), Number(3))
```

## Understanding the Grammar

Let's break down what each part does:

### The Grammar Module

```rust
#[rust_sitter::grammar("arithmetic")]
mod grammar { ... }
```

This creates a grammar named "arithmetic" and generates a `parse` function.

### The Language Root

```rust
#[rust_sitter::language]
pub enum Expr { ... }
```

This marks `Expr` as the root type that the parser will return.

### Leaf Nodes

```rust
#[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
u32,
```

- `pattern`: Regular expression to match
- `transform`: Function to convert matched text to the field type
- `text`: Exact string to match (alternative to `pattern`)

### Precedence

```rust
#[rust_sitter::prec_left(1)]  // Lower precedence
#[rust_sitter::prec_left(2)]  // Higher precedence
```

Higher numbers bind more tightly. Multiplication (2) binds tighter than addition (1).

### Extra Tokens

```rust
#[rust_sitter::extra]
struct Whitespace { ... }
```

Tokens marked as `extra` are automatically skipped between other tokens.

## Common Patterns

### Optional Fields

```rust
pub struct Function {
    pub name: Identifier,
    pub params: Option<Parameters>,
}
```

### Repeated Elements

```rust
pub struct Block {
    #[rust_sitter::repeat]
    pub statements: Vec<Statement>,
}
```

### Delimited Lists

```rust
pub struct ArgList {
    #[rust_sitter::repeat(separator = ",")]
    pub args: Vec<Expression>,
}
```

### String Literals

```rust
pub struct StringLiteral {
    #[rust_sitter::leaf(pattern = r#""([^"\\]|\\.)*""#, transform = |s| s[1..s.len()-1].to_string())]
    pub value: String,
}
```

## Error Handling

Rust-Sitter provides detailed error messages:

```rust
match grammar::parse("1 + + 2") {
    Ok(ast) => println!("Parsed: {:?}", ast),
    Err(e) => {
        println!("Error at position {}: {}", e.position, e.message);
        println!("Expected one of: {:?}", e.expected);
    }
}
```

## Next Steps

Now that you've created your first grammar:

1. Read about [Grammar Definition](../guide/grammar-definition.md) for more complex patterns
2. Learn about [Incremental Parsing](../guide/incremental-parsing.md) for editor integration
3. Explore [Query and Pattern Matching](../guide/query-patterns.md) for code analysis
4. Check out the [Grammar Examples](../reference/grammar-examples.md) for inspiration

## Tips for Success

1. **Start Simple**: Begin with a minimal grammar and add features incrementally
2. **Test as You Go**: Write tests for each grammar rule
3. **Use the Playground**: The interactive playground helps visualize parse trees
4. **Check Examples**: The repository includes many example grammars
5. **Enable Debug Output**: Set `RUST_LOG=debug` to see parser decisions