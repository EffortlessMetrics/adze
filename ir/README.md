# adze-ir

Grammar Intermediate Representation for the [Adze](https://github.com/EffortlessMetrics/adze) parser toolchain.

## Overview

`adze-ir` defines the intermediate representation (IR) used to represent grammars throughout the Adze pipeline. It bridges the gap between user-defined Rust grammar annotations and the low-level parse table generation.

## Key Components

- **Grammar IR** — Core data structures for representing grammar rules, symbols, and productions
- **Symbol Registry** — Manages symbol IDs and metadata across the pipeline
- **Optimizer** — Grammar optimization passes (optional, via `optimize` feature)
- **Validation** — Grammar correctness checking
- **Normalization** — Converts complex symbols (Optional, Repeat, Choice, Sequence) into auxiliary rules

## Features

| Feature | Description |
|---------|-------------|
| `optimize` | Enable grammar optimization passes |
| `strict_docs` | Enforce documentation requirements |

## Usage

This crate is primarily used internally by `adze-tool` and `adze-glr-core`. Direct usage:

```rust
use adze_ir::{Grammar, Symbol, Rule};

let grammar = Grammar::builder()
    .rule("expression", /* ... */)
    .build();
```

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT License](../LICENSE-MIT) at your option.
