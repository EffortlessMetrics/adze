# Adze

[![CI](https://github.com/EffortlessMetrics/adze/actions/workflows/ci.yml/badge.svg)](https://github.com/EffortlessMetrics/adze/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/adze)](https://crates.io/crates/adze)
[![MSRV](https://img.shields.io/badge/MSRV-1.92-blue)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-MIT)

**Rust-native grammar toolchain with GLR-capable parsing and typed extraction.**
Tree-sitter interoperable.

---

## Mental Model

Adze (formerly `rust-sitter`) is a compiler pipeline:

- **Define**: Describe your language using Rust enums/structs + attributes.
- **Compile**: Build tooling turns your types into an optimized LR(1) or GLR parse table in `build.rs`.
- **Parse**: The zero-dependency runtime uses these tables to build a parse forest.
- **Extract**: You receive **typed Rust values** (your own structs), not a generic "node" API.

---

## Minimal Example

```rust
#[adze::grammar("calc")]
mod grammar {
    #[adze::language]
    pub enum Expr {
        // Field type String automatically extracts the token text
        Number(#[adze::leaf(pattern = r"\d+")] String),

        #[adze::prec_left(1)]
        Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
    }
}

fn main() {
    // Returns Result<Expr, Vec<ParseError>>
    let ast = grammar::parse("2+3").unwrap();
    println!("{ast:?}");
}
```

---

## Installation

### Add to `Cargo.toml`

```toml
[dependencies]
adze = "0.8.0-dev"

[build-dependencies]
adze-tool = "0.8.0-dev"
```

### Create `build.rs`

```rust
use std::path::PathBuf;

fn main() {
    // This generates the parser source code at build time
    adze_tool::build_parsers(&PathBuf::from("src/main.rs"));
}
```

---

## Why Adze?

1. **Type Safety**: Your grammar *is* your AST. No more manual mapping from generic trees to domain objects.
2. **Pure Rust**: The default runtime is 100% Rust. No C toolchain required for WASM or cross-compilation.
3. **GLR Power**: Can handle inherently ambiguous grammars (like C++ or JavaScript) that standard LR(1) parsers struggle with.
4. **Interoperable**: Can import existing Tree-sitter grammars and export Tree-sitter compatible tables.

---

## Documentation

* [**Getting Started**](./docs/tutorials/getting-started.md) - Build your first parser in 5 minutes.
* [**Architecture**](./docs/explanations/architecture.md) - How the macro, tool, and runtime fit together.
* [**Grammar Examples**](./docs/reference/grammar-examples.md) - Patterns for common language constructs.
* [**Developer Guide**](./docs/DEVELOPER_GUIDE.md) - For contributors to the Adze project.

---

## Status and Planning

* **Roadmap:** [`ROADMAP.md`](./ROADMAP.md)
* **Execution Plan:** [`docs/status/NOW_NEXT_LATER.md`](./docs/status/NOW_NEXT_LATER.md)
* **Friction Log:** [`docs/status/FRICTION_LOG.md`](./docs/status/FRICTION_LOG.md)

---

## License

Dual-licensed under MIT OR Apache 2.0.
