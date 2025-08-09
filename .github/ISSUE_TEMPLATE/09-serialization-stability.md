---
name: "Serialization stability"
about: "Ensure deterministic and stable tree serialization"
title: "[FEATURE] Serialization stability"
labels: "enhancement, priority-5, testing"
assignees: ""
---

## Overview
Ensure deterministic serialization for golden tests and debugging.

## Implementation Checklist

### Determinism
- [ ] Stable child order traversal
- [ ] Consistent numeric formatting
- [ ] Proper string escaping rules
```rust
// Escape: \n \r \t \\ \"
// Unicode: \uXXXX for control chars
// Byte order: always UTF-8
```

### Formats
- [ ] JSON: Compact and pretty modes
- [ ] S-expression: Lisp-style with fields
- [ ] DOT: Graphviz format for visualization

### Golden Tests
- [ ] Create `tests/serialization_golden/`
- [ ] Store `.sexp`, `.json` for each grammar
- [ ] Byte-for-byte comparison in CI

## Tests

### Stability
- [ ] Parse → serialize → same output 100x
- [ ] Different parser instances → same output
- [ ] After incremental edit → unaffected nodes identical

### Round-trip (future)
- [ ] `Tree::from_json` (optional, later)
- [ ] Preserves all node properties

## Acceptance Criteria
- [x] Deterministic output across runs
- [x] Golden tests prevent regressions
- [x] All formats properly escaped

## Files to Modify
- `runtime/src/serialization.rs` - Ensure determinism
- `runtime/tests/serialization_golden/` - Golden files
- `runtime/tests/serialization_test.rs` - Stability tests