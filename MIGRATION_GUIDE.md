# Migration Guide: Tree-sitter to Rust Sitter

This guide helps you migrate existing Tree-sitter grammars to rust-sitter.

## Overview

Rust Sitter provides ~98% compatibility with Tree-sitter grammars while offering:
- Pure Rust implementation (no C dependencies)
- Type-safe grammar definitions
- Enhanced error recovery
- Better incremental parsing
- Full WASM support

## Quick Start

### 1. Basic Grammar Migration

**Tree-sitter (JavaScript):**
```javascript
module.exports = grammar({
  name: 'my_language',
  
  rules: {
    source_file: $ => repeat($.statement),
    
    statement: $ => choice(
      $.expression_statement,
      $.if_statement
    ),
    
    expression_statement: $ => seq(
      $.expression,
      ';'
    ),
    
    expression: $ => choice(
      $.identifier,
      $.number
    ),
    
    identifier: $ => /[a-zA-Z_]\w*/,
    number: $ => /\d+/
  }
});
```

**Rust Sitter:**
```rust
#[rust_sitter::grammar("my_language")]
mod grammar {
    #[rust_sitter::language]
    pub struct SourceFile {
        statements: Vec<Statement>,
    }
    
    pub enum Statement {
        Expression(ExpressionStatement),
        If(IfStatement),
    }
    
    pub struct ExpressionStatement {
        expression: Expression,
        #[rust_sitter::leaf(text = ";")]
        semicolon: (),
    }
    
    pub enum Expression {
        Identifier(
            #[rust_sitter::leaf(pattern = r"[a-zA-Z_]\w*")]
            String
        ),
        Number(
            #[rust_sitter::leaf(pattern = r"\d+", transform = |s| s.parse().unwrap())]
            u32
        ),
    }
}
```

### 2. Precedence and Associativity

**Tree-sitter:**
```javascript
expression: $ => choice(
  prec.left(2, seq($.expression, '*', $.expression)),
  prec.left(1, seq($.expression, '+', $.expression)),
  $.primary
)
```

**Rust Sitter:**
```rust
pub enum Expression {
    #[rust_sitter::prec_left(2)]
    Multiply(
        Box<Expression>,
        #[rust_sitter::leaf(text = "*")] (),
        Box<Expression>
    ),
    
    #[rust_sitter::prec_left(1)]
    Add(
        Box<Expression>,
        #[rust_sitter::leaf(text = "+")] (),
        Box<Expression>
    ),
    
    Primary(Primary),
}
```

### 3. Field Names

**Tree-sitter:**
```javascript
function_declaration: $ => seq(
  'function',
  field('name', $.identifier),
  field('parameters', $.parameter_list),
  field('body', $.block)
)
```

**Rust Sitter:**
```rust
pub struct FunctionDeclaration {
    #[rust_sitter::leaf(text = "function")]
    keyword: (),
    
    #[rust_sitter::field("name")]
    name: Identifier,
    
    #[rust_sitter::field("parameters")]
    parameters: ParameterList,
    
    #[rust_sitter::field("body")]
    body: Block,
}
```

### 4. Repetition and Options

**Tree-sitter:**
```javascript
parameter_list: $ => seq(
  '(',
  optional(seq(
    $.parameter,
    repeat(seq(',', $.parameter))
  )),
  ')'
)
```

**Rust Sitter:**
```rust
pub struct ParameterList {
    #[rust_sitter::leaf(text = "(")]
    lparen: (),
    
    #[rust_sitter::delimited(
        #[rust_sitter::leaf(text = ",")]
        ()
    )]
    parameters: Vec<Parameter>,
    
    #[rust_sitter::leaf(text = ")")]
    rparen: (),
}
```

### 5. External Scanners

**Tree-sitter (C):**
```c
enum TokenType {
  INDENT,
  DEDENT,
  NEWLINE
};

void *tree_sitter_python_external_scanner_create() {
  return calloc(1, sizeof(Scanner));
}

bool tree_sitter_python_external_scanner_scan(
  void *payload,
  TSLexer *lexer,
  const bool *valid_symbols
) {
  Scanner *scanner = (Scanner *)payload;
  // Scanning logic...
}
```

**Rust Sitter:**
```rust
use rust_sitter::external_scanner::{ExternalScanner, Lexer, ScanResult};

#[derive(Default)]
struct PythonScanner {
    indent_stack: Vec<usize>,
}

impl ExternalScanner for PythonScanner {
    fn scan(&mut self, lexer: &mut Lexer, valid_symbols: &[bool]) -> ScanResult {
        if valid_symbols[NEWLINE] && lexer.lookahead() == '\n' {
            lexer.advance(false);
            lexer.mark_end();
            lexer.result_symbol(NEWLINE);
            
            // Handle indentation...
            
            return ScanResult::Found;
        }
        ScanResult::NotFound
    }
}
```

### 6. Extras and Word

**Tree-sitter:**
```javascript
module.exports = grammar({
  name: 'my_language',
  
  extras: $ => [
    /\s/,
    $.comment
  ],
  
  word: $ => $.identifier,
  
  rules: {
    // ...
  }
});
```

**Rust Sitter:**
```rust
#[rust_sitter::grammar("my_language")]
mod grammar {
    #[rust_sitter::extra]
    pub enum Extra {
        Whitespace(
            #[rust_sitter::leaf(pattern = r"\s")]
            ()
        ),
        Comment(Comment),
    }
    
    #[rust_sitter::word]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_]\w*")]
        value: String,
    }
}
```

## Advanced Migration

### Conflicts

**Tree-sitter:**
```javascript
conflicts: $ => [
  [$.type_expression, $.primary_expression]
]
```

**Rust Sitter:**
```rust
// Handled automatically by GLR parsing
// Or use explicit precedence annotations
```

### Dynamic Precedence

**Tree-sitter:**
```javascript
prec.dynamic(1, $.expression)
```

**Rust Sitter:**
```rust
#[rust_sitter::prec_dynamic(1)]
Expression(Box<Expression>)
```

### Inline Rules

**Tree-sitter:**
```javascript
inline: $ => [
  $._expression,
  $._statement
]
```

**Rust Sitter:**
```rust
// Use Rust's type system instead
type Expression = ExpressionImpl;
type Statement = StatementImpl;
```

## Build System Migration

### Tree-sitter:
```javascript
// binding.gyp, package.json, etc.
```

### Rust Sitter:
```toml
# Cargo.toml
[dependencies]
rust-sitter = "0.4.5"

[build-dependencies]
rust-sitter-tool = "0.4.5"
```

```rust
// build.rs
fn main() {
    rust_sitter_tool::build_parsers(&PathBuf::from("src/grammar.rs"));
}
```

## Query Migration

Tree-sitter queries work unchanged in rust-sitter:

```scheme
(function_declaration
  name: (identifier) @function.name
  body: (block) @function.body)

(#match? @function.name "^test_")
```

## Performance Tips

1. **Use `Box<T>` for recursive types** to avoid infinite size
2. **Prefer enums over choices** for better performance
3. **Use field names** for better incremental parsing
4. **Enable table compression** in release builds

## Common Pitfalls

### 1. Token Transformation
```rust
// Wrong: transform returns wrong type
#[rust_sitter::leaf(pattern = r"\d+", transform = |s| s)]

// Correct: parse string to number
#[rust_sitter::leaf(pattern = r"\d+", transform = |s| s.parse().unwrap())]
```

### 2. Missing Extras
```rust
// Don't forget to mark whitespace as extra
#[rust_sitter::extra]
struct Whitespace {
    #[rust_sitter::leaf(pattern = r"\s+")]
    _ws: (),
}
```

### 3. Recursive Types
```rust
// Wrong: infinite size
pub enum Expr {
    Binary(Expr, Op, Expr)
}

// Correct: use Box
pub enum Expr {
    Binary(Box<Expr>, Op, Box<Expr>)
}
```

## Testing Migration

### Tree-sitter:
```javascript
const parser = require('tree-sitter-my-language');
// Test with tree-sitter CLI
```

### Rust Sitter:
```rust
#[test]
fn test_parsing() {
    let result = grammar::parse("let x = 42;");
    assert!(result.is_ok());
}
```

## Tool Compatibility

- **tree-sitter CLI**: Use `rust-sitter-tool` instead
- **Syntax highlighting**: Compatible with existing queries
- **Language servers**: Use generated parsers as drop-in replacements
- **Editors**: Works with any Tree-sitter-enabled editor

## Getting Help

1. Check examples in the `example/` directory
2. Run with `RUST_SITTER_EMIT_ARTIFACTS=true` to debug
3. Use `cargo insta review` for snapshot testing
4. Join the community Discord for support

## Incremental Migration

You can migrate gradually:

1. Start with core grammar rules
2. Add external scanners if needed
3. Migrate queries and highlights
4. Update build system
5. Test thoroughly

The rust-sitter implementation maintains compatibility while offering better performance and type safety.