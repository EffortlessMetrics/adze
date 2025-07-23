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
  - **NEW**: Precedence and associativity (`prec`, `prec.left`, `prec.right`, `prec.dynamic`) ✨
  - **NEW**: Word token declarations ✨
  - **NEW**: External scanner declarations (parsing only) ✨
  - **NEW**: Supertypes declarations ✨
  - **NEW**: Inline rules ✨
  - **NEW**: Conflicts declarations ✨

- **Parser Generation**
  - LR(1) automaton construction
  - Table compression matching Tree-sitter format
  - NODE_TYPES.json generation
  - FFI-compatible Language struct
  - **NEW**: Non-sequential symbol ID handling ✨

- **Runtime Features**
  - Parse tree construction
  - Error recovery strategies
  - Visitor API for tree traversal
  - Serialization in multiple formats

## ⚠️ Known Limitations

The following Tree-sitter features are not yet supported in v0.5.0-beta:

### Grammar Features

1. **External Scanner Runtime** ❌
   - External scanner declarations are parsed but not executed
   - External C/C++ scanner integration not implemented
   - **Impact**: Cannot parse context-sensitive tokens (Python indentation, C++ raw strings)
   - **Workaround**: None - grammars requiring external scanners will fail at runtime

2. **Named Precedence Levels** ❌
   - `prec('operator', rule)` style precedences
   - **Impact**: Some grammars use named precedence for clarity
   - **Workaround**: Convert to numeric precedence levels

3. **JavaScript-style Function Blocks in grammar.js** ❌
   - `{ const table = [...]; return choice(...); }` patterns
   - **Impact**: Cannot parse some complex grammar patterns
   - **Workaround**: Rewrite as direct expressions

4. **Complex Extras Patterns** ⚠️
   - Extras with complex regex or choices
   - **Impact**: Some whitespace handling may not work correctly
   - **Workaround**: Simplify extras patterns

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
| C | 🟡 Likely Working | Needs testing |
| Go | 🟡 Likely Working | Needs testing |
| Java | 🟡 Likely Working | Needs testing |
| JavaScript | 🟡 Partial | External scanner runtime, JS function blocks |
| TypeScript | 🟡 Partial | External scanner runtime, extends JavaScript |
| Python | ❌ Blocked | External scanner runtime (indentation) |
| Rust | ❌ Blocked | External scanner runtime (raw strings) |
| C++ | ❌ Blocked | External scanner runtime (raw strings) |
| Ruby | ❌ Blocked | External scanner runtime (heredocs) |
| C# | ❌ Blocked | External scanner runtime |

## 🚀 Roadmap

### v0.6.0 (Target: Q1 2025)
- ✅ Precedence and associativity support (DONE)
- ✅ Word token support (DONE)
- ✅ External/supertypes/conflicts parsing (DONE)
- ✨ External scanner runtime implementation
- ✨ Basic CLI tool (`rust-sitter generate`, `rust-sitter parse`)
- 📈 Support for ~70% of popular grammars

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