# Adze API Documentation

> **Status**: Up to date for Adze 0.8.x.

This document describes the public API of the `adze` crate (formerly `rust-sitter`).

## Core Types

### `Parser`
The main entry point for parsing.

```rust
use adze::Parser;

let mut parser = Parser::new();
parser.set_language(my_grammar::language())?;
let tree = parser.parse("input", None).ok_or("parse failed")?;
// If the active parser backend cannot service this input yet (for example,
// parser_v4 Tree integration is still incomplete), parse() may return `None`.
```

### `Extract` Trait
The trait that powers the typed AST extraction.

```rust
pub trait Extract<T> {
    fn extract(node: Node, source: &[u8]) -> T;
}
```

Implementations are provided for:
- `String` (extracts text)
- `Vec<T>` (repeated elements)
- `Option<T>` (optional elements)
- `Box<T>` (recursive elements)
- Generated structs/enums (via `#[adze::grammar]`)

### `Tree`
Represents the parsed syntax tree.

```rust
impl Tree {
    pub fn root_node(&self) -> Node;
    pub fn edit(&mut self, edit: &InputEdit); // Incremental parsing
}
```

## Macro API

See [Getting Started](../tutorials/getting-started.md) for macro usage.

- `#[adze::grammar]`
- `#[adze::language]`
- `#[adze::leaf]`
- `#[adze::extra]`
- `#[adze::prec_left]`, `#[adze::prec_right]`

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `pure-rust` | Use the pure Rust runtime (no C deps) | Yes |
| `simd` | Enable SIMD lexing optimizations | No |
| `glr` | Enable GLR runtime for ambiguous grammars | No |
| `serialization` | Enable parse tree serialization | No |

## Experimental APIs

### `runtime2`
An experimental next-generation runtime located in the `runtime2/` crate. Not yet stable for public use.

### `ts-bridge`
Tool for generating Adze bindings from existing Tree-sitter grammars.

```rust
// In build.rs
ts_bridge::build_grammar("path/to/grammar");
```
