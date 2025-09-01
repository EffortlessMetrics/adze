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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create parser with GLR runtime (production ready)
    use rust_sitter_runtime::Parser;
    
    let mut parser = Parser::new();
    parser.set_language(grammar::language())?; // Generated GLR language
    
    // Parse some expressions
    let expressions = vec![
        "42",
        "1 + 2",
        "1 + 2 * 3",
        "(1 + 2) * 3",
    ];
    
    for expr in expressions {
        println!("Parsing: {}", expr);
        match parser.parse_utf8(expr, None) {
            Ok(tree) => {
                let ast = grammar::extract_ast(&tree)?;
                println!("  Result: {:?}\n", ast);
            },
            Err(e) => println!("  Error: {}\n", e),
        }
    }
    
    // Demonstrate incremental parsing
    let initial = "1 + 2";
    let tree1 = parser.parse_utf8(initial, None)?;
    println!("Initial parse: {}", initial);
    
    let updated = "10 + 20";
    let tree2 = parser.parse_utf8(updated, Some(&tree1))?;  // Incremental!
    println!("Incremental parse: {} (reused compatible subtrees)", updated);
    
    Ok(())
}
```

### Step 2: Add Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-sitter = { version = "0.6", features = ["glr-core", "incremental"] }
rust-sitter-runtime = "0.1"  # GLR runtime

[build-dependencies]
rust-sitter-tool = "0.6"
```

Create `build.rs`:

```rust
fn main() {
    rust_sitter_tool::build_parsers().unwrap();
}
```

### Step 3: Run Your Parser

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

This creates a grammar named "arithmetic" and generates:
- A `language()` function returning a GLR-compatible `Language`
- An `extract_ast()` function to convert parse trees to your AST types
- Parser integration with runtime2's `Parser` API

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

The GLR runtime provides comprehensive error information:

```rust
match parser.parse_utf8("1 + + 2", None) {
    Ok(tree) => {
        match grammar::extract_ast(&tree) {
            Ok(ast) => println!("Parsed: {:?}", ast),
            Err(e) => println!("AST extraction error: {}", e),
        }
    },
    Err(e) => {
        println!("Parse error: {}", e);
        // GLR parsing can handle some syntax errors gracefully
        if let Some(partial_tree) = e.partial_tree() {
            println!("Partial parse available for error recovery");
        }
    }
}
```

## Performance Monitoring

Enable GLR performance metrics:

```bash
RUST_SITTER_LOG_PERFORMANCE=true cargo run
```

Outputs forest-to-tree conversion statistics:
```
🚀 Forest->Tree conversion: 247 nodes, depth 12, took 0.8ms
```

## GLR Features Available

With runtime2, you get:

- **Ambiguous Grammar Support**: Handle conflicts that traditional parsers reject
- **Incremental Parsing**: Automatic subtree reuse for editor-like performance
- **Error Recovery**: Partial parse trees available even with syntax errors
- **Tree-sitter Compatibility**: Drop-in replacement for existing Tree-sitter usage
- **Performance Monitoring**: Built-in metrics for optimization

## Next Steps

Now that you've created your first GLR grammar:

1. Read about [Parser Generation](../guide/parser-generation.md) to understand GLR vs Pure-Rust backends
2. Learn about [Incremental Parsing](../guide/incremental-parsing.md) for real-time editor integration
3. Explore [GLR Ambiguity Handling](../guide/glr-ambiguity.md) for complex grammar conflicts
4. Check out [Performance Optimization](../guide/performance.md) for production deployment

## Tips for Success

1. **Start Simple**: Begin with a minimal grammar and add features incrementally
2. **Test as You Go**: Write tests for each grammar rule
3. **Use the Playground**: The interactive playground helps visualize parse trees
4. **Check Examples**: The repository includes many example grammars
5. **Enable Debug Output**: Set `RUST_LOG=debug` to see GLR parser decisions
6. **Use Performance Monitoring**: Enable `RUST_SITTER_LOG_PERFORMANCE` for conversion metrics
7. **Start with GLR Runtime**: runtime2 provides the most features and Tree-sitter compatibility
8. **Test Incrementally**: Use incremental parsing for responsive applications