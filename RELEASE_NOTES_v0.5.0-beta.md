# Release Notes: rust-sitter v0.5.0-beta

## 🎉 Major Milestone: Pure-Rust Tree-sitter Implementation

We're excited to announce the beta release of rust-sitter v0.5.0-beta, featuring a **complete pure-Rust implementation** of the Tree-sitter parser generator!

### What's New

This release eliminates all C dependencies while maintaining compatibility with the Tree-sitter ecosystem. Key features include:

- ✅ **Pure-Rust Parser Generator**: Complete GLR parser generation in Rust
- ✅ **Tree-sitter Compatible**: Bit-for-bit compatible Language structs and NODE_TYPES.json
- ✅ **Working Grammars**: JSON, TOML, INI, and other simple grammars fully supported
- ✅ **Comprehensive Testing**: Golden test infrastructure with cargo xtask
- ✅ **Developer Tools**: Grammar visualization, error recovery, and migration guides

### Getting Started

```toml
[dependencies]
rust-sitter = "0.5.0-beta"

[build-dependencies]
rust-sitter-tool = "0.5.0-beta"
```

### Current Limitations

This beta release supports basic to medium complexity grammars. The following features are coming in future releases:

- Precedence and associativity (`prec`, `prec.left`, `prec.right`)
- Word token declarations
- External scanners
- Query language
- Incremental parsing

See [KNOWN_LIMITATIONS.md](https://github.com/hydro-project/rust-sitter/blob/main/KNOWN_LIMITATIONS.md) for details.

### Grammar Compatibility

**Fully Supported:**
- JSON ✅
- TOML ✅
- INI ✅
- Simple expression grammars ✅

**Coming Soon:**
- JavaScript (requires precedence, word rules)
- Python (requires external scanners)
- C/C++, Go, Rust, Ruby, etc.

See the [Compatibility Dashboard](https://github.com/hydro-project/rust-sitter/blob/main/docs/compatibility-dashboard.md) for the full list.

### Migration from v0.4.x

Most code should work without changes. If you're using advanced features or internal APIs, see our [Migration Guide](https://github.com/hydro-project/rust-sitter/blob/main/docs/migration-guide.md).

### What's Next

- v0.6.0: Precedence, associativity, word tokens (~60% grammar support)
- v0.7.0: External scanners (~90% grammar support)
- v0.8.0: Query language, incremental parsing
- v1.0.0: Full Tree-sitter compatibility

### Feedback

This is a beta release - we need your feedback! Please:
- Try it with your grammars
- Report issues on [GitHub](https://github.com/hydro-project/rust-sitter/issues)
- Share your experience

### Contributors

Thank you to everyone who made this pure-Rust implementation possible!

---

**Ready to eliminate C dependencies from your parser?** Update to v0.5.0-beta today!