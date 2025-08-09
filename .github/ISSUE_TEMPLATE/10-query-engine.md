---
name: "Query engine completeness"
about: "Complete query DSL implementation with all predicates"
title: "[FEATURE] Query engine completeness"
labels: "enhancement, priority-2, blocked-by-fields"
assignees: ""
---

## Overview
Query system partially implemented. Needs field support (#1) and predicates.

## Implementation Checklist

### DSL Parser
- [ ] Complete pattern parser
- [ ] Support all operators: `.` `?` `+` `*` `@` `#`
- [ ] Field constraints: `field:(pattern)`

### Predicates
- [ ] `#eq?` - String equality
- [ ] `#not-eq?` - String inequality  
- [ ] `#match?` - Regex matching
- [ ] `#not-match?` - Negative regex
- [ ] `#is?` - Type checking
- [ ] `#is-not?` - Negative type check
- [ ] `#set!` - Property setting

### Cursor Optimization
- [ ] Precompute sibling ranges
- [ ] Avoid O(N²) sibling scans
```rust
struct CursorState {
  sibling_ranges: Vec<Range<usize>>,
  current_depth: usize,
}
```

## Tests

### Query Corpus
- [ ] Port tree-sitter query test suite
- [ ] Capture span + order goldens
- [ ] All predicate combinations

### Edge Cases
- [ ] Alternation: `[(foo) (bar)]`
- [ ] Negative predicates
- [ ] Anchor constraints: `.` prefix
- [ ] Wildcard: `_` matching

## Acceptance Criteria
- [x] All tree-sitter queries parse and execute
- [x] Field constraints work (requires #1)
- [x] Performance: < O(N²) for deep trees
- [x] Predicate evaluation correct

## Files to Modify
- `runtime/src/query.rs` - Predicate implementations
- `runtime/tests/test_query_predicates.rs` - Re-enable
- `runtime/tests/query_corpus/` - Test cases

## Dependencies
Blocked by Issue #1 (Field Names) for field constraint support.