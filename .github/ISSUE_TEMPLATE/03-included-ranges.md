---
name: "Included Ranges - Make is_at_included_range_start real"
about: "Implement proper range boundary detection for multi-language parsing"
title: "[FEATURE] Included Ranges - Make is_at_included_range_start real"
labels: "enhancement, priority-3, external-scanner"
assignees: ""
---

## Overview
Support parsing specific byte ranges (e.g., JavaScript within HTML `<script>` tags).

## Implementation Checklist

### Range State
- [ ] Add range tracking to TSLexerAdapter
```rust
struct Ranges { 
  spans: Box<[Range<usize>]>, 
  next: usize 
}
```

### Boundary Detection
- [ ] Implement `is_at_included_range_start`
```rust
impl TSLexerAdapter {
  fn is_at_included_range_start(&self) -> bool {
    self.ranges.spans
      .get(self.ranges.next)
      .is_some_and(|r| r.start == self.cursor)
  }
}
```

### Advance Constraints
- [ ] Respect range boundaries in `advance`
```rust
fn advance(&mut self, skip: bool) -> bool {
  if self.cursor >= self.ranges.spans[self.ranges.next].end { 
    return false; 
  }
  // byte step + row/col updates; respect CRLF
}
```

## Tests

### HTML with Script Tags
- [ ] Parse `<script>...</script>` ranges
- [ ] Tokens switch correctly at boundaries
- [ ] Multiple disjoint ranges

### Edge Cases
- [ ] No ranges → always false, same lexing
- [ ] Empty ranges
- [ ] Overlapping ranges (error case)
- [ ] CRLF line endings at boundaries

## Acceptance Criteria
- [x] HTML+JS multi-language parsing works
- [x] Range transitions are precise to the byte
- [x] No performance impact when ranges not used

## Files to Modify
- `runtime/src/external_scanner_ffi.rs` - Range state and methods
- `runtime/src/ffi.rs:186` - Replace stub returning false
- `runtime/tests/included_ranges_test.rs` - New test file

## Dependencies
Requires Issue #2 (External Scanner) to be completed first.