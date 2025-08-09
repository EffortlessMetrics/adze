---
name: "External Scanner - Wire the adapter into the loop"
about: "Connect external scanner to parse loop via TSLexerAdapter"
title: "[FEATURE] External Scanner - Wire the adapter into the loop"
labels: "enhancement, priority-2, parser-core"
assignees: ""
---

## Overview
External scanners (for Python indentation, nested comments, etc.) need proper adapter wiring into the parse loop.

## Implementation Checklist

### Adapter Implementation
- [ ] Fill TSLexerAdapter methods in `runtime/src/external_scanner_ffi.rs`
```rust
// Keep cursor (byte idx), mark_end (byte idx), and point (row/col)
fn advance(&mut self, skip: bool) { 
  // move cursor and point with precomputed line starts
}
fn mark_end(&mut self) { 
  // self.mark_end = self.cursor 
}
fn get_column(&self) -> u32 { 
  // point.column (do not recompute from start)
}
fn is_at_included_range_start(&self) -> bool { 
  // See issue #3
}
```

### Parse Loop Hook
- [ ] Integrate in `parser_v4::lex_step` (or equivalent)
```rust
if state_has_external(valid_symbols) {
  let mut adapter = TSLexerAdapter::new(src, cursor, ranges);
  let matched = scanner.scan(&mut adapter, &valid_symbols);
  if matched {
    // commit adapter.end_position() and token kind
    return Token::External { kind, len: adapter.len() };
  }
}
// fallback to normal lexer
```

### Scanner Types
- [ ] Rust scanner: `Arc<dyn ExternalScanner + Send + Sync>` per parser
- [ ] C scanner FFI:
  - `create() -> *mut T`
  - `scan(TSLexer*, bool* valid) -> bool`
  - `serialize/deserialize`
  - `destroy`

## Tests

### Python Indentation
- [ ] Multi-line programs with INDENT/DEDENT tokens
- [ ] Nested indentation levels
- [ ] Mixed tabs/spaces handling

### Nested Comments
- [ ] OCaml-style `(* nested (* comments *) *)` 
- [ ] Longest-match bypasses normal lexer

### State Persistence
- [ ] Serialize mid-parse → deserialize → continue
- [ ] Result identical to continuous parse

### Safety
- [ ] Miri: adapter & `destroy_lexer()` without UB
- [ ] Valgrind: no leaks in C scanner path

## Acceptance Criteria
- [x] Python grammar parses with external indent/dedent
- [x] Stateful scanners serialize/deserialize correctly
- [x] No memory safety issues under Miri
- [x] Performance within 5% of C implementation

## Files to Modify
- `runtime/src/external_scanner_ffi.rs` - TSLexerAdapter implementation
- `runtime/src/parser_v4.rs` - Parse loop integration
- `runtime/src/ffi.rs:186` - Fix stub returning false
- `runtime/tests/external_scanner_test.rs` - New test file

## Risk Notes
Touches unsafe code → land with miri + valgrind runs; put behind `external_scanners` feature for soft launch.