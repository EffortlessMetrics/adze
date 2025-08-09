---
name: "Perf & safety harness"
about: "Add performance benchmarks and safety validation"
title: "[INFRASTRUCTURE] Perf & safety harness"
labels: "testing, performance, safety"
assignees: ""
---

## Overview
Need systematic performance tracking and safety validation.

## Implementation Checklist

### C FFI Harness
- [ ] Tiny C program in `tests/ffi/`
```c
// Load emitted language symbol
// Call ts_language_* functions
// Verify field counts, symbol names
// Check ABI compatibility
```
- [ ] Run in CI on Linux/macOS/Windows

### Tree Equality Helper
- [ ] Add `tree_eq` for incremental tests
```rust
fn tree_eq(a: &Tree, b: &Tree) -> bool {
  // Check structure, IDs, byte ranges
  // Ignore internal pointers
}
```

### Miri Coverage
- [ ] Run external scanners under miri
- [ ] Validate FFI boundaries
- [ ] Check for undefined behavior
```bash
cargo +nightly miri test -p runtime --features external_scanners
```

### Performance Benchmarks
- [ ] Use `criterion` for statistical rigor
- [ ] Track: parse time, memory, incremental speedup
- [ ] Baseline: tree-sitter C implementation
- [ ] Regression detection in CI

## Tests

### Benchmarks
- [ ] Small (100 lines), medium (1K), large (10K) files
- [ ] Different languages: JSON, Python, Rust
- [ ] Incremental: single-char, line, block edits
- [ ] Measure: time, allocations, cache misses

### Safety
- [ ] Miri: all unsafe blocks exercised
- [ ] Valgrind: no leaks in C scanner path
- [ ] Thread sanitizer: concurrent parsing
- [ ] Fuzzing: random input → no panics

## Acceptance Criteria
- [x] CI catches performance regressions > 5%
- [x] Miri passes on all configurations
- [x] FFI harness validates ABI
- [x] Benchmark suite runs nightly

## Files to Create
- `tests/ffi/harness.c` - C validation program
- `benches/parser_bench.rs` - Criterion benchmarks
- `tests/helpers/tree_eq.rs` - Comparison helper
- `.github/workflows/bench.yml` - Nightly benchmark job
- `.github/workflows/miri.yml` - Safety validation job