---
name: "CLI - dlopen and real corpus runner"
about: "Add dynamic library loading and corpus testing to CLI"
title: "[FEATURE] CLI - dlopen and real corpus runner"
labels: "enhancement, priority-3, tooling"
assignees: ""
---

## Overview
CLI currently generates temp Cargo project every time. Need direct .so/.dll loading and corpus runner.

## Implementation Checklist

### Parse Command
- [ ] Add flags: `--lib <path> [--symbol <sym>]`
- [ ] Use `libloading` for dynamic loading
```rust
let lib = unsafe { Library::new(path)? };
let language_fn: Symbol<unsafe extern "C" fn() -> *const TSLanguage> = 
  lib.get(symbol.as_bytes())?;
let language = unsafe { language_fn() };
```
- [ ] Parse stdin/file with loaded grammar
- [ ] Output formats: `--format {json,sexp,dot}`

### Test Command
- [ ] Discover `tests/corpus/**/*.txt` sections
- [ ] Parse each test case
- [ ] Compare to `.expected` or inline expectation
- [ ] `--update` flag to rewrite expectations
- [ ] Exit 0/1; show unified diffs

### Error Handling
- [ ] Missing symbol → helpful message with nm output
- [ ] Wrong ABI → detect and explain
- [ ] Platform-specific loading errors

## Tests

### Integration Tests
- [ ] Use `assert_cmd` to spawn CLI
- [ ] Test Windows/macOS/Linux .so/.dll loading
- [ ] Missing library → appropriate error
- [ ] Wrong symbol → lists available symbols

### Corpus Tests
- [ ] Parse tree-sitter format corpus files
- [ ] Handle all delimiters correctly
- [ ] Update mode preserves formatting

## Acceptance Criteria
- [x] `rust-sitter parse --lib parser.so` works
- [x] `rust-sitter test` runs corpus with diffs
- [x] Cross-platform library loading
- [x] Helpful error messages

## Files to Modify
- `cli/src/parse.rs` - Remove temp project generation
- `cli/src/test.rs` - New corpus runner
- `cli/Cargo.toml` - Add `libloading` dependency