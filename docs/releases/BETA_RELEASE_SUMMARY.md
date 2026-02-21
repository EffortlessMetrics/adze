# Adze v0.5.0-beta Release Summary

## 🎉 Release Overview

Adze v0.5.0-beta is the first public beta release of a revolutionary approach to parser generation that allows defining Tree-sitter-compatible grammars directly in Rust using derive macros.

## 🚀 Key Features

### Write Grammars in Rust
```rust
#[adze::grammar("my_language")]
pub mod grammar {
    #[adze::language]
    pub struct Program {
        #[adze::repeat]
        pub statements: Vec<Statement>,
    }
}
```

### Automatic Parser Generation
- Extracts grammar from Rust types at build time
- Generates Tree-sitter JSON grammar
- Compiles to efficient C parser
- Provides Rust API for parsing

### Pure Rust Foundation
- Core parsing engine written in Rust
- GLR algorithm support for ambiguous grammars
- Comprehensive error recovery
- Type-safe AST generation

## 📦 What's Included

### Core Crates
- **adze** - Runtime parsing library
- **adze-macro** - Grammar definition macros
- **adze-tool** - Build-time parser generation
- **adze-cli** - Command-line interface

### Example Grammars
- JavaScript (simplified subset)
- Python (basic syntax)
- Go (minimal subset)
- Comprehensive tutorial example

### Documentation
- Quick start guide (QUICKSTART_BETA.md)
- Grammar examples (GRAMMAR_EXAMPLES.md)
- Migration guide from Tree-sitter
- API documentation

## ⚠️ Beta Limitations

This is a **beta release** with known limitations:

### Not Yet Supported
- Precedence and associativity declarations
- Full external scanner API
- Some Tree-sitter keywords (word, extras, conflicts)
- Complex grammar features (prec.dynamic, alias)

### Known Issues
- 8 failing tests in runtime (query/scanner related)
- Playground crate has compilation errors
- Python grammar scanner not fully compatible

### Workarounds Required
- Binary expressions need manual conflict resolution
- Optional expressions may cause ambiguity
- Complex grammars require simplification

## 🎯 Target Audience

This beta is ideal for:
- Early adopters wanting to experiment
- Simple DSL implementations
- Educational projects
- Prototyping new languages
- Rust enthusiasts interested in parsing

## 💻 Getting Started

```toml
[dependencies]
adze = "0.5.0-beta"

[build-dependencies]
adze-tool = "0.5.0-beta"
```

See QUICKSTART_BETA.md for a complete tutorial.

## 🔮 Future Roadmap

### v0.6.0 (Next Release)
- Full precedence/associativity support
- Complete external scanner API
- Fix all failing tests
- Performance optimizations

### v0.7.0
- Full Tree-sitter compatibility
- Advanced conflict resolution
- Incremental parsing improvements

### v1.0.0
- Production-ready stability
- WASM support
- Language bindings
- IDE integrations

## 📣 Call to Action

We need your feedback! Please:
- Try building a simple grammar
- Report issues on GitHub
- Share your experience
- Contribute improvements

## 🙏 Acknowledgments

This beta represents months of work reimagining how parsers can be generated. While not yet feature-complete, it demonstrates the potential of the Rust-based approach.

Thank you to all early testers and contributors who helped shape this release.

---

**Remember**: This is a beta release. Expect breaking changes and missing features. But also expect a glimpse into the future of parser generation in Rust!

Happy parsing! 🦀🌳