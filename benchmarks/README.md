# adze-benchmarks

Performance benchmarks for the Adze parser and GLR engine.

## Overview

This crate contains criterion-based benchmarks for measuring Adze's parsing performance
across different grammars and input sizes. It benchmarks both the Tree-sitter backend
and the pure-Rust GLR implementation.

## Benchmarks

- **Parser performance**: Parse time for various input sizes
- **GLR engine**: Fork/merge performance with ambiguous grammars
- **Table generation**: IR → parse table compilation speed
- **Compression**: Table compression algorithm efficiency

## Running

```bash
# Run all benchmarks
cargo bench -p adze-benchmarks

# Run a specific benchmark
cargo bench -p adze-benchmarks -- parse
```

## Dependencies

- [criterion](https://crates.io/crates/criterion) - Statistical benchmarking framework
- Adze core crates for parser and grammar access

## License

MIT OR Apache-2.0
