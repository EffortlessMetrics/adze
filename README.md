# Adze

[![CI](https://github.com/EffortlessMetrics/adze/actions/workflows/ci.yml/badge.svg)](https://github.com/EffortlessMetrics/adze/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/adze)](https://crates.io/crates/adze)

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

## Status (v0.6.x -- February 2026)

**Stable -- Macro path**:
- Macro-based grammar definition and code generation: working
- Type-safe AST extraction: working
- Precedence, associativity, repetition, optionals: working
- Pure Rust with zero C dependencies

**Stable -- GLR core**:
- GLR table generation: algorithmically correct, tested in isolation
- ActionCell architecture supporting shift/reduce and reduce/reduce conflicts
- Production grammars (Python with 273 symbols, external scanners) compile successfully

**Experimental -- GLR runtime wiring**:
- GLR tables are generated correctly, but the default runtime for macro grammars uses simple LR
- Full GLR runtime (`parser_v4`) exists and passes its own test suite, but is not yet the default
- Incremental parsing infrastructure exists but is disabled (falls back to fresh parsing)

For production use today: the macro path with the default backend is stable. The pure-Rust GLR runtime path is experimental.

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

```toml
[dependencies]
adze = "0.6"

[build-dependencies]
adze-tool = "0.6"
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

Working grammars are in [example/src/](./example/src/):
- Arithmetic expressions with precedence
- Repetition and delimiter patterns
- Optional fields
- Word boundaries

Run the example tests:
```bash
cargo test -p adze-example
```

---

## Contributing

Contributions are welcome. See **[CONTRIBUTING.md](./CONTRIBUTING.md)** for development setup, coding standards, and PR guidelines.

Before submitting a PR:
1. Run tests: `cargo test`
2. Run linter: `cargo clippy --all -- -D warnings`
3. Run formatter: `cargo fmt -- --check`

Questions? Open a [GitHub Issue](https://github.com/EffortlessMetrics/adze/issues).

---

## Comparison with Tree-sitter

**Similarities**:
- LR-family parsing (GLR in adze, LR(1) in tree-sitter)
- Error recovery
- External scanner support
- Incremental parsing (tree-sitter mature; adze experimental)

**Differences**:
- **Grammar source**: Rust types (adze) vs JavaScript DSL (tree-sitter)
- **Parse output**: Typed Rust AST (adze) vs generic syntax tree (tree-sitter)
- **Build dependencies**: Pure Rust (adze) vs C compiler + Node.js (tree-sitter)
- **Ambiguity**: GLR with multiple parse paths (adze) vs single-parse LR (tree-sitter)

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
