# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2025-01-23

### Added

#### Core Features
- **Grammar Optimization** - Automatic optimization passes to improve parser performance
  - Remove unused symbols and rules
  - Inline simple rules
  - Merge duplicate token patterns
  - Optimize left recursion
- **Error Recovery Strategies** - Comprehensive error recovery for robust parsing
  - Panic mode with synchronization tokens
  - Token insertion/deletion/substitution
  - Phrase-level recovery
  - Scope-based recovery
  - Indentation-based recovery
- **Conflict Resolution** - Advanced conflict resolution for GLR parsing
  - Precedence-based resolution
  - Associativity handling
  - Detailed conflict statistics
- **Grammar Validation** - Early detection of grammar issues
  - Undefined symbol detection
  - Unreachable rule analysis
  - Cycle detection
  - Field validation
- **Parse Tree Visitors** - Flexible API for tree traversal
  - Depth-first and breadth-first traversal
  - Built-in visitor implementations
  - Custom visitor support
- **Tree Serialization** - Multiple serialization formats
  - JSON (full and compact)
  - S-expressions
  - Binary format
- **Visualization Tools** - Grammar and tree visualization
  - Graphviz DOT generation
  - Railroad diagrams
  - ASCII art representation

#### New Crates
- `rust-sitter-ir` - Grammar intermediate representation with optimization and validation
- `rust-sitter-glr-core` - GLR parser generation core with conflict resolution
- `rust-sitter-tablegen` - Table generation and compression

#### Documentation
- Comprehensive API documentation
- Migration guide from C-based Tree-sitter
- Extensive usage examples
- Performance benchmarks

### Changed
- Grammar definition now uses Rust syntax instead of JavaScript
- Parser API uses type parameters for better type safety
- Error types redesigned for better ergonomics
- Improved macro syntax for grammar definition

### Fixed
- Memory leaks in incremental parsing
- Edge cases in error recovery
- Grammar extraction for complex type patterns

### Performance
- Parsing performance improved by ~20% for complex grammars
- Memory usage reduced through better allocation strategies
- Table compression reduces binary size

## [0.4.5] - 2024-06-15

### Added
- Support for external scanners
- Incremental parsing improvements

### Fixed
- Build issues on Windows
- Grammar extraction edge cases

## [0.4.4] - 2024-03-10

### Added
- WASM support improvements
- Better error messages

### Changed
- Updated dependencies

## [0.4.3] - 2024-01-05

### Fixed
- Macro hygiene issues
- Build script reliability

## [0.4.2] - 2023-11-20

### Added
- Support for hidden rules
- Field name extraction

### Fixed
- Grammar generation for recursive types

## [0.4.1] - 2023-09-15

### Fixed
- Packaging issues
- Documentation updates

## [0.4.0] - 2023-08-01

### Added
- Initial pure-Rust implementation
- Macro-based grammar definition
- Basic parser functionality

### Changed
- Complete rewrite from scratch
- New API design

## [0.3.0] - 2023-05-01

### Added
- Tree-sitter 0.20 compatibility
- New node types

### Changed
- API improvements

## [0.2.0] - 2023-02-01

### Added
- Basic Tree-sitter bindings
- Initial grammar support

## [0.1.0] - 2022-11-01

### Added
- Initial release
- Basic functionality

[0.5.0]: https://github.com/hydro-project/rust-sitter/compare/v0.4.5...v0.5.0
[0.4.5]: https://github.com/hydro-project/rust-sitter/compare/v0.4.4...v0.4.5
[0.4.4]: https://github.com/hydro-project/rust-sitter/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/hydro-project/rust-sitter/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/hydro-project/rust-sitter/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/hydro-project/rust-sitter/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/hydro-project/rust-sitter/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/hydro-project/rust-sitter/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/hydro-project/rust-sitter/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/hydro-project/rust-sitter/releases/tag/v0.1.0