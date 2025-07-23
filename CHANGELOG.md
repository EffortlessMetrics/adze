# Changelog

All notable changes to this project will be documented in this file.

## [0.5.0-beta] - 2025-01-23

### 🎉 Major Milestone: Pure-Rust Implementation

This beta release introduces a complete pure-Rust Tree-sitter implementation that eliminates all C dependencies while maintaining compatibility with the Tree-sitter ecosystem.

### Added

#### Core Infrastructure
- **Pure-Rust Parser Generator**: Complete GLR (Generalized LR) parser generator implementation
- **Grammar IR**: Intermediate representation for grammars with full Tree-sitter feature support
- **Table Generation**: Tree-sitter compatible table compression and Language struct generation
- **FFI Compatibility**: Bit-for-bit compatible Language structs with C Tree-sitter

#### Parser Features
- **LR(1) Automaton**: Full LR(1) parser generation with FIRST/FOLLOW set computation
- **GLR Support**: Generalized LR parsing for handling ambiguous grammars
- **Error Recovery**: Comprehensive error recovery strategies
- **Conflict Resolution**: Advanced conflict resolution mechanisms
- **Grammar Optimization**: Multiple optimization passes for generated parsers

#### Development Tools
- **CLI Tools**: Complete command-line interface for grammar development
  - `rust-sitter init` - Initialize new grammar projects
  - `rust-sitter build` - Build grammar parsers with watch mode
  - `rust-sitter parse` - Parse files using grammars
  - `rust-sitter test` - Run grammar tests
  - `rust-sitter doc` - Generate grammar documentation
  - `rust-sitter check` - Validate grammar syntax
  - `rust-sitter stats` - Show grammar statistics
- **LSP Generator**: Create language servers from grammars
- **Interactive Playground**: Web-based grammar testing environment
- **Golden Tests**: Comprehensive test infrastructure with `cargo xtask`
- **Grammar Visualization**: Tools for visualizing grammars and parse trees
- **Performance Benchmarking**: Built-in benchmarking infrastructure
- **Migration Guide**: Documentation for migrating from C-based Tree-sitter

#### Runtime Features
- **Visitor API**: Parse tree visitor for traversal and analysis
- **Serialization**: Multiple serialization formats for parse trees
- **NODE_TYPES.json**: Exact compatibility with Tree-sitter node type generation

### Grammar Support

#### Fully Supported
- JSON grammar ✅
- TOML grammar ✅
- Simple expression grammars ✅
- Basic token patterns ✅
- Arithmetic expressions with operators ✅
- Optional fields and repetitions ✅
- String literals and identifiers ✅

#### Partially Supported (Coming in future releases)
- JavaScript grammar (requires precedence, word rules, externals)
- Python grammar (requires externals for indentation)
- Complex grammars with advanced features

### Known Limitations

This beta release does not yet support:
- Precedence and associativity (`prec`, `prec.left`, `prec.right`)
- Word token declarations
- External scanners
- Conflicts array
- Supertypes
- Query language
- Incremental parsing

See [KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md) for full details.

### Breaking Changes

- Minimum Rust version is now 1.70.0
- Some internal APIs have changed (see migration guide)

### Contributors

Thank you to everyone who contributed to making this pure-Rust implementation possible!

## Previous Releases

### [0.4.5] - 2024-XX-XX
- Last release before pure-Rust implementation
- Bug fixes and minor improvements