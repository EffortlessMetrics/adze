---
name: "Table Compression - Emit + decode for large tables"
about: "Implement table compression for large grammars"
title: "[FEATURE] Table Compression - Emit + decode for large tables"
labels: "enhancement, priority-4, optimization"
assignees: ""
---

## Overview
Table compression for large grammars not fully implemented. Need pooling + row runs.

## Implementation Checklist

### Compression Algorithm
- [ ] Implement in `tablegen/compress.rs`
```rust
// Large-table pooling
// Row run-length encoding
// Bit-packed symbol ranges
```

### Runtime Decoder
- [ ] Update `get_action(state, symbol)` for both paths
```rust
fn get_action(state: TSStateId, symbol: TSSymbol) -> Action {
  if self.use_compression {
    // Decode from compressed tables
  } else {
    // Direct lookup
  }
}
```

### Build Toggle
- [ ] Default compressed; `RUST_SITTER_NO_COMPRESS=1` for debug

## Tests

### Correctness
- [ ] For arith/json/python grammars
- [ ] Exhaustive check: uncompressed vs compressed identical for all (state, symbol)
- [ ] Round-trip: compress → decompress → identical

### Performance
- [ ] Binary size reduction ≥ 30%
- [ ] Runtime performance unchanged (< 2% overhead)
- [ ] Memory usage reduced

## Acceptance Criteria
- [x] Compression reduces table size by ≥ 30%
- [x] No correctness issues
- [x] < 2% runtime overhead

## Files to Modify
- `tablegen/src/compress.rs:53` - Remove validation stub
- `runtime/src/decoder.rs` - Add compressed path
- `tablegen/tests/compression_test.rs` - New tests