# Adze

[![CI](https://github.com/EffortlessMetrics/adze/actions/workflows/ci.yml/badge.svg)](https://github.com/EffortlessMetrics/adze/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/adze)](https://crates.io/crates/adze)
[![docs.rs](https://img.shields.io/docsrs/adze)](https://docs.rs/adze)
[![MSRV](https://img.shields.io/badge/MSRV-1.92-blue)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-MIT)

**Define your grammar as Rust types. Get a typed AST back.**

Adze (formerly `rust-sitter`) is an AST-first grammar toolchain for Rust.
Describe your language with enums and structs, and the build tooling generates
an optimized GLR parser that returns your own types — no manual tree-walking
required.

```rust
#[adze::grammar("arithmetic")]
pub mod grammar {
    #[adze::language]
    #[derive(Debug, PartialEq)]
    pub enum Expr {
        Number(#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),

        #[adze::prec_left(1)]
        Sub(Box<Expr>, #[adze::leaf(text = "-")] (), Box<Expr>),

        #[adze::prec_left(2)]
        Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _ws: (),
    }
}

fn main() {
    let ast = grammar::parse("1 - 2 * 3").unwrap();
    // ast = Sub(Number(1), (), Mul(Number(2), (), Number(3)))
    println!("{ast:?}");
}
```

## Quick Start

Add the dependencies to your `Cargo.toml`:

```toml
[dependencies]
adze = "0.8"

[build-dependencies]
adze-tool = "0.8"
```

Create a `build.rs` in your project root:

```rust
use std::path::PathBuf;

fn main() {
    // Point this at the file containing your `#[adze::grammar(...)]` module.
    // Use `src/main.rs` for binary crates, or `src/lib.rs` for library crates.
    adze_tool::build_parsers(&PathBuf::from("src/main.rs"));
}
```

Define your grammar with `#[adze::grammar]` attributes in your Rust source,
then call `grammar::parse(input)` to get a `Result<YourType, Vec<ParseError>>`.

See the [Getting Started tutorial](./docs/tutorials/getting-started.md) for a
complete walkthrough.

## Features

> Support/proof source of truth: [`SUPPORT_TIERS.md`](./SUPPORT_TIERS.md)

| Feature | Status | Description |
|---------|--------|-------------|
| **Typed extraction** | ✅ Stable | Grammar *is* your AST — parse directly into your Rust types |
| **Pure Rust** | ✅ Stable | Default backend is 100% Rust; no C toolchain needed |
| **GLR parsing** | ✅ Stable | Handles ambiguous grammars (C++, JavaScript, etc.) |
| **Operator precedence** | ✅ Stable | `#[prec_left]`, `#[prec_right]` for disambiguation |
| **Serialization** | 🧪 Experimental | Serialization proof is currently partial in required CI |
| **WASM support** | 🧪 Experimental | Compile support exists; proof runs in optional CI lanes |
| **Tree-sitter interop** | ℹ️ Advisory | `ts-bridge` is available but not in required merge gate |
| **External scanners** | 🧪 Experimental | Custom tokenization via `ExternalScanner` trait |
| **Incremental parsing** | ℹ️ Advisory | Incremental path may fall back to fresh parsing |

### Cargo Features

```toml
# Default: pure-Rust backend
adze = "0.8"

# Enable GLR parser for ambiguous grammars
adze = { version = "0.8", features = ["glr"] }

# Enable WASM support
adze = { version = "0.8", features = ["wasm"] }

# Use the standard C Tree-sitter runtime instead
adze = { version = "0.8", default-features = false, features = ["tree-sitter-standard"] }
```

## Why Adze?

- **Type safety** — Your grammar *is* your AST. No manual mapping from generic
  tree nodes to domain types. Parse errors are caught at the type level.
- **Pure Rust** — The default runtime needs no C compiler, making
  cross-compilation and WASM targets straightforward.
- **GLR power** — Handles inherently ambiguous grammars that standard LR(1)
  parsers cannot, with automatic fork/merge at conflict points.
- **Interoperable** — Import existing Tree-sitter grammars and export
  Tree-sitter-compatible parse tables.

## How It Works

```
  ┌─────────────┐    build.rs     ┌──────────────┐    compile    ┌─────────────┐
  │  Rust types  │ ──────────────▶ │  Parse tables │ ──────────▶  │  Runtime    │
  │  + #[adze]   │   adze-tool    │  (generated)  │              │  parser     │
  └─────────────┘                 └──────────────┘              └──────┬──────┘
                                                                       │
                                                            text ──▶ parse()
                                                                       │
                                                                ▼
                                                        Result<YourType, Vec<ParseError>>
```

1. **Define** — Annotate Rust enums/structs with `#[adze::grammar]`,
   `#[adze::language]`, `#[adze::leaf]`, and precedence attributes.
2. **Generate** — `adze-tool` in `build.rs` extracts your grammar, builds an
   IR, computes LR(1)/GLR parse tables, and emits optimized Rust code.
3. **Parse** — At runtime, call `grammar::parse(input)` to get back your typed
   AST or a list of parse errors.

### Workspace Crates

| Crate | Role |
|-------|------|
| [`adze`](./runtime/) | Runtime library — parsing, extraction, error handling |
| [`adze-macro`](./macro/) | Proc-macro attributes (`#[adze::grammar]`, etc.) |
| [`adze-tool`](./tool/) | Build-time code generation (called from `build.rs`) |
| [`adze-ir`](./ir/) | Grammar intermediate representation |
| [`adze-glr-core`](./glr-core/) | GLR table generation — FIRST/FOLLOW, LR(1) items, conflicts |
| [`adze-tablegen`](./tablegen/) | Parse table compression and FFI-compatible output |

## Documentation

- [**Getting Started**](./docs/tutorials/getting-started.md) — Build your first parser in 5 minutes
- [**Architecture**](./docs/explanations/architecture.md) — How the macro, tool, and runtime fit together
- [**Grammar Examples**](./docs/reference/grammar-examples.md) — Patterns for common language constructs
- [**Quick Reference**](./QUICK_REFERENCE.md) — Attribute cheat sheet
- [**API Reference**](https://docs.rs/adze) — Generated API docs on docs.rs

## Contributing

Contributions are welcome! Please see [`CONTRIBUTING.md`](./CONTRIBUTING.md) for
guidelines and [`ROADMAP.md`](./ROADMAP.md) for planned work.

For internal development setup, see the
[Developer Guide](./docs/DEVELOPER_GUIDE.md).

## License

Licensed under either of

- [Apache License, Version 2.0](./LICENSE-APACHE)
- [MIT License](./LICENSE-MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
