# Rust-Sitter v0.5.0-beta Quick Start Guide

## Installation

Add rust-sitter to your `Cargo.toml`:

```toml
[dependencies]
rust-sitter = "0.5.0-beta"

[build-dependencies]
rust-sitter-tool = "0.5.0-beta"
```

## Creating Your First Grammar

### 1. Define Your Grammar (src/lib.rs)

```rust
#[rust_sitter::grammar("my_language")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct Program {
        #[rust_sitter::repeat]
        pub statements: Vec<Statement>,
    }
    
    #[rust_sitter::language]
    pub enum Statement {
        Assignment(Assignment),
        Expression(Expression),
    }
    
    #[rust_sitter::language]
    pub struct Assignment {
        pub name: Identifier,
        #[rust_sitter::leaf(text = "=")]
        _eq: (),
        pub value: Expression,
    }
    
    #[rust_sitter::language]
    pub enum Expression {
        Number(Number),
        Identifier(Identifier),
    }
    
    #[rust_sitter::language]
    pub struct Number {
        #[rust_sitter::leaf(pattern = r"\d+")]
        pub value: (),
    }
    
    #[rust_sitter::language]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_]\w*")]
        pub name: (),
    }
}
```

### 2. Create Build Script (build.rs)

```rust
use rust_sitter_tool::build_parsers;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    build_parsers(&PathBuf::from("src/lib.rs"));
}
```

### 3. Use Your Parser

```rust
use my_language::grammar::Program;

fn main() {
    let input = "x = 42\ny = 100";
    match Program::parse(input) {
        Ok(tree) => println!("Parsed successfully: {:#?}", tree),
        Err(e) => println!("Parse error: {}", e),
    }
}
```

## Beta Limitations

### ❌ Not Yet Supported
- Precedence declarations (`#[rust_sitter::prec_left(1)]`)
- External scanners (full API)
- Complex conflict resolution
- Some Tree-sitter features (`word`, `extras`, etc.)

### ✅ What Works
- Basic grammar definitions
- Enums and structs
- Repetitions and optionals
- Pattern matching for tokens
- Simple parsing
- **GLR parsing** (ambiguous grammar support) ✨
- **True incremental parsing** with subtree reuse ✨
- **Performance monitoring** and optimization ✨

## Tips for Beta Users

1. **Keep Grammars Simple** - Avoid complex precedence rules
2. **Test Incrementally** - Build up your grammar piece by piece
3. **Check Examples** - Look at the JavaScript, Python, and Go examples
4. **Report Issues** - This is a beta, your feedback is valuable!

## GLR Features & Performance

### Using GLR Parsing
Enable GLR parsing for ambiguous grammars:

```toml
[dependencies]
rust-sitter = { version = "0.5.0-beta", features = ["glr-core"] }
# Note: GLR runtime is currently in runtime2/ directory (not yet published)
rust-sitter-runtime = { path = "../rust-sitter/runtime2", features = ["glr-core"] }
```

```rust
use rust_sitter_runtime::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let language = my_language::language();
    let mut parser = Parser::new();
    parser.set_language(language)?;
    
    let tree = parser.parse_utf8("ambiguous input", None)?;
    println!("Parsed with GLR: root kind = {}", tree.root_kind());
    Ok(())
}
```

### Incremental Parsing
Enable true incremental parsing for large files:

```toml
[dependencies]
rust-sitter = { version = "0.5.0-beta", features = ["incremental_glr"] }
```

```rust
use rust_sitter_runtime::{Parser, glr_incremental};

// Monitor reuse effectiveness
glr_incremental::reset_reuse_counter();

let old_tree = parser.parse_utf8("original content", None)?;
let new_tree = parser.parse_utf8("modified content", Some(&old_tree))?;

let reused = glr_incremental::get_reuse_count();
println!("Reused {} subtrees during incremental parse", reused);
```

### Performance Optimization
Enable performance monitoring to optimize your parser:

```rust
use std::env;

// Enable detailed logging
env::set_var("RUST_SITTER_LOG_PERFORMANCE", "true");

// Parse with metrics
let tree = parser.parse_utf8(large_input, None)?;
// Output: "🚀 Forest->Tree conversion: 1247 nodes, depth 23, took 2.1ms"
```

**Optimization Tips:**
- Use incremental parsing for large files or frequent edits
- Monitor subtree reuse with `SUBTREE_REUSE_COUNT` 
- Set `RUST_TEST_THREADS=2` for consistent benchmarking
- Enable `RUST_SITTER_LOG_PERFORMANCE` during development

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

### Token Patterns
```rust
pub struct StringLiteral {
    #[rust_sitter::leaf(pattern = r#""[^"]*""#)]
    pub value: (),
}
```

## Error Recovery & Robust Parsing

### Safe Span Operations (PR #55) ✅
Rust-sitter now includes comprehensive error recovery for span operations, preventing panics when working with malformed code:

```rust
use rust_sitter::{Spanned, SpanError};

// Safe span extraction with error handling
fn extract_function_name(source: &str, span: &Spanned<()>) -> Option<String> {
    match span.try_slice_str(source) {
        Ok(name) => Some(name.to_string()),
        Err(SpanError::OutOfBounds { span, length }) => {
            eprintln!("Span {:?} exceeds source length {}", span, length);
            None
        },
        Err(SpanError::InvalidRange { start, end }) => {
            eprintln!("Invalid span range: {} > {}", start, end);
            None
        }
    }
}
```

### Graceful Parser Error Handling
```rust
use rust_sitter_runtime::{Parser, ParseError};

fn parse_with_recovery(source: &str) -> Result<Program, String> {
    match Program::parse(source) {
        Ok(tree) => Ok(tree),
        Err(ParseError::UnexpectedToken { expected, found, location }) => {
            eprintln!("Parse error at {}:{}: expected {:?}, found '{}'", 
                location.line, location.column, expected, found);
            Err("Parse failed - try fixing syntax errors".to_string())
        },
        Err(e) => {
            eprintln!("Parse error: {}", e);
            Err("Parse failed".to_string())
        }
    }
}
```

### Incremental Parsing with Error Recovery
```rust
use rust_sitter_runtime::{InputEdit, EditError, Point};

fn apply_edit_safely(source: &mut String, edit: InputEdit) -> Result<(), EditError> {
    // Validate edit bounds before applying
    if edit.start_byte > source.len() {
        return Err(EditError::InvalidRange { 
            start: edit.start_byte, 
            old_end: edit.old_end_byte 
        });
    }
    
    // Apply edit to source
    let start = edit.start_byte;
    let old_end = edit.old_end_byte.min(source.len());
    let new_text = &source[edit.start_byte..edit.new_end_byte];
    
    source.replace_range(start..old_end, new_text);
    Ok(())
}
```

## Troubleshooting

### Grammar Conflicts
If you see "conflict" errors during build:
1. Simplify your grammar
2. Make optional elements explicit
3. Avoid ambiguous patterns

### Error Recovery Issues
If your parser crashes on malformed input:

1. **Use Safe Span Operations**:
   ```rust
   // ❌ Can panic on malformed input
   let text = &source[span.0..span.1];
   
   // ✅ Safe with error recovery
   let text = match span.try_slice_str(source) {
       Ok(text) => text,
       Err(e) => {
           eprintln!("Span error: {}", e);
           return Err("Invalid span");
       }
   };
   ```

2. **Handle Edit Errors**:
   ```rust
   // ✅ Always check edit results
   match tree.edit(&edit) {
       Ok(()) => { /* proceed with reparse */ },
       Err(EditError::InvalidRange { start, old_end }) => {
           eprintln!("Invalid edit range: {} -> {}", start, old_end);
       },
       Err(e) => eprintln!("Edit error: {}", e),
   }
   ```

3. **Test with Malformed Input**:
   ```rust
   #[test]
   fn test_malformed_input() {
       let bad_inputs = vec![
           "fn main(",           // Missing paren
           "let x = ;",         // Missing expr
           "",                  // Empty
       ];
       
       for input in bad_inputs {
           // Should not panic, even on bad input
           let result = Program::parse(input);
           assert!(result.is_err());
       }
   }
   ```

### Missing Features
If a Tree-sitter feature isn't working:
1. Check the known limitations
2. Find a workaround in the examples
3. Wait for the next release 😊

## Next Steps

- Explore the examples in `/examples`
- Read `GRAMMAR_EXAMPLES.md` for more patterns
- Check `KNOWN_LIMITATIONS.md` for current restrictions
- Join the discussion on GitHub

Happy parsing! 🦀🌳