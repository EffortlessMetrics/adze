# Rust Sitter Roadmap

This document outlines the current status and future direction of the rust-sitter project.

## ✅ Completed Features (v1.0)

### Core Parser Infrastructure
- [x] Pure-Rust LR(1) parser generator
- [x] GLR (Generalized LR) parsing for ambiguous grammars
- [x] Complete Tree-sitter ABI compatibility (v15)
- [x] Zero-copy parsing with efficient memory layout
- [x] Full Unicode support

### Language Features
- [x] External scanner support (FFI and native Rust)
- [x] Built-in scanners (indentation, heredoc)
- [x] Query language (S-expressions) with pattern matching
- [x] Syntax highlighting support
- [x] Field names and metadata
- [x] Precedence and associativity
- [x] Dynamic precedence

### Advanced Features
- [x] Incremental parsing with O(log n) complexity
- [x] Error recovery with multiple strategies
- [x] Parse tree visitors and transformers
- [x] Grammar optimization and validation
- [x] Table compression (Tree-sitter compatible)

### Build System
- [x] Procedural macro for grammar definition
- [x] Build-time code generation
- [x] Scanner discovery and compilation
- [x] WASM target support

### Developer Experience
- [x] Comprehensive error messages
- [x] Grammar visualization tools
- [x] Debug output for parser states
- [x] Snapshot testing support

## 🚧 In Progress (v1.1)

### Performance Optimizations
- [ ] SIMD acceleration for lexing
- [ ] Parallel parsing for large files
- [ ] Memory pool allocators
- [ ] Profile-guided optimization

### Enhanced Error Recovery
- [ ] Machine learning-based recovery
- [ ] Context-aware error messages
- [ ] Quick fix suggestions

### Tooling
- [ ] Language Server Protocol (LSP) generator
- [ ] VS Code extension generator
- [ ] Interactive grammar playground
- [ ] Performance profiler

## 📋 Future Plans (v2.0+)

### Next-Generation Features
- [ ] Incremental compilation for grammars
- [ ] Streaming parser for gigabyte files
- [ ] GPU-accelerated parsing
- [ ] Real-time collaborative parsing

### Language Support
- [ ] First-class support for layout-sensitive languages
- [ ] Template/macro expansion handling
- [ ] Multi-dialect grammar support
- [ ] Grammar composition and inheritance

### Integration
- [ ] Native bindings for other languages (Python, JavaScript)
- [ ] Cloud-based grammar repository
- [ ] GitHub Actions for grammar testing
- [ ] Package manager for reusable grammar components

### Research
- [ ] Formal verification of parser correctness
- [ ] Automatic grammar inference from examples
- [ ] Natural language grammar specifications
- [ ] Quantum parsing algorithms (experimental)

## Migration Path

For users migrating from Tree-sitter:

1. **Immediate**: Drop-in replacement for most grammars
2. **Short-term**: Migration tools for complex grammars
3. **Long-term**: Native rust-sitter features for enhanced functionality

## Contributing

We welcome contributions in the following areas:

- Grammar implementations for new languages
- Performance optimizations
- Documentation and tutorials
- Bug reports and feature requests

See [CONTRIBUTING.md](./CONTRIBUTING.md) for details.

## Timeline

- **Q1 2024**: v1.0 Release (Complete) ✅
- **Q2 2024**: v1.1 Performance Update
- **Q3 2024**: v1.2 Tooling Ecosystem
- **Q4 2024**: v2.0 Planning
- **2025**: v2.0 Next-Generation Features

## Community

- GitHub Discussions: Feature requests and questions
- Discord: Real-time chat and support
- Blog: Updates and tutorials
- Twitter: Announcements

## License

Rust Sitter is dual-licensed under MIT and Apache 2.0, maintaining compatibility with Tree-sitter's MIT license.