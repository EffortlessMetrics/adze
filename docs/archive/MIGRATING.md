# Migration Guide

This document describes breaking changes and how to migrate your code, with a focus on the GLR runtime2 integration.

## 0.6.0 - GLR Runtime2 Integration (Production Ready)

### Major Change: Runtime to Runtime2 Migration

The biggest breaking change is the introduction of the production-ready `runtime2` crate with GLR capabilities.

**Before (runtime):**
```toml
[dependencies]
adze = { version = "0.5", features = ["runtime"] }
```

```rust
let result = grammar::parse(input)?;
```

**After (runtime2):**
```toml
[dependencies]
adze-runtime = { version = "0.1", features = ["glr-core", "incremental"] }

[build-dependencies]
adze-tool = "0.6"
```

```rust
use adze_runtime::Parser;

let mut parser = Parser::new();
parser.set_language(grammar::language())?;  // Generated GLR language
let tree = parser.parse_utf8(input, None)?;
let result = grammar::extract_ast(&tree)?;
```

### Language Generation Changes

Generated grammars now include GLR-specific components:

**Before:**
```rust
// Simple grammar generation
let language = grammar::language();
```

**After:**
```rust
// GLR language with parse table and tokenizer
let language = grammar::language();
assert!(language.parse_table.is_some());  // Required for GLR
assert!(language.tokenize.is_some());     // Required for tokenization
```

### Parser API Changes

**Before:**
```rust
// Direct parse function
match grammar::parse("1 + 2") {
    Ok(ast) => { /* use ast */ },
    Err(e) => { /* handle error */ },
}
```

**After:**
```rust
// Parser instance with language validation
let mut parser = Parser::new();
parser.set_language(grammar::language())?; // Validates GLR requirements

match parser.parse_utf8("1 + 2", None) {
    Ok(tree) => {
        let ast = grammar::extract_ast(&tree)?;
        // use ast
    },
    Err(e) => { /* handle ParseError */ },
}
```

### Incremental Parsing Changes

**Before:**
```rust
// No incremental support or manual implementation
```

**After:**
```rust
// Built-in incremental parsing with Tree-sitter compatibility
let tree1 = parser.parse_utf8("initial", None)?;
let tree2 = parser.parse_utf8("modified", Some(&tree1))?;  // Automatic incremental!

// Manual tree editing for complex scenarios
let mut tree = tree1.clone();
tree.edit(&InputEdit {
    start_byte: 0,
    old_end_byte: 7,
    new_end_byte: 8,
    // ... position info
})?;
let updated = parser.parse_utf8("modified", Some(&tree))?;
```

### Error Handling Changes

**Before:**
```rust
// Simple error types
match result {
    Err(e) => println!("Parse error: {}", e),
}
```

**After:**
```rust
// Comprehensive error handling with GLR capabilities
use adze_runtime::{ParseError, EditError};

match parser.parse_utf8(input, old_tree) {
    Ok(tree) => {
        match grammar::extract_ast(&tree) {
            Ok(ast) => { /* success */ },
            Err(e) => println!("AST extraction error: {}", e),
        }
    },
    Err(ParseError::NoLanguage) => println!("No language set"),
    Err(ParseError::InvalidInput(msg)) => println!("Invalid input: {}", msg),
    Err(e) => println!("Parse error: {}", e),
}

// Tree editing errors
match tree.edit(&edit) {
    Ok(()) => { /* edit successful */ },
    Err(EditError::InvalidRange { start, old_end }) => {
        println!("Invalid edit range: {}..{}", start, old_end);
    },
    Err(EditError::ArithmeticOverflow) => {
        println!("Edit would cause position overflow");
    },
    Err(EditError::ArithmeticUnderflow) => {
        println!("Edit would cause position underflow");
    },
}
```

### Feature Flag Migration

**Before:**
```toml
adze = { version = "0.5", features = ["pure-rust"] }
```

**After:**
```toml
adze-runtime = { 
    version = "0.1", 
    features = [
        "glr-core",          # GLR parsing (default)
        "incremental",       # Tree editing and incremental parsing
        "arenas",           # Memory optimization
        "external-scanners", # Custom scanners
    ] 
}
```

### Build System Changes

**Before:**
```rust
// build.rs - if any
```

**After:**
```rust
// build.rs - Required for GLR grammar generation
fn main() {
    adze_tool::build_parsers().unwrap();
}
```

### Performance Monitoring Integration

**New in runtime2:**
```bash
# Enable GLR performance monitoring
ADZE_LOG_PERFORMANCE=true cargo run
```

Outputs:
```
🚀 Forest->Tree conversion: 1247 nodes, depth 23, took 2.1ms
```

## 0.8.0

### Error Unification

New crate errors provide consistent error handling across the workspace:
- `ir::IrError` - IR-level errors (via `thiserror`)
- `glr-core::GLRError` (re-exported as `GlrError`) - Parser generation errors
- `tablegen::TableGenError` - Table generation and compression errors

The `tablegen` crate now has transparent error variants that preserve source chains:

```rust
pub enum TableGenError {
    // ... existing variants ...
    
    #[error(transparent)]
    Glr(#[from] adze_glr_core::GLRError),
    
    #[error(transparent)]
    Ir(#[from] adze_ir::error::IrError),
}
```

This enables better error debugging with preserved source chains:

```rust
// Error sources are preserved through conversions
let result: Result<_, TableGenError> = build_lr1_automaton(&g, &ff)
    .map_err(Into::into)?;
    
// You can traverse the error chain
if let Err(e) = result {
    let mut source = Some(&e as &dyn std::error::Error);
    while let Some(err) = source {
        eprintln!("Error: {}", err);
        source = err.source();
    }
}
```

When handling errors, you can now match on the specific typed variants:

```rust
use adze_tablegen::error::TableGenError;

match err {
    TableGenError::Ir(e)  => eprintln!("IR error: {e}"),
    TableGenError::Glr(e) => eprintln!("GLR error: {e}"),
    TableGenError::TableGeneration(msg) => eprintln!("Table generation: {msg}"),
    other => eprintln!("Other tablegen error: {other}"),
}
```

For now `GLRError` remains the canonical name for compatibility. We may standardize
on `GlrError` in a future release with a deprecation window.

### Documentation Features

New `strict_docs` feature flag for conditional documentation enforcement:

```toml
[features]
strict_docs = []  # Enforces documentation at compile time
```

When building documentation for docs.rs:

```bash
RUSTDOCFLAGS="--cfg docsrs" cargo doc --features strict_docs
```

### Removed: `compress_default`

The deprecated `compress_default` method has been removed. Replace with:

```rust
use adze_tablegen::compress::TableCompressor;
use adze_tablegen::helpers::collect_token_indices;

// Before (0.7.x):
let compressed = compressor.compress_default(&parse_table, &grammar)?;

// After (0.8.0):
let token_indices = collect_token_indices(&grammar, &parse_table);
let start_can_be_empty = eof_accepts_or_reduces(&parse_table);
let compressed = compressor.compress(&parse_table, &token_indices, start_can_be_empty)?;
```

The new API is more explicit about what data is needed and gives you control over
the `start_can_be_empty` parameter for performance optimization.

## 0.7.0

### TableCompressor API Changes

The `TableCompressor` constructor no longer takes grammar and parse table parameters.
Instead, these are passed to the `compress` method directly.

```rust
// Before (0.6.x):
let compressor = TableCompressor::new(&grammar, &parse_table);
let compressed = compressor.compress(&options)?;

// After (0.7.0):
let compressor = TableCompressor::new();
let token_indices = collect_token_indices(&grammar, &parse_table);
let compressed = compressor.compress(&parse_table, &token_indices, start_can_be_empty)?;
```