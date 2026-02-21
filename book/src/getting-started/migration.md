# Migration Guide: Runtime to Runtime2 (GLR Integration)

This guide covers migrating from the original `runtime` crate to the new `runtime2` crate with production-ready GLR (Generalized LR) parser integration. The GLR runtime provides Tree-sitter API compatibility with enhanced capabilities for ambiguous grammars and incremental parsing.

## Major Changes

### 1. Runtime Crate Replacement

The most significant change is switching from `runtime` to `runtime2` with GLR integration.

**Before (runtime):**
```toml
[dependencies]
adze = { version = "0.5", features = ["runtime"] }
```

**After (runtime2):**
```toml
[dependencies]
adze-runtime = { version = "0.1", features = ["glr-core", "incremental"] }
```

This change provides:
- **GLR parsing capabilities**: Handle ambiguous grammars with conflicts
- **Tree-sitter API compatibility**: Drop-in replacement for Tree-sitter parsers
- **Enhanced incremental parsing**: Automatic subtree reuse optimization
- **Performance monitoring**: Built-in forest-to-tree conversion metrics

### 2. Parser API Changes

The parser instantiation and usage pattern has evolved:

**Before (runtime):**
```rust
use adze::Parser;

let parser = Parser::new();
let result = parser.parse(input)?;
```

**After (runtime2):**
```rust
use adze_runtime::Parser;

let mut parser = Parser::new();
parser.set_language(glr_language)?;  // GLR language with parse table
let tree = parser.parse_utf8(input, None)?;  // Optional incremental parsing
let ast = grammar::extract_ast(&tree)?;      // Convert tree to AST
```

### 3. Language Definition Changes

Language definition now requires GLR-specific components:

**Before (runtime):**
```rust
// Generated automatically from grammar annotations
let language = grammar::language();
let parser = Parser::new(language);
```

**After (runtime2):**
```rust
// Generated with GLR support
let language = grammar::language();  // Now includes parse_table and tokenizer
let mut parser = Parser::new();
parser.set_language(language)?;      // Validates GLR requirements
```

### 4. Incremental Parsing Integration

Incremental parsing is now seamlessly integrated:

**Before (runtime):**
```rust
// Manual incremental parsing (if available)
let tree1 = parser.parse(input1)?;
// Complex edit tracking and partial reparse logic
```

**After (runtime2):**
```rust
// Automatic incremental parsing
let tree1 = parser.parse_utf8(input1, None)?;           // Initial parse
let tree2 = parser.parse_utf8(input2, Some(&tree1))?;   // Incremental parse
// Parser automatically reuses compatible subtrees
```

### 5. GLR Parser Features

Runtime2 includes production-ready GLR capabilities:

- **Multi-Action Cells**: Each (state, symbol) can hold multiple conflicting actions
- **Runtime Forking**: Automatic parsing path forking on conflicts
- **Forest Management**: Efficient handling of ambiguous parse forests
- **Performance Monitoring**: Built-in metrics for forest-to-tree conversion
- **Conservative Incremental**: Safe subtree reuse that maintains GLR correctness

Example of GLR conflict handling:

```rust
// Grammar with shift/reduce conflicts (e.g., empty production)
#[adze::language]
struct Module {
    statements: Vec<Statement>, // REPEAT(_statement) creates conflict
}

// GLR parser handles both cases automatically:
let empty_tree = parser.parse_utf8("", None)?;         // Reduce to empty
let stmt_tree = parser.parse_utf8("def main():", None)?; // Shift statement
```

### 6. Feature Flag System

Runtime2 uses a comprehensive feature flag system:

```toml
[dependencies]
adze-runtime = { version = "0.1", features = [
    "glr-core",          # GLR parsing engine (default)
    "incremental",       # Incremental parsing support
    "arenas",           # Arena allocators for performance
    "external-scanners", # Custom external scanner support
    "queries"           # Tree-sitter query language (future)
] }
```

## Migration Steps

### 1. Update Dependencies

Change your `Cargo.toml` to use runtime2:

```toml
[dependencies]
# Remove old runtime
# adze = "0.5"

# Add GLR runtime
adze-runtime = { version = "0.1", features = ["glr-core", "incremental"] }

[build-dependencies]
adze-tool = "0.6"  # Ensure build tool compatibility
```

### 2. Update Build Configuration

Ensure your `build.rs` uses the latest tool:

```rust
fn main() {
    adze_tool::build_parsers().unwrap();
}
```

### 3. Update Parser Usage

**Before:**
```rust
let result = grammar::parse(input)?;
```

**After:**
```rust
use adze_runtime::Parser;

let mut parser = Parser::new();
parser.set_language(grammar::language())?;
let tree = parser.parse_utf8(input, None)?;
let result = grammar::extract_ast(&tree)?;
```

### 4. Enable Performance Monitoring (Optional)

```bash
ADZE_LOG_PERFORMANCE=true cargo run
```

### 5. Test Incremental Parsing

```rust
let tree1 = parser.parse_utf8("initial input", None)?;
let tree2 = parser.parse_utf8("modified input", Some(&tree1))?;  // Incremental!
```

## Common Issues and Solutions

### Issue: "Language has no parse table - GLR integration pending"
**Solution**: Ensure your grammar generates GLR-compatible language with parse table:
```rust
// Generated function should include parse table
let language = grammar::language();  // Must have parse_table: Some(...)
```

### Issue: "Language has no tokenizer"
**Solution**: The generated GLR language needs a tokenizer. This is automatically provided by `adze-tool`.

### Issue: "GLR core feature not enabled"
**Solution**: Add the `glr-core` feature to your dependencies:
```toml
adze-runtime = { version = "0.1", features = ["glr-core"] }
```

### Issue: Performance issues with large inputs
**Solution**: 
1. Enable arena allocators: `features = ["arenas")`
2. Use incremental parsing for repeated edits
3. Monitor performance with `ADZE_LOG_PERFORMANCE=true`

## New Features to Explore

### GLR Capabilities
- **Ambiguous Grammar Support**: Parse grammars with shift/reduce and reduce/reduce conflicts
- **Multiple Parse Paths**: Automatic forking and merging of parse paths
- **Tree-sitter Compatibility**: Drop-in replacement for existing Tree-sitter parsers
- **Production Readiness**: Tested with complex grammars like Python (273 symbols, 57 fields)

### Performance Features
- **Forest-to-Tree Conversion**: High-performance conversion with real-time metrics
- **Incremental Parsing**: Conservative subtree reuse maintaining GLR correctness
- **Arena Allocators**: Optional memory optimization for parsing-heavy workloads
- **Zero-Cost Monitoring**: Performance instrumentation with no runtime overhead when disabled

### Development Features
- **Comprehensive Error Handling**: `EditError` with overflow/underflow protection
- **Feature-Gated Compilation**: Choose exactly the features you need
- **Thread Safety**: Concurrent parsing support with bounded resource usage
- **Debugging Support**: Built-in performance and parse state monitoring

## Benefits of Migration

1. **Enhanced Grammar Support**: Handle previously unparseable ambiguous grammars
2. **Better Performance**: Incremental parsing with intelligent subtree reuse
3. **Tree-sitter Ecosystem**: Compatible with existing Tree-sitter tooling and queries
4. **Production Ready**: Battle-tested GLR implementation with comprehensive error handling
5. **Future Proof**: Foundation for advanced features like query optimization and LSP generation

For more details on GLR features and best practices, see the [Parser Generation Guide](../guide/parser-generation.md).