---
name: "get_goto_state panic - Improve message"
about: "Make get_goto_state panic more helpful for debugging"
title: "[IMPROVEMENT] get_goto_state panic - Better error message"
labels: "enhancement, good-first-issue, dx"
assignees: ""
---

## Overview
Current panic message unhelpful. Need debugging context.

## Implementation

### Current Code
```rust
// runtime/src/pure_parser.rs:1102
fn get_goto_state(...) -> TSStateId {
  panic!("get_goto_state called on pure_parser - this is a bug!")
}
```

### Improved Version
```rust
fn get_goto_state(state: TSStateId, symbol: TSSymbol, language: &Language) -> TSStateId {
  panic!(
    "get_goto_state called on pure_parser - this is a bug!\n\
     State: {}, Symbol: {} ({}), Grammar: {}\n\
     This likely means the parser is in an invalid state.\n\
     Please report this at https://github.com/rust-sitter/rust-sitter/issues",
    state, 
    symbol,
    language.symbol_name(symbol).unwrap_or("<unknown>"),
    language.name.unwrap_or("<unnamed>")
  )
}
```

## Tests
- [ ] Trigger panic in test to verify message format
- [ ] Ensure all context available at panic site

## Acceptance Criteria
- [x] Panic includes state, symbol, grammar name
- [x] Links to issue tracker
- [x] Suggests possible causes

## Files to Modify
- `runtime/src/pure_parser.rs:1102` - Update panic message