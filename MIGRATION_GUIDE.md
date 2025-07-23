# Migration Guide: From C-based Tree-sitter to Pure-Rust Implementation

This guide helps you migrate from the traditional C-based Tree-sitter to the pure-Rust implementation.

## Table of Contents
1. [Overview](#overview)
2. [Key Differences](#key-differences)
3. [Migration Steps](#migration-steps)
4. [Grammar Definition](#grammar-definition)
5. [Parser Usage](#parser-usage)
6. [Advanced Features](#advanced-features)
7. [Troubleshooting](#troubleshooting)

## Overview

The pure-Rust Tree-sitter implementation provides:
- 100% Rust code with no C dependencies
- Full compatibility with Tree-sitter grammars
- Enhanced features like built-in optimization and validation
- Better integration with Rust ecosystem
- WASM support out of the box

## Key Differences

### 1. Grammar Definition

**C-based Tree-sitter:**
```javascript
// grammar.js
module.exports = grammar({
  name: 'my_language',
  rules: {
    expression: $ => choice(
      $.number,
      $.binary_expression
    ),
    number: $ => /\d+/,
    binary_expression: $ => prec.left(1, seq(
      field('left', $.expression),
      field('operator', '+'),
      field('right', $.expression)
    ))
  }
});
```

**Pure-Rust Implementation:**
```rust
#[rust_sitter::grammar("my_language")]
pub mod grammar {
    #[rust_sitter::language]
    pub enum Expression {
        Number(
            #[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            i32
        ),
        #[rust_sitter::prec_left(1)]
        Add(
            Box<Expression>,
            #[rust_sitter::leaf(text = "+")]
            (),
            Box<Expression>
        ),
    }
}
```

### 2. Build Process

**C-based Tree-sitter:**
- Requires Node.js to generate parser
- Generates C code that needs compilation
- Complex build.rs with cc crate

**Pure-Rust Implementation:**
- Pure Rust proc macros
- No external toolchain required
- Simple build.rs:

```rust
// build.rs
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src");
    rust_sitter_tool::build_parsers(&PathBuf::from("src/main.rs"));
}
```

## Migration Steps

### Step 1: Update Dependencies

Replace your Tree-sitter dependencies:

```toml
# Old
[dependencies]
tree-sitter = "0.20"
tree-sitter-my-language = { path = "../tree-sitter-my-language" }

# New
[dependencies]
rust-sitter = { version = "0.4", features = ["tree-sitter-c2rust"] }

[build-dependencies]
rust-sitter-tool = "0.4"
```

### Step 2: Convert Grammar

1. Create a new Rust module for your grammar
2. Convert JavaScript rules to Rust enums/structs
3. Add appropriate attributes

**Conversion patterns:**

| Tree-sitter JS | Rust Sitter |
|----------------|-------------|
| `seq(a, b, c)` | Struct with fields |
| `choice(a, b)` | Enum variants |
| `repeat(x)` | `Vec<T>` |
| `optional(x)` | `Option<T>` |
| `prec.left(n, x)` | `#[rust_sitter::prec_left(n)]` |
| `/regex/` | `#[rust_sitter::leaf(pattern = "regex")]` |
| `'literal'` | `#[rust_sitter::leaf(text = "literal")]` |

### Step 3: Update Parser Usage

**Old:**
```rust
let mut parser = tree_sitter::Parser::new();
parser.set_language(tree_sitter_my_language::language())?;
let tree = parser.parse(input, None).unwrap();
let root = tree.root_node();
```

**New:**
```rust
use rust_sitter::Parser;

let parser = Parser::<grammar::MyLanguage>::new();
let tree = parser.parse(input, None).unwrap();
let root = tree.root_node();

// Or use the high-level API:
let ast = grammar::parse(input)?;
```

### Step 4: Update Tree Navigation

The tree navigation API remains largely the same:

```rust
// Both versions
let mut cursor = root.walk();
cursor.goto_first_child();
let node = cursor.node();
println!("Node kind: {}", node.kind());
println!("Node text: {}", node.utf8_text(input)?);
```

## Grammar Definition

### Basic Types

```rust
// Leaf nodes (terminals)
#[rust_sitter::leaf(text = "if")]
struct If;

#[rust_sitter::leaf(pattern = r"\d+")]
struct Number(String);

#[rust_sitter::leaf(pattern = r"\d+", transform = |s| s.parse().unwrap())]
struct ParsedNumber(i32);

// Sequences (non-terminals)
struct IfStatement {
    if_keyword: If,
    condition: Expression,
    then_block: Block,
}

// Choices
enum Statement {
    If(IfStatement),
    While(WhileStatement),
    Expression(Expression),
}
```

### Repetition and Optionals

```rust
struct Block {
    #[rust_sitter::repeat(non_empty = false)]
    statements: Vec<Statement>,
}

struct Function {
    name: Identifier,
    parameters: Option<ParameterList>,
}

struct ParameterList {
    #[rust_sitter::delimited(
        #[rust_sitter::leaf(text = ",")]
        ()
    )]
    parameters: Vec<Parameter>,
}
```

### Precedence and Associativity

```rust
#[rust_sitter::prec]
enum Expression {
    #[rust_sitter::prec_left(1)]
    Add(Box<Expression>, #[rust_sitter::leaf(text = "+")] (), Box<Expression>),
    
    #[rust_sitter::prec_left(2)]
    Multiply(Box<Expression>, #[rust_sitter::leaf(text = "*")] (), Box<Expression>),
    
    #[rust_sitter::prec(3)]
    Unary(#[rust_sitter::leaf(text = "-")] (), Box<Expression>),
}
```

## Parser Usage

### Basic Parsing

```rust
// Parse to tree
let parser = Parser::<grammar::MyLanguage>::new();
let tree = parser.parse(code, None)?;

// Parse to AST
let ast: grammar::MyLanguage = grammar::parse(code)?;
```

### Error Handling

```rust
match grammar::parse(code) {
    Ok(ast) => {
        // Process AST
    }
    Err(errors) => {
        for error in errors {
            eprintln!("Parse error at {}:{}: {}", 
                error.start, error.end, error.reason);
        }
    }
}
```

### Incremental Parsing

```rust
let mut parser = Parser::<grammar::MyLanguage>::new();
let old_tree = parser.parse(old_code, None)?;

// Edit the code
let new_tree = parser.parse(new_code, Some(&old_tree))?;
```

## Advanced Features

### Grammar Optimization

```rust
use rust_sitter_ir::{GrammarOptimizer};

let mut optimizer = GrammarOptimizer::new();
optimizer.optimize_grammar(&mut grammar);

println!("Optimization stats: {:?}", optimizer.get_stats());
```

### Grammar Validation

```rust
use rust_sitter_ir::{GrammarValidator};

let validator = GrammarValidator::new();
let result = validator.validate(&grammar);

for error in &result.errors {
    eprintln!("Grammar error: {}", error);
}
```

### Visitor Pattern

```rust
use rust_sitter::visitor::{Visitor, TreeWalker, VisitorAction};

struct MyVisitor {
    identifier_count: usize,
}

impl Visitor for MyVisitor {
    fn enter_node(&mut self, node: &Node) -> VisitorAction {
        if node.kind() == "identifier" {
            self.identifier_count += 1;
        }
        VisitorAction::Continue
    }
}

let walker = TreeWalker::new(source);
let mut visitor = MyVisitor { identifier_count: 0 };
walker.walk(tree.root_node(), &mut visitor);
```

### Error Recovery

```rust
use rust_sitter::error_recovery::{ErrorRecoveryConfig, ErrorRecoveryState};

let config = ErrorRecoveryConfig::default()
    .with_sync_tokens(vec![SEMICOLON, RBRACE])
    .with_scope_delimiters(vec![(LPAREN, RPAREN)]);

let mut recovery = ErrorRecoveryState::new(config);
// Use during parsing for better error recovery
```

## Troubleshooting

### Common Issues

1. **"Expected a path" panic during build**
   - Ensure leaf types are simple paths (not complex types)
   - Use type aliases for complex types

2. **Grammar doesn't compile**
   - Check that all recursive types use `Box<T>`
   - Ensure enums have at least one variant
   - Verify leaf patterns are valid regex

3. **Performance differences**
   - Enable release mode: `cargo build --release`
   - Use `tree-sitter-c2rust` feature for best performance
   - Consider grammar optimization

### Migration Checklist

- [ ] Update Cargo.toml dependencies
- [ ] Create Rust grammar module
- [ ] Convert all rules to Rust types
- [ ] Add build.rs with `build_parsers`
- [ ] Update parser initialization code
- [ ] Test with existing test cases
- [ ] Enable new features (optimization, validation)
- [ ] Update CI/CD pipelines

## Resources

- [API Documentation](./API_DOCUMENTATION.md)
- [Example Grammars](./example/src/)
- [Performance Guide](./PERFORMANCE_RESULTS.md)
- [GitHub Repository](https://github.com/hydro-project/rust-sitter)

## Getting Help

If you encounter issues during migration:
1. Check the example grammars for patterns
2. Review the API documentation
3. Open an issue on GitHub
4. Join the community discussions

The pure-Rust implementation aims to be a drop-in replacement with enhanced features. Most Tree-sitter concepts translate directly, making migration straightforward for most grammars.