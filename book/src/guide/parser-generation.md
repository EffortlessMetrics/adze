# Parser Generation

This chapter explains how Adze transforms your grammar definitions into efficient parsers.

## The Generation Process

### Build-Time Generation

Adze uses a build script to generate parsers at compile time:

1. **Grammar Extraction**: The `adze-tool` reads your Rust source files
2. **IR Generation**: Converts Rust types to an intermediate representation
3. **Parser Generation**: Creates Tree-sitter grammar JSON or pure-Rust tables
4. **Compilation**: Compiles the generated parser into your binary

### Generated Files

When you build with `ADZE_EMIT_ARTIFACTS=true`, you can inspect:

```
target/debug/build/*/out/
├── grammar.json        # Tree-sitter grammar
├── parser.c           # Generated C parser (C backend)
├── parser_tables.rs   # Generated Rust tables (pure-Rust)
└── node-types.json    # AST node type information
```

## Runtime Backends

### GLR Runtime (runtime2) - **Production Ready**

The modern GLR runtime provides:
- **Tree-sitter Compatible API**: Drop-in replacement with `Parser::new()`, `parse()`, etc.
- **GLR Engine Integration**: Handles ambiguous grammars with multiple parse paths
- **Incremental Parsing**: Automatic subtree reuse with conservative conflict avoidance
- **Performance Monitoring**: Built-in metrics via `ADZE_LOG_PERFORMANCE`
- **Feature Gates**: `glr-core`, `incremental`, `arenas` for different capabilities

```rust
use adze_runtime::{Parser, Language};

let mut parser = Parser::new();
parser.set_language(glr_language)?;  // Validates parse table presence
let tree = parser.parse_utf8("def main(): pass", None)?;
```

### Pure-Rust Backend (runtime)

The original pure-Rust backend generates:
- Static parse tables as Rust constants
- Compile-time optimized state machines
- Zero runtime parser generation
- WASM-compatible parsing

### C Backend

The C backend generates:
- Tree-sitter grammar.json
- C parser via Tree-sitter CLI
- Runtime parser initialization

## Optimization Phases

With the `optimize` feature enabled:

1. **Dead Code Elimination**: Removes unreachable rules
2. **Inline Expansion**: Inlines simple rules
3. **State Minimization**: Reduces parser states
4. **Table Compression**: Compresses parse tables

## Understanding Parse Tables

### LR(1) Tables

The parser uses LR(1) tables containing:
- **Action Table**: Maps (state, token) → action
- **Goto Table**: Maps (state, non-terminal) → state
- **Reduce Table**: Production rules for reductions

### GLR Extensions (Production Ready - Enhanced v0.6.1)

GLR parsing in runtime2 provides robust conflict resolution:
- **Multi-Action Cells**: Each (state, symbol) pair can hold multiple conflicting actions
- **Runtime Forking**: Parser dynamically forks on conflicts, exploring all valid paths
- **Precedence Disambiguation**: Correctly resolves operator precedence (e.g., `1+2*3` → `1+(2*3)`)
- **Error Recovery**: Graceful handling of malformed input with error node insertion
- **EOF Processing**: Fixed `process_eof()` parameter usage for proper end-of-input handling
- **Forest Management**: Efficient handling of ambiguous parse forests
- **Tree Conversion**: High-performance forest-to-tree conversion with metrics
- **Conflict Preservation**: Precedence/associativity orders actions but preserves alternatives

**Example: Handling Ambiguous Grammar**
```rust
// Grammar with shift/reduce conflicts
#[adze::language]
struct Module {
    statements: Vec<Statement>, // REPEAT(_statement) creates conflicts
}

// GLR parser handles both:
// 1. Empty files (reduce to empty module)
// 2. Files with statements (shift tokens)
let tree = parser.parse_utf8("", None)?;          // Empty file
let tree = parser.parse_utf8("def main():", None)?; // With statement

// Error recovery example:
let tree = parser.parse_utf8("1 + + 2", None)?;   // Recovers from double operator
// Result includes error nodes for invalid syntax while continuing to parse
```

### Error Recovery Enhancements (v0.6.1)

The GLR parser now includes robust error recovery:

```rust
// Input with syntax errors:
let malformed_input = "def func( # missing closing paren\n  pass";

// Parser gracefully recovers:
let tree = parser.parse_utf8(malformed_input, None)?;

// Parse tree includes error nodes:
Module {
    statements: vec![
        FunctionDef {
            name: Identifier("func"),
            params: ErrorNode {           // Error recovery inserted
                children: [/* partial param list */]
            },
            body: Block {
                statements: [Pass]        // Continues parsing after error
            }
        }
    ]
}
```

## Debugging Generation

### Enable Debug Output

```bash
RUST_LOG=debug cargo build
```

Shows:
- Grammar extraction steps
- Conflict detection
- Table generation
- Optimization decisions
- GLR state construction

### Enable Performance Monitoring

```bash
ADZE_LOG_PERFORMANCE=true cargo run
```

Outputs:
```
🚀 Forest->Tree conversion: 1247 nodes, depth 23, took 2.1ms
```

### Inspect Generated Grammar

```bash
ADZE_EMIT_ARTIFACTS=true cargo build
cat target/debug/build/*/out/grammar.json | jq
```

### GLR-Specific Debugging

```rust
// Debug GLR parse table loading
let language = Language::new_glr(parse_table, tokenizer, symbols);
match language.validate_glr() {
    Ok(()) => println!("GLR validation passed"),
    Err(msg) => eprintln!("GLR validation failed: {}", msg),
}
```

## Common Issues

### Large Parse Tables

Large grammars can generate big tables. Solutions:
1. Enable the `optimize` feature for table compression
2. Simplify grammar rules and reduce conflicts
3. Use `Box` for recursive types to reduce stack usage
4. Consider using GLR runtime2 which handles complex grammars efficiently

### Slow Build Times

Parser generation can be slow for complex grammars:
1. Use `cargo check` during development
2. Enable incremental compilation
3. Consider splitting large grammars
4. Use runtime2 for faster development iteration

### GLR Memory Usage

GLR parsing uses more memory due to multiple parse paths:
1. Enable `arenas` feature for better allocation performance
2. Monitor forest-to-tree conversion metrics
3. Use incremental parsing to reduce full parse frequency
4. Consider parser timeouts for pathological inputs

## Runtime Selection Guide

### Choose runtime2 (GLR) when:
- Parsing ambiguous grammars (C++, natural language)
- Need incremental parsing performance
- Want Tree-sitter API compatibility
- Require robust conflict handling

### Choose runtime (Pure Rust) when:
- WASM deployment is critical
- Grammar is unambiguous
- Maximum performance for simple grammars
- Minimal binary size requirements

## Next Steps

- Learn about [Incremental Parsing](incremental-parsing.md)
- Explore [Performance Optimization](performance.md) 
- Read about [Error Recovery](error-recovery.md)
- Understand [GLR Ambiguity Handling](glr-ambiguity.md)