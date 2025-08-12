# Migration Guide

This document describes breaking changes and how to migrate your code.

## 0.8.0

### Error Unification

New crate errors provide consistent error handling across the workspace:
- `ir::IrError` - IR-level errors (via `thiserror`)
- `glr-core::GLRError` (re-exported as `GlrError`) - Parser generation errors
- `tablegen::TableGenError` - Table generation and compression errors

The `tablegen` crate implements `From<GLRError>` and `From<IrError>`, so the `?` operator
will automatically convert upstream errors into `TableGenError`:

```rust
// In tablegen code, these all work with `?`:
let ff = FirstFollowSets::compute(&g);  // Returns GlrResult
let pt = build_lr1_automaton(&g, &ff)?; // GLRError -> TableGenError
let compressed = compressor.compress(&pt, &indices, nullable)?; // Already TableGenError
```

For now `GLRError` remains the canonical name for compatibility. We may standardize
on `GlrError` in a future release with a deprecation window.

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