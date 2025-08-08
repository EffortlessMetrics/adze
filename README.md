# Rust Sitter: Next-Generation Parser Development in Rust

[![Crates.io](https://img.shields.io/crates/v/rust-sitter)](https://crates.io/crates/rust-sitter)
[![Docs.rs](https://docs.rs/rust-sitter/badge.svg)](https://docs.rs/rust-sitter)
[![CI](https://github.com/hydro-project/rust-sitter/actions/workflows/ci.yml/badge.svg)](https://github.com/hydro-project/rust-sitter/actions/workflows/ci.yml)

**Rust Sitter is a modern parser-generator for Rust that brings the power of Tree-sitter's GLR parsing to the Rust ecosystem, with the convenience of procedural macros.**

Define your grammar directly in your Rust code with powerful, type-safe macros, and let Rust Sitter handle the complexity of generating a high-performance, error-recovering parser. It's designed for a new generation of developer tools, static analysis, and language servers.

> **Project Status (v0.5.0-beta):** Rust Sitter is currently in beta. It's a powerful tool for building parsers, but the API is still evolving. We welcome feedback and contributions!

## Key Features

*   **Grammar in Rust**: Define your grammar using intuitive macros directly on your Rust structs and enums. No separate grammar file needed.
*   **GLR Parsing**: Handle ambiguous grammars with ease. Rust Sitter's GLR parser can explore multiple parse trees and resolve conflicts, making it perfect for complex languages.
*   **Pure-Rust & WASM**: Generate parsers with zero C dependencies. Compile your parser to WebAssembly for use in the browser or other WASM runtimes.
*   **Advanced Error Recovery**: Build robust parsers that can gracefully handle syntax errors, providing meaningful feedback to users.
*   **Performance**: Leverage SIMD-accelerated lexing and parallel parsing for high-throughput applications.
*   **Incremental Parsing**: Reparse only the changed parts of a document, essential for responsive IDEs and language servers.

## Getting Started

### 1. Installation

Add `rust-sitter` to your project's `Cargo.toml`:

```toml
[dependencies]
rust-sitter = "0.5.0-beta"

[build-dependencies]
rust-sitter-tool = "0.5.0-beta"
```

### 2. Build Script

Set up a `build.rs` file to generate your parser at compile time:

```rust
// build.rs
use std::path::PathBuf;

fn main() {
    rust_sitter_tool::build_parsers(&PathBuf::from("src/lib.rs"));
}
```

### 3. Define a Grammar

Create a simple grammar for an arithmetic language in `src/lib.rs`:

```rust
// src/lib.rs
#[rust_sitter::grammar("arithmetic")]
pub mod grammar {
    #[rust_sitter::language]
    #[derive(Debug)]
    pub enum Expression {
        Number(
            #[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32,
        ),
        #[rust_sitter::prec_left(1)]
        Add {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "+")]
            _plus: (),
            right: Box<Expression>,
        },
        #[rust_sitter::prec_left(1)]
        Sub {
            left: Box<Expression>,
            #[rust_sitter::leaf(text = "-")]
            _minus: (),
            right: Box<Expression>,
        },
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}
```

### 4. Parse Some Code

Use your generated parser to parse a string:

```rust
// In your main.rs or a test
fn main() {
    let source_code = "1 + 2 - 3";
    let ast = grammar::parse(source_code).unwrap();
    println!("{:#?}", ast);
}
```

## How it Works

Rust Sitter uses procedural macros to extract a grammar definition from your Rust types. At compile time, the `rust-sitter-tool` build dependency generates a high-performance GLR parse table from your grammar. The `rust-sitter` runtime library then uses this table to parse your input text, constructing a type-safe Abstract Syntax Tree (AST).

## Documentation

For more detailed information, please refer to our documentation. A good place to start is the [documentation index](./docs/README.md).

## Contributing

Rust Sitter is an open-source project and we welcome contributions! Whether you're interested in improving the parser, adding new features, or enhancing the documentation, we'd love to have your help. Please see our `CONTRIBUTING.md` for more details.

## License

This project is licensed under the MIT License.
