# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Development Commands

### Building
```bash
# Build all workspace members
cargo build

# Build with release optimizations
cargo build --release

# Build a specific package
cargo build -p rust-sitter
cargo build -p rust-sitter-macro
cargo build -p rust-sitter-tool
```

### Testing
```bash
# Run all tests in the workspace
cargo test

# Run tests for a specific package
cargo test -p rust-sitter
cargo test -p rust-sitter-macro
cargo test -p rust-sitter-tool

# Run a specific test
cargo test test_name

# Run tests with output displayed
cargo test -- --nocapture

# Update snapshot tests (uses insta)
cargo insta review
```

### Linting and Formatting
```bash
# Run clippy on all workspace members
cargo clippy --all

# Run clippy and fail on warnings
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting without making changes
cargo fmt -- --check
```

## Architecture Overview

Rust Sitter is a Rust workspace consisting of multiple interconnected crates that work together to generate Tree-sitter parsers from Rust code annotations:

### Core Components

1. **`rust-sitter` (runtime crate)** - The main runtime library that users depend on
   - Located in `/runtime/`
   - Provides the `Extract` trait and core parsing functionality
   - Supports two Tree-sitter backends via features:
     - `tree-sitter-c2rust` (default): Pure Rust implementation for WASM support
     - `tree-sitter-standard`: Standard C runtime

2. **`rust-sitter-macro` (proc-macro crate)** - Procedural macros for grammar definition
   - Located in `/macro/`
   - Provides attributes: `#[rust_sitter::grammar]`, `#[rust_sitter::language]`, `#[rust_sitter::leaf]`, etc.
   - Only defines the macro interfaces; actual expansion logic is in common crate

3. **`rust-sitter-tool` (build tool)** - Build-time code generation
   - Located in `/tool/`
   - Called from `build.rs` to generate Tree-sitter grammar JSON and C parser code
   - Key function: `build_parsers()` which processes annotated Rust files

4. **`rust-sitter-common`** - Shared utilities
   - Located in `/common/`
   - Contains grammar expansion logic used by both macro and tool crates

5. **`example`** - Example grammars and usage patterns
   - Located in `/example/`
   - Contains arithmetic, optional, repetition, and word grammar examples
   - Uses snapshot testing with `insta` for parser output verification

### New Pure-Rust Implementation Components

6. **`rust-sitter-ir`** - Grammar Intermediate Representation
   - Located in `/ir/`
   - Defines the IR for representing grammars with GLR support
   - Supports precedence, associativity, field mappings, and fragile tokens

7. **`rust-sitter-glr-core`** - GLR Parser Generation Core
   - Located in `/glr-core/`
   - Implements FIRST/FOLLOW set computation
   - LR(1) item sets and canonical collection building
   - Conflict detection and GLR fork/merge logic

8. **`rust-sitter-tablegen`** - Table Generation and Compression
   - Located in `/tablegen/`
   - Implements Tree-sitter's table compression algorithms
   - Generates static Language objects with FFI compatibility
   - Produces NODE_TYPES JSON metadata

### Key Design Patterns

1. **Grammar Definition Flow**:
   - User defines grammar using Rust types with macro annotations
   - `build.rs` calls `rust_sitter_tool::build_parsers()` at build time
   - Tool extracts grammar from Rust code and generates Tree-sitter JSON grammar
   - Tree-sitter generates C parser from JSON
   - C parser is compiled and linked into the final binary

2. **Two-Stage Processing**:
   - Compile-time: Macros mark types but don't generate parser code
   - Build-time: Tool reads the marked types and generates actual parser

3. **Environment Variables**:
   - `RUST_SITTER_EMIT_ARTIFACTS=true`: Outputs generated grammar files to `target/debug/build/<crate>-<hash>/out/` for debugging

### Working with the Codebase

When making changes:
1. Grammar expansion logic is shared between macro and tool in the `common` crate
2. The macro crate only provides attribute definitions, not implementations
3. The tool crate handles all build-time code generation
4. Test changes using the example crate which has comprehensive snapshot tests
5. Use `cargo insta review` to update snapshots when grammar output changes intentionally

### Pure-Rust Implementation Development

When working on the pure-Rust implementation:
1. The IR crate defines the grammar representation - modify this for new grammar features
2. The GLR core implements the parser generation algorithms - this is where conflict resolution happens
3. The tablegen crate handles compression - ensure bit-for-bit compatibility with Tree-sitter
4. Use `emit_ir!()` macro to debug grammar extraction
5. Test table generation with `cargo test -p rust-sitter-tablegen`
6. Verify Language struct layout matches Tree-sitter ABI exactly

### Testing Guidelines

1. **Grammar Tests**: Add new grammars to `/example/src/` with corresponding snapshot tests
2. **Compression Tests**: Verify table compression maintains Tree-sitter compatibility
3. **FFI Tests**: Ensure generated Language structs match C ABI requirements
4. **Integration Tests**: Test with real Tree-sitter grammars for validation