# Adze

[![CI](https://github.com/EffortlessMetrics/adze/actions/workflows/ci.yml/badge.svg)](https://github.com/EffortlessMetrics/adze/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/adze)](https://crates.io/crates/adze)
[![MSRV](https://img.shields.io/badge/MSRV-1.92-blue)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-MIT)

**AST-first grammar toolchain for Rust.**

Define language structure using Rust types, compile to parse tables at build time, parse with GLR at runtime, extract typed Rust values. Tree-sitter interoperable, not affiliated.

```rust
#[adze::grammar("calc")]
mod grammar {
    #[adze::language]
    pub enum Expr {
        Number(#[leaf(pattern = r"\d+")] i32),
        #[prec_left(1)]
        Add(Box<Expr>, #[leaf(text = "+")] (), Box<Expr>),
    }
}

fn main() {
    let ast = grammar::parse("2 + 3").unwrap();  // Typed AST!
    // ast: Expr::Add(Expr::Number(2), (), Expr::Number(3))
}
```

---

## Why Adze?

| Feature | adze | tree-sitter | nom | pest |
|---------|------|-------------|-----|------|
| **Grammar in** | Rust types | JavaScript | Rust code | PEG file |
| **Output** | Typed AST | Generic tree | Combinator | Generic tree |
| **GLR** | Built-in | LR only | No | No |
| **Build-time** | Pure Rust | Needs Node.js | Pure Rust | Pure Rust |
| **Type safety** | Compile-time | Runtime | Manual | Runtime |

**Good for**: CLI tools, typed parsing, ambiguous grammars, pure-Rust pipelines.

---

## Quick Start

Get parsing in 5 minutes: **[QUICK_START.md](./QUICK_START.md)**

Or install now:

```bash
cargo add adze
cargo add --build adze-tool
```

---

## Status

**Published: 0.6.x (beta) · Dev head: 0.8.0-dev (unreleased) · MSRV: 1.92**

**Usable today — Macro path**:
- Macro-based grammar definition and code generation: working
- Type-safe AST extraction: working
- Precedence, associativity, repetition, optionals: working
- Pure Rust with zero C dependencies

**Usable today — GLR core**:
- GLR table generation: algorithmically correct, tested in isolation
- ActionCell architecture supporting shift/reduce and reduce/reduce conflicts
- Grammar crates (Python, JavaScript, Go) compile successfully

**GLR runtime** (`runtime2/`):
- Passes its own test suite; not yet the default backend for macro grammars

**Incremental parsing**:
- Infrastructure exists, currently falls back to fresh parsing for consistency

**Ecosystem tools** (early stage):
- CLI (`cli/`), LSP generator (`lsp-generator/`), golden tests (`golden-tests/`), playground (`playground/`) exist as prototypes

> This repo tracks dev head; published crate versions lag behind. When in doubt, trust the working examples in `example/`.

---

## How It Works

```
  Build-time                          Run-time
  =========                           ========

  Rust types                          Source text
  + #[adze::...] attributes           "2 + 3"
       |                                  |
       v                                  v
  Macro expansion                     GLR parser
  (validate + mark types)             (parse tables + token stream)
       |                                  |
       v                                  v
  build.rs                            Parse tree / forest
  (types -> IR -> grammar JSON        (concrete syntax tree)
   -> parse tables)                       |
       |                                  v
       v                              Typed extraction
  Compiled parser                     Expr::Add(Expr::Number(2),
  (static tables linked                       (), Expr::Number(3))
   into your binary)
```

Two-stage processing keeps macros lightweight (validation only) while `build.rs` does the heavy lifting of parser generation. At runtime, the compiled tables drive GLR parsing and typed AST extraction in a single pass.

See [ARCHITECTURE.md](./ARCHITECTURE.md) for full details.

---

## Features

**Grammar definition with Rust types**:
```rust
#[adze::grammar("mylang")]
mod grammar {
    #[adze::language]
    pub enum Expr {
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            i32
        ),

        #[adze::prec_left(1)]
        Add(Box<Expr>, #[leaf(text = "+")] (), Box<Expr>),

        #[adze::prec_left(2)]
        Mul(Box<Expr>, #[leaf(text = "*")] (), Box<Expr>),
    }

    #[adze::extra]
    struct Whitespace {
        #[leaf(pattern = r"\s")] _ws: (),
    }
}
```

**What you get**:
- Typed AST values, not generic trees
- Precedence handling: `2+3*4` parses as `2+(3*4)` automatically
- Error recovery for malformed input
- GLR support for ambiguous grammars
- Compile-time grammar validation

**Repetition (lists)**:
```rust
pub struct ArgList {
    #[adze::repeat]
    #[adze::delimited(#[leaf(text = ",")] ())]
    args: Vec<Expr>,  // Comma-separated list
}
```

**Optional elements**:
```rust
pub struct Function {
    name: String,
    params: Option<ParamList>,
}
```

**External scanners** (for context-sensitive lexing like Python indentation):
```rust
impl adze::ExternalScanner for IndentScanner {
    fn scan(&mut self, lexer: &mut Lexer, valid: &[bool]) -> ScanResult {
        // Custom lexing logic
    }
}
```

---

## Documentation

- **[Book](./book/)** -- mdBook-based guide and reference
- **[Quick Start](./QUICK_START.md)** -- Get parsing in 5 minutes
- **[Full Tutorial](./docs/GETTING_STARTED.md)** -- Complete guide
- **[API Reference](./API_DOCUMENTATION.md)** -- Complete API
- **[Architecture](./ARCHITECTURE.md)** -- How it all fits together
- **[FAQ](./FAQ.md)** -- Common questions
- **[Roadmap](./ROADMAP.md)** -- Future plans
- **[Changelog](./CHANGELOG.md)** -- Release history
- **[Contributing](./CONTRIBUTING.md)** -- How to help

---

## Installation

**Published (stable)**:
```toml
[dependencies]
adze = "0.6"

[build-dependencies]
adze-tool = "0.6"
```

**Dev head (unreleased — next publish target is 0.8.0)**:
```toml
[dependencies]
adze = { git = "https://github.com/EffortlessMetrics/adze" }

[build-dependencies]
adze-tool = { git = "https://github.com/EffortlessMetrics/adze" }
```

Create `build.rs`:
```rust
fn main() {
    adze_tool::build_parsers(&std::path::PathBuf::from("src/main.rs"));
}
```

**Backends**:
- `tree-sitter-c2rust` (default) -- Pure Rust, no C dependencies
- `tree-sitter-standard` -- Standard C Tree-sitter runtime

---

## Examples

**Macro grammars** in [example/src/](./example/src/):
- Arithmetic expressions with precedence
- Repetition and delimiter patterns
- Optional fields
- Word boundaries

**Grammar crates** in [grammars/](./grammars/):
- Python, JavaScript, Go — compile against the GLR core; not yet published

**Golden tests** in [golden-tests/](./golden-tests/):
- Validate adze-generated parsers against Tree-sitter reference implementations

**Downstream demo** in [samples/downstream-demo/](./samples/downstream-demo/):
- Consumer integration example for parser crates built from adze grammars

Run the example tests:
```bash
cargo test -p adze-example
```

---

## Workspace Layout

**Core** — the macro-based grammar pipeline:
- [`runtime/`](./runtime/) — main runtime library (the `adze` crate)
- [`macro/`](./macro/) — proc-macro definitions (`adze-macro`)
- [`tool/`](./tool/) — build-time code generation (`adze-tool`)
- [`common/`](./common/) — shared grammar expansion logic

**GLR Engine** — pure-Rust parser generation:
- [`ir/`](./ir/) — grammar intermediate representation
- [`glr-core/`](./glr-core/) — FIRST/FOLLOW, LR(1) item sets, conflict resolution
- [`tablegen/`](./tablegen/) — table compression and FFI-compatible Language generation
- [`runtime2/`](./runtime2/) — GLR runtime with Tree-sitter compatible API

**Grammars**: [`grammars/`](./grammars/) — Python, JavaScript, Go grammar crates

**Tools**:
- [`cli/`](./cli/) — command-line interface (early stage)
- [`lsp-generator/`](./lsp-generator/) — LSP server generator (prototype)
- [`playground/`](./playground/) — interactive grammar playground (prototype)
- [`wasm-demo/`](./wasm-demo/) — WebAssembly demo
- [`tools/ts-bridge/`](./tools/ts-bridge/) — Tree-sitter to GLR bridge

**Testing**:
- [`golden-tests/`](./golden-tests/) — validation against Tree-sitter reference
- [`benchmarks/`](./benchmarks/) — performance benchmarks
- [`example/`](./example/) — example grammars with snapshot tests
- [`runtime/fuzz/`](./runtime/fuzz/) — fuzz testing

**Internal**: [`crates/`](./crates/) — BDD contracts, governance — in development

---

## Contributing

Contributions are welcome. See **[CONTRIBUTING.md](./CONTRIBUTING.md)** for development setup, coding standards, and PR guidelines.

**Requirements**: Rust 1.92.0+ (MSRV), Rust 2024 edition.

Before submitting a PR:
1. Run tests: `cargo test` (or `./scripts/test-capped.sh` for stable concurrency)
2. Run linter: `cargo clippy --all -- -D warnings`
3. Run formatter: `cargo fmt -- --check`

Questions? Open a [GitHub Issue](https://github.com/EffortlessMetrics/adze/issues).

---

## Comparison with Tree-sitter

**Similarities**:
- LR-family parsing (GLR in adze, LR(1) in tree-sitter)
- Error recovery
- External scanner support
- Tree-sitter interoperability: validated via golden tests for selected grammars
- Incremental parsing (tree-sitter mature; adze experimental)

**Differences**:
- **Grammar source**: Rust types (adze) vs JavaScript DSL (tree-sitter)
- **Parse output**: Typed Rust AST (adze) vs generic syntax tree (tree-sitter)
- **Build dependencies**: Pure Rust (adze) vs C compiler + Node.js (tree-sitter)
- **Ambiguity**: GLR runtime exists and passes its test suite (adze) vs single-parse LR (tree-sitter)

**Choose adze when you**:
- Want type-safe parsing in pure Rust
- Prefer defining grammars as Rust types
- Need GLR support for ambiguous grammars

**Choose tree-sitter when you**:
- Need mature editor integration today
- Want battle-tested incremental parsing
- Have existing tree-sitter grammars to reuse

See [FAQ.md](./FAQ.md) for comparisons with nom, pest, and lalrpop.

---

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

---

## Acknowledgments

Adze builds on ideas from:
- [Tree-sitter](https://tree-sitter.github.io/) -- Inspiration for table format and incremental parsing design
- [GLR parsing](https://en.wikipedia.org/wiki/GLR_parser) -- Ambiguity handling via parallel parse stacks
- [LALR](https://en.wikipedia.org/wiki/LALR_parser) -- Parser algorithm foundations
- [Rust Sitter](https://github.com/hydro-project/rust-sitter)

Thanks to all contributors.
