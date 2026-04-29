# Adze

[![CI](https://github.com/EffortlessMetrics/adze/actions/workflows/ci.yml/badge.svg)](https://github.com/EffortlessMetrics/adze/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/adze)](https://crates.io/crates/adze)
[![docs.rs](https://img.shields.io/docsrs/adze)](https://docs.rs/adze)
[![MSRV](https://img.shields.io/badge/MSRV-1.92-blue)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-MIT)

**Your grammar is your AST.** Formerly `rust-sitter`.

Most parser generators make you maintain two things: the grammar and the AST you actually want to use. Adze collapses those into one. You describe your language as Rust types, and the build tooling generates a parser that returns those types directly.

```text
grammar as Rust types  →  generated parser  →  typed Rust AST
```

not:

```text
grammar  →  generic tree  →  hand-written tree walker  →  hand-written AST mapper
```

```rust
#[adze::grammar("arithmetic")]
pub mod grammar {
    #[adze::language]
    #[derive(Debug, PartialEq)]
    pub enum Expr {
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            i32,
        ),

        #[adze::prec_left(1)]
        Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),

        #[adze::prec_left(2)]
        Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s+")]
        _ws: (),
    }
}

let expr = grammar::parse("1 + 2 * 3")?;
// expr = Add(Number(1), (), Mul(Number(2), (), Number(3)))
```

## Install

```toml
[dependencies]
adze = "0.8"

[build-dependencies]
adze-tool = "0.8"
```

Add a `build.rs` pointing at the file that contains your `#[adze::grammar]` module:

```rust
use std::path::PathBuf;

fn main() {
    // src/lib.rs for library crates, src/main.rs for binary crates
    adze_tool::build_parsers(&PathBuf::from("src/lib.rs"));
}
```

Then call `grammar::parse(input)` to get `Result<YourType, Vec<ParseError>>`.

See the [Getting Started tutorial](./docs/tutorials/getting-started.md) for a full walkthrough.

## Current status

Adze is under active development. The core parser pipeline is working and tested. Some broader surfaces — WASM, grammar crates, golden tests, benchmarks, Tree-sitter bridge — are still being hardened.

| Surface                     | Status                      | Notes                                                                                            |
| --------------------------- | --------------------------- | ------------------------------------------------------------------------------------------------ |
| Pure-Rust parser generation | Supported / hardening       | Default path for generated parsers.                                                              |
| Typed AST extraction        | Supported for proven shapes | Exact-value tests define the contract.                                                           |
| Operator precedence         | Supported / hardening       | Used for expression grammars.                                                                    |
| GLR routing                 | Stabilizing                 | True GLR conflict routing is in place; broader ambiguity behavior is still being expanded.       |
| Structured parse errors     | Stabilizing                 | Covered by focused runtime tests.                                                                |
| External scanners           | Experimental                | Useful, but not yet a broad support claim.                                                       |
| Incremental parsing         | Experimental                | Exists, but should be treated as a developing surface.                                           |
| Tree-sitter bridge          | Advisory                    | Useful interop path; workflow and parity coverage are being hardened.                            |
| WASM                        | Advisory                    | Compile/proof surface is being expanded; runtime/browser execution is not yet the main contract. |
| Grammar crates              | Advisory                    | Valuable smoke coverage; not all grammar crates are production-ready.                            |
| Benchmarks                  | Advisory                    | Benchmarks are signal, not support proof.                                                        |

Adze is a good fit if you are building a parser or DSL in Rust, you control the grammar, and you want the result as plain Rust types integrated into a normal Cargo build. It is not yet a drop-in replacement for mature parser generators, and surfaces outside the core lane should be treated as developing.

## How it works

At build time, `adze-tool` reads your annotated Rust types, constructs a grammar IR, computes LR(1)/GLR parse tables via `adze-glr-core`, and emits generated Rust through `adze-tablegen`. At runtime, `adze` uses those tables to parse input and return your typed value directly — no generic tree, no secondary mapping step.

```text
adze-macro  →  adze-tool  →  adze-ir  →  adze-glr-core  →  adze-tablegen  →  adze runtime  →  typed extraction
```

## Repository layout

| Crate           | Role                                                  |
| --------------- | ----------------------------------------------------- |
| `adze`          | Runtime parser, parse trees, typed extraction, errors |
| `adze-macro`    | Grammar attributes and extraction support             |
| `adze-tool`     | Build-time parser generation                          |
| `adze-ir`       | Grammar intermediate representation                   |
| `adze-glr-core` | LR/GLR automata, conflicts, ambiguity machinery       |
| `adze-tablegen` | Parse table generation and compression                |

Additional workspace areas include grammar crates, benchmarks, WASM demos, golden tests, bridge tooling, and test-support infrastructure. These are useful but not all are part of the supported core lane.

## Documentation

- [Getting Started](./docs/tutorials/getting-started.md) — build your first parser
- [Architecture](./docs/explanations/architecture.md) — how the macro, tool, and runtime fit together
- [Grammar Examples](./docs/reference/grammar-examples.md) — patterns for common constructs
- [API Reference](https://docs.rs/adze) — generated docs on docs.rs

## Contributing

Contributions are welcome. See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for guidelines, [`ROADMAP.md`](./ROADMAP.md) for planned work, and the [Developer Guide](./docs/DEVELOPER_GUIDE.md) for internal setup.

## Development

```bash
just ci-supported      # required PR gate — fmt + clippy + tests on core crates
just test              # core lib tests
just clippy            # lint core crates
cargo fmt --all --check
cargo t2               # tests with 2 threads
```

## License

Licensed under either:

- Apache-2.0
- MIT

at your option.
