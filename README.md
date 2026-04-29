# Adze

**Typed parser generation for Rust.**

Adze lets you describe a grammar with Rust enums and structs, generate a parser at build time, and parse text back into your own Rust types.

It is for projects where a generic parse tree is not the final product. The goal is:

```text
grammar as Rust types
→ generated parser
→ typed Rust AST
```

not:

```text
grammar
→ generic tree
→ hand-written tree walker
→ hand-written AST mapper
```

Adze is under active development. The core parser pipeline is working and tested. Some broader surfaces — WASM demos, grammar crates, golden tests, benchmarks, Tree-sitter bridge workflows, and alternative runtimes — are still being hardened.

## What problem does Adze solve?

Parser generators usually make you maintain two models:

1. the grammar
2. the AST you actually want to use

Adze tries to collapse those into one model.

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
        Add(
            Box<Expr>,
            #[adze::leaf(text = "+")] (),
            Box<Expr>,
        ),

        #[adze::prec_left(2)]
        Mul(
            Box<Expr>,
            #[adze::leaf(text = "*")] (),
            Box<Expr>,
        ),
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s+")]
        _ws: (),
    }
}
```

Then parse into those types:

```rust
let expr = grammar::parse("1 + 2 * 3")?;
```

The intended result is not a generic node tree. It is your Rust AST.

## Current status

Adze’s core pipeline is the supported path:

```text
adze-macro
→ adze-tool
→ adze-ir
→ adze-glr-core
→ adze-tablegen
→ adze runtime
→ typed extraction
```

The supported lane covers the core crates and keeps the required gate bounded enough to be useful day to day. Broader surfaces are valuable, but not all are part of the supported contract yet.

| Surface                     | Status                      | Notes                                                                                            |
| --------------------------- | --------------------------- | ------------------------------------------------------------------------------------------------ |
| Pure-Rust parser generation | Supported / hardening       | Default path for generated parsers.                                                              |
| Typed AST extraction        | Supported for proven shapes | Exact-value tests define the contract.                                                           |
| Operator precedence         | Supported / hardening       | Used for expression grammars.                                                                    |
| GLR routing                 | Stabilizing                 | True GLR conflict routing is in place; broader ambiguity behavior is still being expanded.      |
| Structured parse errors     | Stabilizing                 | Covered by focused runtime tests.                                                                |
| External scanners           | Experimental                | Useful, but not yet a broad support claim.                                                       |
| Incremental parsing         | Experimental                | Exists, but should be treated as a developing surface.                                           |
| Tree-sitter bridge          | Advisory                    | Useful interop path; workflow and parity coverage are being hardened.                            |
| WASM                        | Advisory                    | Compile/proof surface is being expanded; runtime/browser execution is not yet the main contract. |
| Grammar crates              | Advisory                    | Valuable smoke coverage; not all grammar crates are production-ready.                            |
| Benchmarks                  | Advisory                    | Benchmarks are signal, not support proof.                                                        |

## Install

```toml
[dependencies]
adze = "0.8"

[build-dependencies]
adze-tool = "0.8"
```

Add a `build.rs`:

```rust
use std::path::PathBuf;

fn main() {
    adze_tool::build_parsers(&PathBuf::from("src/lib.rs"));
}
```

Define your grammar in Rust source, then call the generated parser:

```rust
let value = grammar::parse(input)?;
```

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

The important boundary is typed extraction. Adze is not finished when it can parse text into nodes. It is finished when it can return the expected Rust value for the grammar shape being claimed.

## What Adze is good for today

Adze is a good fit when:

* you are building a parser or DSL in Rust
* you control the grammar
* you want the AST to be regular Rust types
* you want parser generation integrated into `build.rs`
* you want a pure-Rust path without relying on a C toolchain
* you are comfortable with a project that is still hardening its broader ecosystem surface

## What Adze is not yet

Adze is not yet a fully polished replacement for every mature parser generator.

In particular, treat these as developing surfaces:

* large real-world grammar crates
* full Tree-sitter grammar import parity
* browser/runtime WASM execution proof
* broad incremental parsing guarantees
* external scanner contracts
* benchmark claims beyond the documented benchmark scope
* runtime2 and governance microcrates

Those may work in places. They are not all part of the same support tier.

## Repository layout

| Crate           | Role                                                  |
| --------------- | ----------------------------------------------------- |
| `adze`          | Runtime parser, parse trees, typed extraction, errors |
| `adze-macro`    | Grammar attributes and extraction support             |
| `adze-tool`     | Build-time parser generation                          |
| `adze-ir`       | Grammar intermediate representation                   |
| `adze-glr-core` | LR/GLR automata, conflicts, ambiguity machinery       |
| `adze-tablegen` | Parse table generation and compression                |

Additional workspace areas include grammar crates, benchmarks, WASM demos, golden tests, bridge tooling, and governance/test-support crates. These are useful, but not all are part of the supported core lane.

## Running the core checks

The supported core check is:

```bash
just ci-supported
```

Useful focused checks:

```bash
cargo test -p adze --features glr --test test_parser_routing
cargo test -p adze --features glr --test test_e2e_ambiguous_grammar_glr
cargo test -p adze --test typed_ast_contract
cargo test -p adze --test extract_trait_v9
cargo test -p adze-tool --all-features --test build_pipeline
cargo fmt --all --check
```

## Design principles

Adze is built around a few constraints:

* The grammar should be readable as Rust.
* The AST should be the user’s Rust type, not a second mapping layer.
* Ambiguity should be handled explicitly, not hidden by silent fallback.
* Claims should be tied to proof commands.
* Unsupported or advisory surfaces should be labeled as such.

## Roadmap

Near-term work:

* expand the typed-extraction shape matrix
* keep GLR conflict routing honest and tested
* improve diagnostics around grammar/build/runtime failures
* add advisory product-proof CI for broader surfaces
* align README/support claims with proof
* clean up duplicate PR families and benchmark/dependency hygiene

Longer-term work:

* stronger grammar crate canaries
* stricter golden output tests
* browser/runtime WASM proof
* deeper Tree-sitter bridge parity
* clearer benchmark truth and performance baselines

## License

Licensed under either:

* Apache-2.0
* MIT

at your option.
