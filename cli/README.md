# adze-cli

Command-line interface for [Adze](https://github.com/EffortlessMetrics/adze) grammar development.

## Overview

`adze-cli` provides the `adze` command-line tool for grammar validation, inspection, debugging, and development workflows. It wraps the core Adze libraries into an ergonomic developer experience.

## Installation

```bash
cargo install adze-cli
```

## Usage

```bash
# Validate a grammar file
adze validate path/to/grammar.rs

# Inspect grammar structure
adze inspect path/to/grammar.rs

# Watch for changes and rebuild
adze watch path/to/grammar.rs
```

## Features

| Feature | Description |
|---------|-------------|
| `pure-rust` | Use pure-Rust parsing backend |
| `tree-sitter-standard` | Use standard C Tree-sitter backend |
| `tree-sitter-c2rust` | Use c2rust Tree-sitter backend |
| `python-grammar` | Include Python grammar support |
| `javascript-grammar` | Include JavaScript grammar support |
| `dynamic` | Enable dynamic library loading |

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or [MIT License](../LICENSE-MIT) at your option.
