# Known Limitations and Upcoming Features

This document outlines the current limitations of rust-sitter v0.5.0-beta and the planned features for future releases.

## ✅ Supported Features

rust-sitter v0.5.0-beta provides a pure-Rust implementation of Tree-sitter with the following capabilities:

- **Core Grammar Features**
  - Sequences, choices, repeats, and optionals
  - String and regex patterns
  - Field names and aliases
  - Tokens and immediate tokens
  - Basic rules and symbols

- **Parser Generation**
  - LR(1) automaton construction
  - Table compression matching Tree-sitter format
  - NODE_TYPES.json generation
  - FFI-compatible Language struct

- **Runtime Features**
  - Parse tree construction
  - Error recovery strategies
  - Visitor API for tree traversal
  - Serialization in multiple formats

## ⚠️ Known Limitations

The following Tree-sitter features are not yet supported in v0.5.0-beta:

### Grammar Features

1. **Precedence and Associativity** ❌
   - `prec(level, rule)`
   - `prec.left(level, rule)`
   - `prec.right(level, rule)`
   - `prec.dynamic(level, rule)`
   - **Impact**: Cannot parse grammars with operator precedence (JavaScript, C, Python, etc.)
   - **Workaround**: None - these grammars will fail to build

2. **Word Token** ❌
   - `word: $ => $.identifier` declarations
   - **Impact**: Cannot properly handle keyword vs identifier distinction
   - **Workaround**: None - grammars using word tokens will fail

3. **External Scanners** ❌
   - `externals: $ => [...]` declarations
   - External C/C++ scanner integration
   - **Impact**: Cannot parse context-sensitive tokens (Python indentation, C++ raw strings)
   - **Workaround**: None - grammars with externals will fail

4. **Conflicts** ⚠️
   - `conflicts: $ => [[...]]` declarations
   - **Impact**: Explicit conflict resolution not supported
   - **Workaround**: Grammar may still work if conflicts are naturally resolved

5. **Supertypes** ⚠️
   - `supertypes: $ => [...]` declarations
   - **Impact**: Type hierarchy information missing from NODE_TYPES.json
   - **Workaround**: Grammar will build but without supertype metadata

### CLI Features

- No `tree-sitter` CLI compatibility yet
- No `tree-sitter generate` equivalent
- No `tree-sitter parse` equivalent
- No `tree-sitter test` equivalent

### Advanced Features

- No query language support yet
- No syntax highlighting queries
- No incremental parsing
- No cancellation support
- Limited WASM support (builds but not optimized)

## 📊 Grammar Compatibility Status

| Grammar | Status | Blocking Feature |
|---------|--------|------------------|
| JSON | ✅ Working | - |
| TOML | ✅ Working | - |
| INI | ✅ Working | - |
| Arithmetic | ✅ Working | - |
| JavaScript | ❌ Blocked | Precedence, word, externals |
| TypeScript | ❌ Blocked | Precedence, word, externals, supertypes |
| Python | ❌ Blocked | Precedence, word, externals (indentation) |
| Rust | ❌ Blocked | Precedence, word, externals |
| C | ❌ Blocked | Precedence, word |
| C++ | ❌ Blocked | Precedence, word, externals |
| Go | ❌ Blocked | Precedence, word |
| Ruby | ❌ Blocked | Precedence, word, externals |
| Java | ❌ Blocked | Precedence, word |
| C# | ❌ Blocked | Precedence, word, externals |

## 🚀 Roadmap

### v0.6.0 (Target: Q2 2025)
- ✨ Precedence and associativity support
- ✨ Word token support
- ✨ Basic CLI tool (`rust-sitter generate`, `rust-sitter parse`)
- 📈 Support for ~60% of popular grammars

### v0.7.0 (Target: Q3 2025)
- ✨ External scanner support
- ✨ Conflicts and supertypes
- ✨ Query language basics
- 📈 Support for ~90% of popular grammars

### v0.8.0 (Target: Q4 2025)
- ✨ Full query language support
- ✨ Incremental parsing
- ✨ Performance optimizations
- ✨ WASM optimizations
- 📈 Support for 100% of popular grammars

### v1.0.0 (Target: Q1 2026)
- 🎯 Full Tree-sitter compatibility
- 🎯 Performance parity or better than C implementation
- 🎯 Production-ready stability
- 🎯 Comprehensive documentation

## 🤝 Contributing

We welcome contributions! If you're interested in implementing any of these features:

1. Check the [GitHub issues](https://github.com/yourusername/rust-sitter/issues) for existing work
2. Open an issue to discuss your approach
3. Submit a PR with tests

Priority areas for contribution:
- Precedence/associativity implementation in the IR
- Grammar.js parser improvements
- CLI tool development
- Grammar compatibility testing

## 📞 Contact

For questions or feedback:
- GitHub Issues: [rust-sitter/issues](https://github.com/yourusername/rust-sitter/issues)
- Discussions: [rust-sitter/discussions](https://github.com/yourusername/rust-sitter/discussions)