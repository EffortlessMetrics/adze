# Rust Sitter
[![Crates.io](https://img.shields.io/crates/v/rust-sitter)](https://crates.io/crates/rust-sitter)

Rust Sitter makes it easy to create efficient parsers in Rust by leveraging the [Tree Sitter](https://tree-sitter.github.io/tree-sitter/) parser generator. With Rust Sitter, you can define your entire grammar with annotations on idiomatic Rust code, and let macros generate the parser and type-safe bindings for you!

> **v0.5.0-beta Status**: Rust Sitter is feature-complete and production-ready. All planned features have been implemented and thoroughly tested. The project is approaching the v1.0 stable release. Note: Some language grammars (Python, JavaScript, Go) require updates to handle empty production rules properly.

## Documentation

For a complete overview of the project, including guides, tutorials, and API reference, please see our [**comprehensive documentation**](./docs/SUMMARY.md).

## Key Features (v0.5.0-beta)

- **GLR Parsing**: Full support for ambiguous grammars with efficient fork/merge handling
- **Pure-Rust Option**: Generate static parsers at compile-time without C dependencies  
- **Enhanced Error Recovery**: Sophisticated error recovery strategies for robust parsing
- **Two-Phase Parser**: Proper reduction-shift separation for correct GLR semantics
- **Comprehensive Testing**: Golden tests, benchmarks, and validation infrastructure
- **WASM Support**: Full WebAssembly compatibility with the pure-Rust backend
- **Performance Optimizations**: SIMD lexing, parallel parsing, and memory pooling

## Installation
First, add Rust/Tree Sitter to your `Cargo.toml`:
```toml
[dependencies]
rust-sitter = "0.5.0-beta"

[build-dependencies]
rust-sitter-tool = "0.5.0-beta"
```

Choose your backend via features:
- `pure-rust` (recommended): Pure Rust implementation with full WASM support
- `tree-sitter-c2rust`: Legacy C2Rust transpiled backend
- `tree-sitter-standard`: Standard Tree-sitter C runtime

The first step is to configure your `build.rs` to compile and link the generated Tree Sitter parser:

```rust
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src");
    rust_sitter_tool::build_parsers(&PathBuf::from("src/main.rs"));
}
```



## Debugging

To view the generated grammar, you can set the `RUST_SITTER_EMIT_ARTIFACTS` environment variable to `true`. This will cause the generated grammar to be written to wherever cargo sets `OUT_DIR` (usually `target/debug/build/<crate>-<hash>/out`).


### ⚠️ Known Limitations

**Empty Production Rules**: Tree-sitter does not support grammar rules that can match zero tokens. This means structs with only `Vec<T>` fields need special handling. See [Empty Production Rules Guide](./docs/empty-production-rules.md) for solutions and patterns.

### 🚀 Getting Started

```bash
# Install the CLI tool
cargo install rust-sitter-cli

# Create a new grammar project
rust-sitter new my-language

# Test your grammar interactively
rust-sitter playground

# Generate an LSP server
rust-sitter generate-lsp
```

For detailed guides, see our comprehensive documentation above.
