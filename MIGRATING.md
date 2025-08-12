# Migration Guide

This document describes breaking changes and how to migrate your code.

## 0.8.0

### Error Unification

New crate errors provide consistent error handling across the workspace:
- `ir::IrError` - IR-level errors (via `thiserror`)
- `glr-core::GLRError` (re-exported as `GlrError`) - Parser generation errors
- `tablegen::TableGenError` - Table generation and compression errors

The `tablegen` crate now has transparent error variants that preserve source chains:

```rust
pub enum TableGenError {
    // ... existing variants ...
    
    #[error(transparent)]
    Glr(#[from] rust_sitter_glr_core::GLRError),
    
    #[error(transparent)]
    Ir(#[from] rust_sitter_ir::error::IrError),
}
```

This enables better error debugging with preserved source chains:

```rust
// Error sources are preserved through conversions
let result: Result<_, TableGenError> = build_lr1_automaton(&g, &ff)
    .map_err(Into::into)?;
    
// You can traverse the error chain
if let Err(e) = result {
    let mut source = Some(&e as &dyn std::error::Error);
    while let Some(err) = source {
        eprintln!("Error: {}", err);
        source = err.source();
    }
}
```

When handling errors, you can now match on the specific typed variants:

```rust
use rust_sitter_tablegen::error::TableGenError;

match err {
    TableGenError::Ir(e)  => eprintln!("IR error: {e}"),
    TableGenError::Glr(e) => eprintln!("GLR error: {e}"),
    TableGenError::TableGeneration(msg) => eprintln!("Table generation: {msg}"),
    other => eprintln!("Other tablegen error: {other}"),
}
```

For now `GLRError` remains the canonical name for compatibility. We may standardize
on `GlrError` in a future release with a deprecation window.

### Documentation Features

New `strict_docs` feature flag for conditional documentation enforcement:

```toml
[features]
strict_docs = []  # Enforces documentation at compile time
```

When building documentation for docs.rs:

```bash
RUSTDOCFLAGS="--cfg docsrs" cargo doc --features strict_docs
```

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