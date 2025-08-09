---
name: "Timeouts & Cancellation - Actually enforce"
about: "Add timeout and cancellation support to parser_v4"
title: "[FEATURE] Timeouts & Cancellation - Actually enforce"
labels: "enhancement, priority-3, safety"
assignees: ""
---

## Overview
Timeout/cancellation support is stubbed. Need actual enforcement in parse loop.

## Implementation Checklist

### Parser State
- [ ] Add timing fields to parser
```rust
struct Parser {
  start_time: Option<Instant>,
  timeout_micros: u64,
  cancellation_flag: Option<Arc<AtomicBool>>,
  // Check every K=4096 transitions
}
```

### Main Loop Checks
- [ ] Add to parser_v4 main loop
```rust
// Every K transitions:
if self.transitions % 4096 == 0 {
  if let Some(start) = self.start_time {
    if start.elapsed() > Duration::from_micros(self.timeout_micros) {
      return Err(ParseAborted::Timeout);
    }
  }
  if let Some(flag) = &self.cancellation_flag {
    if flag.load(Ordering::Relaxed) {
      return Err(ParseAborted::Cancelled);
    }
  }
}
```

### API Integration
- [ ] Wire in `unified_parser.rs:54`
```rust
if let Some(timeout) = self.timeout_micros {
  v4_parser.set_timeout(timeout);
}
if let Some(flag) = &self.cancellation_flag {
  v4_parser.set_cancellation_flag(flag.clone());
}
```

## Tests

### Timeout
- [ ] Huge/ambiguous input + tiny timeout → deterministic abort
- [ ] Parser reusable after timeout
- [ ] Timeout precision within 10ms

### Cancellation
- [ ] Toggle atomic from another thread → abort quickly
- [ ] Cancellation checked within 100ms
- [ ] Parser reusable after cancellation

### Performance
- [ ] K=4096 has no measurable impact (< 1% overhead)
- [ ] Profile with `perf` to verify

## Acceptance Criteria
- [x] Timeouts enforced within specified duration
- [x] Cancellation flag checked regularly
- [x] Parser remains reusable after abort
- [x] < 1% performance overhead

## Files to Modify
- `runtime/src/parser_v4.rs` - Add timeout/cancellation fields and checks
- `runtime/src/unified_parser.rs:54` - Remove TODO, wire settings
- `runtime/tests/timeout_test.rs` - New test file

## Risk Notes
K=4096 chosen for balance. Too small = overhead, too large = poor responsiveness. Tune based on benchmarks.