# adze-tablegen

Parse table generation and compression for the [Adze](https://github.com/EffortlessMetrics/adze) parser toolchain.

## Overview

`adze-tablegen` transforms GLR parse tables into compressed, Tree-sitter-compatible binary formats. It implements the compression algorithms needed to produce compact static `Language` objects with FFI compatibility.

## Key Components

- **Table Compression** — Implements Tree-sitter's compression algorithms for >10x size reduction
- **ABI Builder** — Generates FFI-compatible Language structs matching Tree-sitter ABI
- **Language Generation** — Produces static language objects from parse tables
- **Node Types** — Generates NODE_TYPES JSON metadata
- **ParseTable Writer** — Serializes parse tables to `.parsetable` binary files
- **Validation** — Ensures compressed tables maintain correctness invariants

## Features

| Feature | Description |
|---------|-------------|
| `serialization` | Enable `.parsetable` file generation |
| `compression` | Enable compression/decompression utilities |
| `small-table` | Enable compressed table generation for smaller grammars |
| `tree-sitter-c2rust` | Use pure-Rust Tree-sitter backend (default) |
| `tree-sitter-standard` | Use standard C Tree-sitter backend |
| `strict_docs` | Enforce documentation requirements |

## Usage

This crate is primarily used internally by `adze-tool` during build-time code generation.

```rust
use adze_tablegen::compress::compress_parse_table;

let compressed = compress_parse_table(&parse_table, &grammar)?;
```

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT License](../LICENSE-MIT) at your option.
