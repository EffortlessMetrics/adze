# adze-tool

Build-time code generation tool for the [Adze](https://github.com/EffortlessMetrics/adze) parser toolchain.

## Overview

`adze-tool` handles all build-time code generation for Adze grammars. It reads annotated Rust source files, extracts grammar definitions, and generates parser code (Tree-sitter JSON grammars, C parsers, or pure-Rust parse tables).

## Key Components

- **Grammar Extraction** — Parses Rust source files to find `#[adze::grammar]` annotations
- **Grammar.js Generation** — Converts grammar IR to Tree-sitter grammar.js format
- **Pure-Rust Builder** — Generates Rust-native parse tables without Tree-sitter C dependency
- **Scanner Build** — Compiles external scanner definitions
- **CLI** — `adze-gen` binary for standalone grammar generation
- **Visualization** — Grammar and parse tree visualization tools

## Binaries

- `adze-gen` — Standalone grammar generation tool

## Usage

Typically called from `build.rs`:

```rust
// build.rs
fn main() {
    adze_tool::build_parsers();
}
```

Or as a CLI tool:

```bash
adze-gen path/to/grammar.rs
```

## Features

| Feature | Description |
|---------|-------------|
| `build_parsers` | Enable Tree-sitter C parser generation (default) |
| `serialization` | Enable parse table serialization (default) |
| `optimize` | Enable grammar optimization |
| `strict_docs` | Enforce documentation requirements |

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT License](../LICENSE-MIT) at your option.
