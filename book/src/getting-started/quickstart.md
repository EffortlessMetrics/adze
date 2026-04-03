# Quick Start

This guide will walk you through creating your first Adze grammar and parser.

## Creating a Simple Arithmetic Parser

Let's build a parser for basic arithmetic expressions like `1 + 2 * 3`.

### Step 1: Define the Grammar

Create a new Rust project and add this to your `src/main.rs`:

```rust
#[adze::grammar("arithmetic")]
mod grammar {
    #[adze::language]
    pub enum Expr {
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32,
        ),
        
        #[adze::prec_left(1)]
        Add(
            Box<Expr>,
            #[adze::leaf(text = "+")] (),
            Box<Expr>,
        ),
        
        #[adze::prec_left(2)]
        Multiply(
            Box<Expr>,
            #[adze::leaf(text = "*")] (),
            Box<Expr>,
        ),
        
        Parenthesized(
            #[adze::leaf(text = "(")] (),
            Box<Expr>,
            #[adze::leaf(text = ")")] (),
        ),
    }
    
    // Define whitespace as extra (automatically skipped)
    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create parser with GLR runtime (production ready)
    use adze::unified_parser::Parser;
    
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
        match parser.parse(expr) {
            Some(tree) => {
                let ast = grammar::extract_ast(&tree)?;
                println!("  Result: {:?}\n", ast);
            },
            None => println!("  Error: Failed to parse\n"),
        }
    }
    
    // Demonstrate incremental parsing
    let initial = "1 + 2";
    let tree1 = parser.parse(initial).ok_or("Failed to parse initial")?;
    println!("Initial parse: {}", initial);
    
    let updated = "10 + 20";
    let tree2 = parser.parse(updated).ok_or("Failed to parse updated")?;  // Note: Currently falls back to full reparse
    println!("Incremental parse: {} (reused compatible subtrees)", updated);
    
    Ok(())
}
```

### Step 2: Add Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
adze = { version = "0.8.0-dev", features = ["glr", "incremental_glr"] }

[build-dependencies]
adze-tool = "0.8.0-dev"
```

Create `build.rs`:

```rust
fn main() {
    adze_tool::build_parsers().unwrap();
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
#[adze::grammar("arithmetic")]
mod grammar { ... }
```

This creates a grammar named "arithmetic" and generates:
- A `language()` function returning a GLR-compatible `Language`
- An `extract_ast()` function to convert parse trees to your AST types
- Parser integration with runtime2's `Parser` API

### The Language Root

```rust
#[adze::language]
pub enum Expr { ... }
```

This marks `Expr` as the root type that the parser will return.

### Leaf Nodes

```rust
#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
u32,
```

- `pattern`: Regular expression to match
- `transform`: Function to convert matched text to the field type
- `text`: Exact string to match (alternative to `pattern`)

### Precedence

```rust
#[adze::prec_left(1)]  // Lower precedence
#[adze::prec_left(2)]  // Higher precedence
```

Higher numbers bind more tightly. Multiplication (2) binds tighter than addition (1).

### Extra Tokens

```rust
#[adze::extra]
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
    #[adze::repeat]
    pub statements: Vec<Statement>,
}
```

### Delimited Lists

```rust
pub struct ArgList {
    #[adze::repeat(separator = ",")]
    pub args: Vec<Expression>,
}
```

### String Literals

```rust
pub struct StringLiteral {
    #[adze::leaf(pattern = r#""([^"\\]|\\.)*""#, transform = |s| s[1..s.len()-1].to_string())]
    pub value: String,
}
```

## GLR Parser Integration (PR #56) ✨

The new GLR parser brings advanced capabilities for handling ambiguous grammars:

### ActionCell Support
The GLR parser uses **ActionCells** - each parser state can hold multiple conflicting actions:

```rust
use adze::glr_parser_no_error_recovery::GLRParser;
use adze_glr_core::{build_lr1_automaton, FirstFollowSets};

// Build GLR parser with ActionCell support
let grammar = create_grammar();
let first_follow = FirstFollowSets::compute(&grammar)?;
let parse_table = build_lr1_automaton(&grammar, &first_follow)?;
let mut glr_parser = GLRParser::new(parse_table);

// GLR parser returns parse forests for ambiguous grammars
let forest = glr_parser.parse(&tokens)?;
println!("Parse alternatives: {}", forest.roots.len());

// Traditional single-tree extraction
let tree = runtime_parser.parse_utf8("1 + 2", None)?;
```

### Parse Forest Analysis
When dealing with ambiguous grammars, analyze all interpretations:

```rust
// Example: Ambiguous expression "1+2*3"  
// Can be parsed as ((1+2)*3) or (1+(2*3))
let forest = glr_parser.parse(&ambiguous_tokens)?;

for (i, root) in forest.roots.iter().enumerate() {
    println!("Interpretation {}: Symbol {} at {:?}", 
             i, root.symbol.0, root.span);
    println!("  Child alternatives: {}", root.alternatives.len());
}

// Use forest analyzer for complex ambiguity analysis
let ambiguous_nodes = forest.nodes.values()
    .filter(|node| node.alternatives.len() > 1)
    .count();
println!("Ambiguous decision points: {}", ambiguous_nodes);
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
ADZE_LOG_PERFORMANCE=true cargo run
```

Outputs forest-to-tree conversion statistics:
```
🚀 Forest->Tree conversion: 247 nodes, depth 12, took 0.8ms
```

## GLR Tree Structure Understanding

GLR parsers produce trees with different structure than traditional parsers. Following PR #64 analysis, here's how to work with GLR trees:

### Grammar-Rooted Trees

GLR parsers always root trees at the grammar's start symbol, not the content:

```rust
let tree = parser.parse_utf8("42", None)?;
let root = tree.root_node();

// ✅ Correct: Grammar start symbol as root
assert_eq!(root.kind(), "expr");           // Grammar start rule
assert_eq!(root.child_count(), 1);         // Contains content as child

// Navigate to actual content  
let content = root.child(0).unwrap();       // Get actual number
assert_eq!(content.kind(), "number");      // Content type

// ❌ Incorrect: Expecting content directly
// assert_eq!(root.kind(), "number");      // Wrong - content-centric thinking
```

### Tree Navigation Pattern

When traversing GLR trees, always navigate through the grammar hierarchy:

```rust
let mut cursor = tree.root_node().walk();

// Start at grammar root
assert_eq!(cursor.node().kind(), "expr");      // Grammar symbol
assert!(cursor.goto_first_child());            // Navigate to content
assert_eq!(cursor.node().kind(), "number");    // Actual content

// For complex expressions:
// expr (root)
// └── add_expr  
//     ├── number ("1")
//     ├── "+"
//     └── number ("2")
```

### Testing GLR Trees

When writing tests for GLR functionality:

```rust
#[test]
fn test_glr_arithmetic() {
    let tree = parser.parse_utf8("1 + 2", None).unwrap();
    let root = tree.root_node();
    
    // Root is grammar start symbol
    assert_eq!(root.kind(), "expr");
    
    // Content is child of grammar symbol
    let add_expr = root.child(0).unwrap();
    assert_eq!(add_expr.kind(), "add");
    
    // Navigate through production structure
    let left_operand = add_expr.child(0).unwrap();
    assert_eq!(left_operand.kind(), "number");
    assert_eq!(left_operand.text(source), "1");
}
```

This structure reflects the actual grammar productions rather than a content-centric view, which enables GLR's ambiguity handling capabilities.

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
3. Explore [GLR precedence behavior](../guide/glr-precedence-resolution.md) for complex grammar conflicts
4. Check out [Performance Optimization](../guide/performance.md) for production deployment

## Tips for Success

1. **Start Simple**: Begin with a minimal grammar and add features incrementally
2. **Test as You Go**: Write tests for each grammar rule
3. **Use the Playground**: The interactive playground helps visualize parse trees
4. **Check Examples**: The repository includes many example grammars
5. **Enable Debug Output**: Set `RUST_LOG=debug` to see GLR parser decisions
6. **Use Performance Monitoring**: Enable `ADZE_LOG_PERFORMANCE` for conversion metrics
7. **Start with GLR Runtime**: runtime2 provides the most features and Tree-sitter compatibility
8. **Test Incrementally**: Use incremental parsing for responsive applications
