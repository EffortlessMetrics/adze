# Rust-Sitter v0.5.0 Release Notes

## 🎉 Major Release: Pure-Rust Tree-sitter Implementation

We are excited to announce the release of Rust-Sitter v0.5.0, featuring a complete pure-Rust implementation of Tree-sitter with significant enhancements and new capabilities.

### 🌟 Highlights

- **100% Pure Rust**: No C dependencies required
- **Enhanced Features**: Grammar optimization, validation, and visualization
- **Improved Performance**: Competitive parsing speeds with efficient memory usage
- **Better Developer Experience**: Comprehensive error recovery and diagnostics
- **Full Compatibility**: Works with existing Tree-sitter grammars

## 📋 What's New

### Core Features

#### 1. Grammar Optimization (`rust-sitter-ir`)
- Automatic removal of unused symbols and rules
- Rule inlining for better performance
- Token pattern merging
- Left recursion optimization
- Comprehensive optimization statistics

#### 2. Advanced Error Recovery (`rust-sitter`)
- Multiple recovery strategies:
  - Panic mode with synchronization tokens
  - Token insertion/deletion/substitution
  - Phrase-level recovery
  - Scope-based recovery (bracket matching)
  - Indentation-based recovery
- Configurable recovery behavior
- Better error messages with context

#### 3. Conflict Resolution (`rust-sitter-glr-core`)
- Precedence-based resolution
- Associativity handling
- GLR fork/merge decision support
- Detailed conflict statistics
- Support for complex grammars

#### 4. Grammar Validation (`rust-sitter-ir`)
- Early detection of grammar issues:
  - Undefined symbols
  - Unreachable rules
  - Non-productive symbols
  - Cycle detection
  - Field validation
- Comprehensive warnings and suggestions
- Grammar statistics reporting

#### 5. Parse Tree Visitors (`rust-sitter`)
- Flexible visitor pattern API
- Depth-first and breadth-first traversal
- Built-in visitors:
  - Statistics collection
  - Node searching
  - Pretty printing
  - Tree transformation
- Easy custom visitor implementation

#### 6. Tree Serialization (`rust-sitter`)
- Multiple serialization formats:
  - JSON (full and compact)
  - S-expressions
  - Binary format
- Configurable serialization options
- Efficient deserialization

#### 7. Visualization Tools (`rust-sitter-tool`)
- Grammar visualization:
  - Graphviz DOT generation
  - Railroad diagrams
  - ASCII art representation
  - Dependency graphs
- Interactive debugging support

### Performance Improvements

- **Fast Parsing**: Sub-millisecond parsing for typical expressions
- **Linear Scaling**: Performance scales well with input size
- **Memory Efficient**: No memory leaks, stable usage patterns
- **Optimized Tables**: Compressed parse tables for smaller binary size

### Developer Experience

- **Better Error Messages**: Clear, actionable error diagnostics
- **Comprehensive Documentation**: Migration guide, usage examples, API docs
- **Type-Safe AST**: Leverage Rust's type system for safer code
- **Macro Improvements**: More intuitive grammar definition syntax

## 🔄 Breaking Changes

### Grammar Definition
- Grammar definitions now use Rust enums and structs instead of JavaScript
- New attribute-based syntax for grammar rules
- See [Migration Guide](./MIGRATION_GUIDE.md) for details

### API Changes
- Parser initialization uses type parameters: `Parser::<Grammar>::new()`
- AST types are now strongly typed Rust structures
- Error types have been redesigned for better ergonomics

## 🚀 Migration

See our comprehensive [Migration Guide](./MIGRATION_GUIDE.md) for step-by-step instructions on migrating from C-based Tree-sitter.

### Quick Start

```rust
// Define grammar
#[rust_sitter::grammar("my_language")]
pub mod grammar {
    #[rust_sitter::language]
    pub enum Expression {
        Number(
            #[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            i32
        ),
        // ... more rules
    }
}

// Parse code
let ast = grammar::parse("123").unwrap();
```

## 📊 Performance

Benchmarks show excellent performance characteristics:
- Simple expressions: ~35 µs
- Complex expressions: ~177 µs
- Large expressions (50 terms): ~1.37 ms

See [Performance Results](./PERFORMANCE_RESULTS.md) for detailed benchmarks.

## 🛠️ Compatibility

- **Rust Version**: 1.70.0 or later
- **Platforms**: Linux, macOS, Windows
- **Architectures**: x86_64, ARM64, WASM
- **Features**: 
  - `tree-sitter-c2rust` (default): Pure Rust backend
  - `tree-sitter-standard`: Standard C backend (legacy)
  - `serialization`: Enable tree serialization features

## 📚 Documentation

- [API Documentation](./API_DOCUMENTATION.md)
- [Migration Guide](./MIGRATION_GUIDE.md)
- [Usage Examples](./USAGE_EXAMPLES.md)
- [Implementation Roadmap](./IMPLEMENTATION_ROADMAP.md)

## 🤝 Contributing

We welcome contributions! Please see our contributing guidelines and check the [Implementation Roadmap](./IMPLEMENTATION_ROADMAP.md) for areas where help is needed.

## 🙏 Acknowledgments

Thanks to all contributors who made this release possible:
- The original Tree-sitter team for the foundational work
- The rust-sitter community for feedback and testing
- Contributors to the pure-Rust implementation effort

## 🐛 Bug Reports

Please report issues on our [GitHub repository](https://github.com/EffortlessMetrics/rust-sitter/issues).

## 📅 Future Plans

- Performance optimizations using SIMD
- Language server protocol integration
- More built-in grammar optimizations
- Extended WASM support
- Grammar synthesis from examples

---

**Full Changelog**: [v0.4.5...v0.5.0](https://github.com/EffortlessMetrics/rust-sitter/compare/v0.4.5...v0.5.0)