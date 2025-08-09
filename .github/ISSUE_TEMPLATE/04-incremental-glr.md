---
name: "Incremental GLR - Localized reparse, not full"
about: "Implement efficient incremental parsing for GLR parser"
title: "[FEATURE] Incremental GLR - Localized reparse, not full"
labels: "enhancement, priority-2, performance"
assignees: ""
---

## Overview
Currently falls back to full reparse. Need windowed recompute for performance.

## Implementation Checklist

### Edit Mapping
- [ ] Map `Edit { old_byte.., new_len }` to dirty window
- [ ] Find minimal covering ancestor span
- [ ] Track affected byte ranges

### Incremental Algorithm
- [ ] Implement in `glr_incremental::reparse`
```rust
// Re-lex only dirty window
// Restart GLR from closest stable stack frontier
// Reuse SubtreeIds outside window
// Renumber only inside; adjust byte/point deltas upward
```

### Integration
- [ ] Fix `unified_parser::parse_with_old_tree`
```rust
pub fn parse_with_old_tree(&mut self, source: &[u8], old_tree: Option<&Tree>, edit: Option<&Edit>) -> Option<Tree> {
  // Call incremental path, not full reparse
  glr_incremental::reparse(self, source, old_tree, edit)
}
```

## Tests

### Correctness
- [ ] Single-line insert/delete/replace at head/mid/tail
- [ ] Structural equality to fresh parse
- [ ] Unchanged subtree IDs outside window
- [ ] Property test: random edits vs fresh parse equality

### Performance
- [ ] ≥5× faster than full parse on 5k-line file single-line edit
- [ ] Memory usage stays flat with repeated edits
- [ ] Profile hotspots with `perf`/`flamegraph`

## Acceptance Criteria
- [x] Incremental parse produces identical trees to full parse
- [x] 5× speedup on typical single-line edits
- [x] Tests in `incremental_glr_comprehensive_test.rs` pass

## Files to Modify
- `runtime/src/unified_parser.rs:106` - Remove fallback
- `runtime/src/glr_incremental.rs` - Core algorithm
- Re-enable `runtime/tests/incremental_glr_comprehensive_test.rs`
- Re-enable `runtime/tests/property_incremental_test.rs`

## Risk Notes
Correctness > speed. Keep clean fallback to full parse. Add `RUST_SITTER_DEBUG_INCREMENTAL=1` for CI debugging.