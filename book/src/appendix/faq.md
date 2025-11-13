# Frequently Asked Questions

## General Questions

### What is Rust-Sitter?

Rust-Sitter is a Rust framework that makes it easy to create efficient parsers by leveraging the Tree-sitter parser generator. It allows you to define grammars using Rust's type system with procedural macros.

### How does it differ from Tree-sitter?

While Tree-sitter requires writing grammars in JavaScript, Rust-Sitter lets you define grammars directly in Rust with type safety and IDE support. It generates Tree-sitter-compatible parsers while providing a more ergonomic Rust API.

### What languages are supported?

Rust-Sitter can parse any language you define a grammar for. Example grammars are provided for JavaScript, Python, Go, and more. See the [Language Support](../reference/language-support.md) page for details.

## Technical Questions

### What is GLR parsing?

GLR (Generalized LR) parsing is an extension of LR parsing that can handle ambiguous grammars by maintaining multiple parse stacks. When the parser encounters ambiguity, it forks and explores all possibilities, merging when paths converge.

### Should I use the pure-Rust or C backend?

**Use pure-Rust if you need:**
- WASM support
- No C dependencies
- Better cross-compilation
- Static parser generation

**Use the C backend if you need:**
- Maximum compatibility with existing Tree-sitter tools
- Specific Tree-sitter features not yet in pure-Rust

### How do I handle whitespace?

Define whitespace as an "extra" token that's automatically skipped:

```rust
#[rust_sitter::extra]
struct Whitespace {
    #[rust_sitter::leaf(pattern = r"\s+")]
    _ws: (),
}
```

### Can I use external scanners?

Yes, but with limitations in the current beta. Basic external scanner support exists, but the full Tree-sitter external scanner API is still being implemented.

## Performance Questions

### How fast is Rust-Sitter?

Rust-Sitter parsers are comparable in speed to hand-written parsers:
- **Parsing**: 50-200 MB/s typical
- **Incremental**: Sub-millisecond for typical edits
- **Memory**: Low overhead with object pooling

### Does it support incremental parsing?

Yes! Incremental parsing is fully supported, allowing efficient reparsing after edits. This is essential for editor integrations.

### What optimizations are available?

Enable optimizations via features:
- `optimize`: Grammar optimizer
- `parallel`: Multi-threaded parsing
- `simd`: SIMD-accelerated lexing

## Troubleshooting

### "Multiple applicable items in scope" error

You likely have conflicting backend features enabled. Choose only one:
- `pure-rust`
- `tree-sitter-c2rust`
- `tree-sitter-standard`

### Build fails with macro errors

Ensure both dependencies are present:
```toml
[dependencies]
rust-sitter = "0.8.0-dev"

[build-dependencies]
rust-sitter-tool = "0.8.0-dev"
```

### Grammar has conflicts

This is normal for ambiguous grammars. Options:
1. Add precedence annotations
2. Refactor to remove ambiguity
3. Use GLR parsing (automatic and production-ready)

### How do I fix precedence errors?

Common precedence errors and solutions:

**Multiple precedence attributes:**
```rust
// ❌ Error
#[rust_sitter::prec(1)]
#[rust_sitter::prec_left(2)]
struct Bad { }

// ✅ Fix: Use only one
#[rust_sitter::prec_left(2)]
struct Good { }
```

**Invalid precedence value:**
```rust
// ❌ Error: String instead of integer
#[rust_sitter::prec("high")]

// ✅ Fix: Use integer literal
#[rust_sitter::prec(10)]
```

**Variable instead of literal:**
```rust
// ❌ Error: Cannot use variables
const HIGH: u32 = 10;
#[rust_sitter::prec(HIGH)]

// ✅ Fix: Use literal value directly
#[rust_sitter::prec(10)]
```

### What precedence values should I use?

**Guidelines:**
- Range: `0` to `4294967295` (u32)
- Zero is valid (lowest precedence)
- Use meaningful gaps (10, 20, 30) for future expansion
- Higher numbers bind tighter

**Common patterns:**
```rust
#[rust_sitter::prec_left(10)]  // Addition, subtraction
#[rust_sitter::prec_left(20)]  // Multiplication, division
#[rust_sitter::prec_right(30)] // Exponentiation
#[rust_sitter::prec(40)]       // Comparison operators
```

### WASM build fails

Make sure you're using the `pure-rust` feature:
```toml
rust-sitter = { version = "0.8.0-dev", features = ["pure-rust"] }
```

## Migration Questions

### How do I migrate from Tree-sitter?

See the comprehensive [Migration Guide](../getting-started/migration.md).

### What changed in v0.8.0-dev?

Major changes from v0.5 include:
- Production-ready GLR parsing with 100% test pass rate
- Precedence disambiguation for correct operator precedence
- Enhanced error recovery and EOF processing
- Pure-Rust backend as default with full WASM support
- Comprehensive incremental parsing integration

See the [Changelog](changelog.md) for details.

### Is v0.8.0-dev stable?

v0.8.0-dev is production-ready with comprehensive test coverage. The implementation has been thoroughly validated with real-world grammars like Python (273 symbols). The API is stable and recommended for production use.

## Contributing

### How can I contribute?

We welcome contributions! See our [Contributing Guide](../development/contributing.md) for:
- Code style guidelines
- Testing requirements
- PR process

### Where do I report bugs?

Please report issues on our [GitHub repository](https://github.com/hydro-project/rust-sitter/issues).

### How do I add a new language grammar?

1. Create a new module with your grammar
2. Add tests for the grammar
3. Submit a PR with examples
4. See [Grammar Examples](../reference/grammar-examples.md) for patterns