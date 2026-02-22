# Adze

[![CI](https://github.com/EffortlessMetrics/adze/actions/workflows/ci.yml/badge.svg)](https://github.com/EffortlessMetrics/adze/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/adze)](https://crates.io/crates/adze)
[![MSRV](https://img.shields.io/badge/MSRV-1.92-blue)](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-MIT)

**Rust-native grammar toolchain with GLR-capable parsing and typed extraction.**
Tree-sitter interoperable.

---

## Mental model

Adze is a compiler pipeline:

- **Define**: Rust enums/structs + attributes describe structure, tokens, precedence.
- **Compile**: build tooling turns that into IR + parse tables in `build.rs`.
- **Parse**: runtime uses tables (LR/GLR paths) to build a tree/forest.
- **Extract**: you receive typed Rust values (your enums/structs), not a generic node API.

---

## Minimal example

```rust
#[adze::grammar("calc")]
mod grammar {
    #[adze::language]
    pub enum Expr {
        Number(#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),

        #[adze::prec_left(1)]
        Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
    }
}

fn main() {
    let ast = grammar::parse("2+3").unwrap();
    println!("{ast:?}");
}
```

Working end-to-end examples live in:

* `example/`

> Some docs outside `docs/status/` are being refreshed. When in doubt, treat the code as truth.

---

## Install

### Published (stable)

```toml
[dependencies]
adze = "0.8.0-dev"

[build-dependencies]
adze-tool = "0.8.0-dev"
```

### Dev head (unreleased)

```toml
[dependencies]
adze = { git = "https://github.com/EffortlessMetrics/adze" }

[build-dependencies]
adze-tool = { git = "https://github.com/EffortlessMetrics/adze" }
```

### `build.rs`

```rust
fn main() {
    adze_tool::build_parsers(&std::path::PathBuf::from("src/main.rs"));
}
```

---

## Repo map

**Core pipeline**

* `runtime/` — public crate (`adze`)
* `macro/` — proc-macros (`adze-macro`)
* `tool/` — build-time compiler (`adze-tool`)
* `common/`, `ir/`, `tablegen/`, `glr-core/` — IR + table generation + GLR machinery

**Validation & tooling**

* `golden-tests/` — parity validation (selected grammars)
* `grammars/` — grammar crates (Python/JS/Go, etc.)
* `cli/`, `lsp-generator/`, `playground/`, `wasm-demo/` — tools/prototypes

### What's stable vs experimental

- **Stable:** macro grammars, build-time table generation, typed extraction.
- **Experimental:** GLR runtime (`features = ["glr"]`), incremental parsing.
- **Prototypes:** CLI, LSP generator, playground, wasm-demo (useful, not merge-gated).

---

## Status and planning

These files are the maintained "current truth":

* **Roadmap (durable outcomes):** [`ROADMAP.md`](./ROADMAP.md)
* **Now / Next / Later (rolling plan):** [`docs/status/NOW_NEXT_LATER.md`](./docs/status/NOW_NEXT_LATER.md)
* **Friction log (paper cuts we burn down):** [`docs/status/FRICTION_LOG.md`](./docs/status/FRICTION_LOG.md)
* **Known red (what's excluded from the supported lane):** [`docs/status/KNOWN_RED.md`](./docs/status/KNOWN_RED.md)

---

## Contributing (short version)

* Run the supported gate: `just ci-supported`
* Use the hooks: `just pre` (or `.githooks/pre-commit`)
* If something is painful twice, add it to the **Friction Log** with a link to an issue.

See [`CONTRIBUTING.md`](./CONTRIBUTING.md).

---

## License

Dual-licensed under MIT OR Apache 2.0.
