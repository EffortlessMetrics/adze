# Migration Guide

This document describes breaking changes and how to migrate your code.

## 0.8.0

### Removed: `compress_default`

The deprecated `compress_default` method has been removed. Replace with:

```rust
use rust_sitter_tablegen::compress::TableCompressor;
use rust_sitter_tablegen::helpers::collect_token_indices;

// Before (0.7.x):
let compressed = compressor.compress_default(&parse_table, &grammar)?;

// After (0.8.0):
let token_indices = collect_token_indices(&grammar, &parse_table);
let start_can_be_empty = eof_accepts_or_reduces(&parse_table);
let compressed = compressor.compress(&parse_table, &token_indices, start_can_be_empty)?;
```

The new API is more explicit about what data is needed and gives you control over
the `start_can_be_empty` parameter for performance optimization.

## 0.7.0

### TableCompressor API Changes

The `TableCompressor` constructor no longer takes grammar and parse table parameters.
Instead, these are passed to the `compress` method directly.

```rust
// Before (0.6.x):
let compressor = TableCompressor::new(&grammar, &parse_table);
let compressed = compressor.compress(&options)?;

// After (0.7.0):
let compressor = TableCompressor::new();
let token_indices = collect_token_indices(&grammar, &parse_table);
let compressed = compressor.compress(&parse_table, &token_indices, start_can_be_empty)?;
```