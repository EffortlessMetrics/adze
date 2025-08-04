# Parser Generation

This chapter explains how Rust-Sitter transforms your grammar definitions into efficient parsers.

## The Generation Process

### Build-Time Generation

Rust-Sitter uses a build script to generate parsers at compile time:

1. **Grammar Extraction**: The `rust-sitter-tool` reads your Rust source files
2. **IR Generation**: Converts Rust types to an intermediate representation
3. **Parser Generation**: Creates Tree-sitter grammar JSON or pure-Rust tables
4. **Compilation**: Compiles the generated parser into your binary

### Generated Files

When you build with `RUST_SITTER_EMIT_ARTIFACTS=true`, you can inspect:

```
target/debug/build/*/out/
├── grammar.json        # Tree-sitter grammar
├── parser.c           # Generated C parser (C backend)
├── parser_tables.rs   # Generated Rust tables (pure-Rust)
└── node-types.json    # AST node type information
```

## Backend Differences

### Pure-Rust Backend

The pure-Rust backend generates:
- Static parse tables as Rust constants
- Compile-time optimized state machines
- Zero runtime parser generation

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

### GLR Extensions

GLR parsing adds:
- **Fork States**: States where parsing can diverge
- **Merge Points**: States where paths reconverge
- **Conflict Resolution**: Dynamic precedence handling

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

### Inspect Generated Grammar

```bash
RUST_SITTER_EMIT_ARTIFACTS=true cargo build
cat target/debug/build/*/out/grammar.json | jq
```

## Common Issues

### Large Parse Tables

Large grammars can generate big tables. Solutions:
1. Enable the `optimize` feature
2. Simplify grammar rules
3. Use `Box` for recursive types

### Slow Build Times

Parser generation can be slow for complex grammars:
1. Use `cargo check` during development
2. Enable incremental compilation
3. Consider splitting large grammars

## Next Steps

- Learn about [Query and Pattern Matching](query-patterns.md)
- Explore [Performance Optimization](performance.md)
- Read about [Error Recovery](error-recovery.md)