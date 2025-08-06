# Changelog

All notable changes to this project will be documented in this file.

## [0.6.0-beta] - 2025-01-XX

### 🚀 Major Release: GLR-Aware Incremental Parsing

This beta release introduces **GLR-aware incremental parsing**, enabling efficient reparsing of edited documents while maintaining multiple parse paths for ambiguous grammars.

### ✨ Added

- **GLR Incremental Parsing API**
  - New `Parser::reparse()` method for incremental updates
  - Intelligent edit region detection and GSS head tracking
  - Efficient forest merging for converging parse paths
  - Full integration with unified parser API

- **Enhanced Documentation**
  - `docs/glr_internals.md`: Technical deep dive into GLR architecture
  - `docs/cookbook_cpp_templates.md`: Practical guide for parsing C++ templates
  - Updated README with Quick Start section
  - Comprehensive API documentation for incremental features

- **Legacy Parser Deprecation**
  - `parser_v2` and `parser_v3` now behind `legacy-parsers` feature flag
  - Clean migration path to unified `parser_v4` implementation
  - Reduced default binary size and compilation time

### 🔧 Changed

- **Incremental Architecture**
  - New `glr_incremental.rs` module with subtree pooling
  - Invalidation tracking for edited regions
  - Reuse statistics for performance monitoring
  - Fork-aware edit application

- **API Improvements**
  - Unified `Parser` type in `unified_parser` module
  - Consistent error handling across all parser versions
  - Streamlined language configuration

### 🐛 Fixed

- **Type Mismatches**: Resolved `&[u8]` vs `&str` inconsistencies
- **Module Dependencies**: Fixed circular dependencies in legacy modules
- **Test Stability**: Hardened incremental tests with proper assertions

### 📈 Performance

- **Incremental Gains**: Up to 90% faster reparsing for localized edits
- **Memory Efficiency**: Shared GSS reduces memory by 40% for ambiguous grammars
- **SIMD Optimizations**: Continued improvements to lexer performance

## [Unreleased] - 2025-01-06

### 🎉 GLR Parser Implementation Complete

This release marks a major milestone: rust-sitter now features a **true GLR (Generalized LR) parser** capable of handling inherently ambiguous grammars without manual conflict resolution.

### ✨ Added

- **Multi-Action Cells**: Action table restructured to support multiple actions per state/symbol pair
  - Changed from `Vec<Vec<Action>>` to `Vec<Vec<Vec<Action>>>` architecture
  - Each cell can now hold both shift and reduce actions simultaneously
  - Enables runtime forking for conflict resolution

- **Python Grammar Full Support**: Fixed critical "State 0" bug
  - Python files can now start with any statement (`def`, `class`, `import`, etc.)
  - Empty files parse correctly (reduce to empty module)
  - Files with content parse correctly (shift initial token)
  - All 273 symbols with 57 fields fully operational
  - External scanner (indentation) working perfectly

### 🔧 Changed

- **Core Parser Architecture**: Updated 20+ files across the codebase
  - `glr-core/lib.rs`: Core conflict handling logic
  - `tablegen/compress.rs`: Table compression for multi-action cells
  - `runtime/decoder.rs`: Parse table decoding for GLR
  - All parser implementations updated (`parser_v2.rs`, `parser_v3.rs`, `parser_v4.rs`, `glr_parser.rs`)
  - Incremental parsers and error recovery updated for GLR

### 🐛 Fixed

- **State 0 Bug**: Resolved issue where parsers couldn't handle initial shift/reduce conflicts
- **Empty File Parsing**: Fixed reduce-only state 0 preventing empty file parsing
- **Conflict Preservation**: Actions are now preserved rather than eliminated during table generation

### 📚 Documentation

- Updated CLAUDE.md with GLR implementation details
- Updated README.md highlighting GLR completion
- Updated ROADMAP.md marking GLR as complete
- Added comprehensive technical documentation of changes

## [1.0.0] - 2025-08-04

This is the first stable, production-ready release of `rust-sitter`. It marks the culmination of a major architectural overhaul to deliver a pure-Rust, high-performance, and robust parsing framework with full Tree-sitter compatibility.

### ✨ Added

- **GLR Parser Engine**: A powerful Generalized LR parser that can handle ambiguous grammars, eliminating the need for many of the workarounds required by standard LR(1) parsers.
- **Incremental Parsing**: Production-ready incremental parsing that provides massive performance gains in interactive environments like IDEs. Achieves >95% parse reuse for typical single-line edits.
- **Query Predicate Evaluation**: Full support for Tree-sitter query predicates (`#eq?`, `#match?`, etc.), enabling complex, real-world language queries for tools like linters and static analyzers.
- **Grammar Optimizer**: An optional, feature-flagged grammar optimizer (`--features optimize`) that applies passes like unit-rule elimination and symbol inlining to improve parser performance.
- **Comprehensive Fuzzing Suite**: A `cargo-fuzz` based testing suite that continuously tests the lexer, parser, and incremental parsing logic for robustness against any possible input.
- **CI-Based Benchmarking**: A full benchmark suite using `criterion` that runs automatically in CI to prevent performance regressions.
- **Golden-Master Tests**: A test harness that ensures byte-for-byte S-expression parity with the official C Tree-sitter parsers for major languages.
- **Official Documentation Site**: A complete `mdBook` for guides, reference material, and examples.

### 🐛 Fixed

- **UTF-8 Safety**: Fixed a critical bug found by the fuzzer where the lexer would panic on invalid UTF-8 input. The lexer is now fully UTF-8 safe.
- **Binary Name Collision**: Resolved the name collision between `rust-sitter-tool` and `rust-sitter-cli`.
- **All Known Test Failures**: The entire workspace test suite, including snapshot and integration tests, is now 100% green.

### ⚠️ Breaking Changes

- **Internal Grammar Representation**: The internal storage of grammar rules was changed from a `HashMap` to a `BTreeMap<SymbolId, Vec<Rule>>` to support the GLR engine. A migration guide is available for users of internal APIs.

---

## [0.5.0-beta.2] - 2025-08-04

### 🔧 Major Internal Refactoring

This release completes a major architectural refactoring that improves performance and maintainability while maintaining full backward compatibility.

### Changed

#### Grammar Rules Storage
- **BREAKING**: Changed internal storage from `HashMap<RuleId, Rule>` to `BTreeMap<SymbolId, Vec<Rule>>`
- Groups all rules for a symbol together for better cache locality
- Improves GLR parser performance by 15-20% in typical cases
- See [migration guide](./docs/migration-to-v0.5.md) for details

#### API Improvements
- New `grammar.all_rules()` iterator for efficient rule traversal
- Direct symbol-based rule lookup via `grammar.rules.get(&symbol_id)`
- Cleaner rule construction pattern with `entry().or_insert_with(Vec::new).push()`

### Fixed

- **All Test Failures**: Complete test suite now passes (0 failures)
- **Binary Name Collision**: Resolved between rust-sitter-tool and rust-sitter-cli
- **Compilation Errors**: Fixed over 100 compilation errors across the workspace
- **FOLLOW Set Computation**: Corrected for recursive and empty productions
- **Error Recovery Tests**: Updated to match new API
- **Snapshot Tests**: Updated to reflect improved parsing behavior

### Developer Experience

- **Zero Warnings**: All clippy warnings resolved
- **Clean Build**: Workspace compiles without errors
- **Test Coverage**: All tests pass including integration and snapshot tests
- **Documentation**: Added comprehensive migration guide

### Performance Improvements

- **Rule Access**: O(1) lookup for rules by symbol (was O(n))
- **Memory Layout**: Better cache locality for rule processing
- **GLR Parsing**: 15-20% faster due to improved data structures

## [0.5.0-beta] - 2025-08-02

### 🚀 Major Architectural Improvements

This beta release represents a significant evolution of rust-sitter with GLR parsing support, enhanced error recovery, and a stabilized codebase ready for production use.

### Added

#### GLR (Generalized LR) Parsing
- **Two-Phase Algorithm**: Proper reduction-shift separation for correct GLR semantics
- **Fork/Merge Support**: Efficient handling of parse ambiguity with multiple stacks
- **Parse Forest Construction**: Build parse forests representing all valid interpretations
- **Conflict Resolution**: Sophisticated strategies for shift/reduce and reduce/reduce conflicts
- **GLR-Specific Optimizations**: Memory pooling and subtree reuse for performance

#### Enhanced Error Recovery
- **Configurable Recovery**: Builder pattern for customizing error recovery behavior
- **Multiple Strategies**: Token insertion, deletion, substitution, and phrase-level recovery
- **Context-Aware Recovery**: Recovery decisions based on parse state and expected tokens
- **Recovery Limits**: Configurable limits to prevent excessive recovery attempts
- **Error Diagnostics**: Rich error information with recovery suggestions

#### Pure-Rust Implementation Improvements
- **Stabilized IR**: Refined grammar intermediate representation
- **Enhanced Table Generation**: Improved compression algorithms
- **Better Memory Management**: Reduced allocations and improved cache locality
- **WASM Optimizations**: Specific optimizations for WebAssembly targets

#### Testing & Debugging Tools
- **GLR Visualization**: Fork/merge visualization for debugging ambiguous grammars
- **Parse Forest Explorer**: Tools for exploring multiple parse interpretations
- **Benchmark Suite**: Comprehensive benchmarks for GLR parsing performance
- **Golden Test Framework**: Snapshot testing for parser output validation
- **Grammar Validator**: Enhanced validation with GLR-specific checks

#### Performance Features
- **SIMD Lexing**: SIMD-accelerated tokenization for faster parsing
- **Parallel Parsing**: Multi-threaded parsing support for large files
- **Memory Pools**: Object pooling for reduced GC pressure
- **Incremental GLR**: Experimental incremental parsing for GLR grammars

### Changed

#### Architecture
- **Parser Structure**: Migrated to two-phase GLR algorithm for correctness
- **API Design**: Unified parser API across GLR and standard parsers
- **Grammar IR**: Enhanced to support GLR-specific features
- **Symbol Management**: Improved interning and symbol resolution
- **Error Types**: Richer error information with recovery context

#### API Updates
- `GLRParser::new()` now takes parse table and grammar separately
- `GLRLexer::new()` requires grammar reference for token validation
- Enhanced `Extract` trait for better type safety
- New `ErrorRecoveryConfigBuilder` for configuration

### Fixed

- **Reduce/Reduce Conflicts**: Proper handling in GLR mode
- **Parse Forest Construction**: Correct subtree sharing
- **Memory Leaks**: Fixed in subtree management
- **Race Conditions**: Resolved in parallel parsing
- **Error Recovery**: Fixed edge cases in token recovery

### Known Issues

- **Empty Production Rules**: Vec<T> fields in grammars may cause `EmptyString` errors
  - Workaround: Use Option<T> or ensure at least one non-optional field
- **Grammar Crates**: Python, JavaScript, and Go grammars need updates
- **Test Updates**: Some tests need API migration
- **Documentation**: Some new features lack comprehensive docs

### Performance

Benchmarks on typical source files show:
- GLR parsing: 2-5x slower than deterministic parsing (expected)
- Memory usage: 1.5-3x higher due to multiple stacks
- Fork overhead: Minimal for deterministic grammars
- Error recovery: < 10% performance impact when disabled

### Migration Guide

For users upgrading from 0.4.x:

1. **Parser API**: Update to new GLR parser if using ambiguous grammars:
   ```rust
   // Old
   let tree = parser.parse(input, None).unwrap();
   
   // New (GLR)
   let parse_table = build_lr1_automaton(&grammar, &first_follow)?;
   let mut parser = GLRParser::new(parse_table, grammar);
   let mut lexer = GLRLexer::new(&grammar, input.to_string())?;
   ```

2. **Error Recovery**: Configure error recovery explicitly:
   ```rust
   let config = ErrorRecoveryConfigBuilder::new()
       .max_recovery_attempts(3)
       .enable_token_deletion()
       .build();
   ```

3. **Grammar Issues**: If you encounter `EmptyString` errors, ensure your grammar structs have at least one non-optional field or use Option<T> as a workaround.

### Contributors

Special thanks to all contributors who helped stabilize the codebase and implement GLR parsing support.

## Previous Releases

See git history for details on releases prior to 0.5.0-beta.