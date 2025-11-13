# Known Limitations and Upcoming Features

This document outlines the current limitations of rust-sitter v0.8.0-dev and the planned features for future releases.

## ✅ Supported Features

rust-sitter v0.8.0-dev provides a pure-Rust implementation of Tree-sitter with the following capabilities:

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

## ⚠️ Critical Issues in v0.8.0-dev

### 🚨 Transform Function Execution ❌ (BLOCKING)
   - **Issue**: Custom lexer type conversion not fully implemented
   - **Impact**: Grammars with number literals, strings, and identifier transforms fail to parse
   - **Examples**: Python-simple tests (6 tests failing) - basic arithmetic fails
   - **Affected**: Most real-world grammars using `transform` functions
   - **Workaround**: None currently available
   - **Reference**: CRITICAL_ISSUES_SUMMARY.md - Issue #74

### 🚨 Performance Benchmarks ❌ (DOCUMENTATION)
   - **Issue**: Current benchmarks measure character iteration mocks, not actual parsing
   - **Claims**: "815 MB/sec", "100x faster than Tree-sitter"
   - **Reality**: No real parsing happening in benchmarks
   - **Reference**: CRITICAL_ISSUES_SUMMARY.md - Issue #73

## ⚠️ Known Limitations

The following Tree-sitter features are not yet supported in v0.8.0-dev:

### Grammar Features

1. **External Scanner Runtime** ❌
   - External scanner declarations are parsed but not executed
   - External C/C++ scanner integration not implemented
   - **Impact**: Cannot parse context-sensitive tokens (Python indentation, C++ raw strings)
   - **Workaround**: None - grammars requiring external scanners will fail at runtime
   - **Status**: Planned for future release

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

### v0.8.0-dev (Current - November 2025)
**Status**: Active development with architecture in place, critical gaps identified

**Completed**:
- ✅ GLR parser architecture and design
- ✅ Grammar macro system and compile-time generation
- ✅ LR(1) automaton construction
- ✅ MSRV updated to 1.89 (Rust 2024 edition)
- ✅ Precedence and associativity support
- ✅ Word token support
- ✅ External/supertypes/conflicts parsing

**Critical Work In Progress**:
- ❌ Transform function execution (blocking most grammars)
- ❌ Real performance benchmarks (current benchmarks are mocks)
- ❌ External scanner runtime implementation

**Next Release Priority**:
1. Fix transform function execution (3-4 weeks)
2. Implement real performance benchmarks (2 weeks)
3. External scanner support (4-6 weeks)
4. Comprehensive grammar testing and certification

### v0.9.0 (Target: Q2 2025)
- ✨ Transform function execution (CRITICAL)
- ✨ Real performance benchmarks
- ✨ 50+ grammar compatibility testing
- 📈 Support for ~50% of popular grammars (honest assessment)

### v1.0.0 (Target: Q4 2025)
- ✨ External scanner runtime support
- ✨ Full query language support
- ✨ Incremental parsing (tested with real grammars)
- ✨ CLI tool compatibility
- 📈 Support for ~90% of popular grammars
- 🎯 Production-ready for simple grammars

### v1.1.0+ (Future)
- 🎯 Full Tree-sitter compatibility
- 🎯 Performance parity with C implementation
- 🎯 Comprehensive documentation and examples

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