# Adze

[![CI](https://github.com/EffortlessMetrics/adze/actions/workflows/ci.yml/badge.svg)](https://github.com/EffortlessMetrics/adze/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/adze)](https://crates.io/crates/adze)
[![docs.rs](https://img.shields.io/docsrs/adze)](https://docs.rs/adze)
[![MSRV](https://img.shields.io/badge/MSRV-1.92-blue)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-MIT)

**Typed parser generation for Rust.** Formerly `rust-sitter`.

Describe a grammar with Rust enums and structs. Get a typed AST back. No tree-walking required.

```toml
[dependencies]
adze = "0.8"

[build-dependencies]
adze-tool = "0.8"
```

See the [Getting Started tutorial](./docs/tutorials/getting-started.md) for a full walkthrough.

---

## What problem does Adze solve?

Parser generators usually make you maintain two models: the grammar and the AST you actually want to use. Adze collapses those into one.

The goal is:

```text
grammar as Rust types  →  generated parser  →  typed Rust AST
```

not:

```text
grammar  →  generic tree  →  hand-written tree walker  →  hand-written AST mapper
```

You write Rust types:

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
```

Then parse directly into those types:

```rust
let expr = grammar::parse("1 + 2 * 3")?;
// expr = Add(Number(1), (), Mul(Number(2), (), Number(3)))
```

## Quick start

**1. Add dependencies** (shown above).

**2. Add a `build.rs`** pointing at the file that contains your `#[adze::grammar]` module:

```rust
use std::path::PathBuf;

fn main() {
    // src/lib.rs for library crates, src/main.rs for binary crates
    adze_tool::build_parsers(&PathBuf::from("src/lib.rs"));
}
```

**3. Define your grammar** with `#[adze::grammar]`, `#[adze::language]`, `#[adze::leaf]`, and precedence attributes in your Rust source.

**4. Parse:** call the generated `grammar::parse(input)` to get `Result<YourType, Vec<ParseError>>`.

## Current status

Adze is under active development. The core parser pipeline is working and tested. Some broader surfaces — WASM, grammar crates, golden tests, benchmarks, Tree-sitter bridge — are still being hardened.

The supported lane runs through these crates:

```text
adze-macro  →  adze-tool  →  adze-ir  →  adze-glr-core  →  adze-tablegen  →  adze runtime  →  typed extraction
```

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

## What Adze is good for today

- Building a parser or DSL in Rust where you control the grammar
- Projects that want the AST as plain Rust types, not a second mapping layer
- Pure-Rust builds with no C toolchain dependency
- Parser generation integrated into `build.rs` as part of a normal Cargo workflow

## What Adze is not yet

- A drop-in replacement for mature, broadly-tested parser generators
- A guaranteed-stable surface for large real-world grammars
- Full Tree-sitter grammar import parity
- Proven browser/runtime WASM execution
- Stable incremental parsing or external scanner contracts
- Alternative runtime implementations or internal policy infrastructure

Those areas exist in the codebase and may work in practice. They are not part of the same support tier as the core pipeline.

## How it works

```text
Rust types + #[adze]
        │
        ▼
adze-macro extracts grammar shape
        │
        ▼
adze-tool builds grammar IR
        │
        ▼
adze-glr-core builds parse automata
        │
        ▼
adze-tablegen emits parse tables / generated Rust
        │
        ▼
adze runtime parses input
        │
        ▼
typed extraction returns your AST
```

The key boundary is typed extraction. Adze is not done when it can parse text into nodes; it is done when it returns the expected Rust value for the grammar shape being claimed.

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

## Running the core checks

```bash
just ci-supported      # required PR gate — fmt + clippy + tests on core crates
just test              # core lib tests
just clippy            # lint core crates
cargo fmt --all --check
cargo t2               # tests with 2 threads
```

## Design principles

- The grammar should be readable as Rust.
- The AST should be the user's Rust type, not a second mapping layer.
- Ambiguity should be handled explicitly, not hidden by silent fallback.
- Claims should be tied to proof commands.
- Unsupported or advisory surfaces should be labeled as such.

## Documentation

- [Getting Started](./docs/tutorials/getting-started.md) — build your first parser
- [Architecture](./docs/explanations/architecture.md) — how the macro, tool, and runtime fit together
- [Grammar Examples](./docs/reference/grammar-examples.md) — patterns for common constructs
- [API Reference](https://docs.rs/adze) — generated docs on docs.rs

## Roadmap

Near-term:

- expand the typed-extraction shape matrix
- keep GLR conflict routing honest and tested
- improve diagnostics around grammar/build/runtime failures
- add advisory CI coverage for broader surfaces

Longer-term:

- stronger grammar crate canaries
- stricter golden output tests
- browser/runtime WASM proof
- deeper Tree-sitter bridge parity
- clearer benchmark baselines

## Contributing

Contributions are welcome. See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for guidelines and [`ROADMAP.md`](./ROADMAP.md) for planned work. For internal development setup, see the [Developer Guide](./docs/DEVELOPER_GUIDE.md).

## License

Licensed under either:

- Apache-2.0
- MIT

at your option.
