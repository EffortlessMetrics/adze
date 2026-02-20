# Known Red: Test Compilation Errors

Last updated: 2026-02-20

These test files in `rust-sitter` (runtime crate) have **compilation errors**
or **clippy errors** that prevent `cargo clippy -p rust-sitter --all-targets -- -D warnings`
from passing. They represent API drift in test code, not library code.

**Note:** `pure-rust` is a **default** feature, so these errors appear even
without explicit feature flags.

The supported CI lane (`just ci-supported`) uses `--lib` for `rust-sitter` to
avoid these test compilation failures. Library code is clean.

---

## Compilation Errors (API drift)

### 1. `runtime/tests/integration_test.rs` (9 errors)

**Errors:** Type annotations needed (`E0282`), missing methods (`E0599`)

**Root cause:** Test relies on API signatures that have changed during
GLR parser refactoring.

**Reproduce:**
```bash
cargo clippy -p rust-sitter --tests -- -D warnings 2>&1 | grep integration_test
```

### 2. `runtime/tests/unified_parser_test.rs` (lines 143, 149, 162)

**Error:** `no method 'parse' found for struct 'Parser'`

**Root cause:** The unified parser API was refactored; `parse()` no longer
exists with this signature.

### 3. `runtime/tests/tree_node_lifetime_test.rs` (26 errors)

**Error:** `arguments to this method are incorrect` (`E0308`)

**Root cause:** `TreeNodeData` API changed; test passes wrong argument types
throughout.

### 4. `runtime/tests/parser_v3_test.rs` (3 errors)

**Error:** Various compilation errors from API drift.

### 5. `runtime/tests/conflict_preservation_runtime.rs` (line 118)

**Error:** `expected type, found module 'runtime_conflict_preservation'`

**Root cause:** `std::any::type_name::<runtime_conflict_preservation>()` passes
a module path where a type is expected.

### 6. `runtime/tests/arena_allocator_test.rs` (1 error)

**Error:** Compilation error from API drift.

### 7. `runtime/tests/test_action_decoding.rs` (1 error)

**Error:** `empty line after doc comment` (clippy, `-D warnings`)

### 8. `runtime/tests/test_ambiguous_expr_table_decode.rs` (1 error)

**Error:** `unused import: rust_sitter_glr_core::Action` (clippy, `-D warnings`)

### 9. `runtime/tests/parser_arena_integration_test.rs` (4 errors)

**Errors:** Unused import (`ArenaMetrics`), unnecessary mutable variable,
assertion always true, useless `vec![]`.

---

## Separate Crate Errors

### 10. `grammars/python/tests/smoke_test.rs` (lines 74, 78)

**Error:** `no method 'root_kind' found for struct 'Tree'`

**Root cause:** The `Tree` API does not expose a `root_kind()` method; test was
written against a planned but unimplemented interface.

**Reproduce:**
```bash
cargo clippy -p rust-sitter-python --all-targets -- -D warnings
```

### 11. `grammars/python/tests/incremental_glr_test.rs` (lines 60, 104, 145)

**Error:** `no method 'parse' found for struct 'Parser'`

**Root cause:** Same unified parser API drift as item 2 above.

---

## Other Crate Test Failures

### 12. `rust-sitter-macro` (10 snapshot test failures)

**Error:** Snapshot assertion failures - snapshots are stale after `GRAMMAR_NAME`
constant was added to the `Extract` trait impl.

**Root cause:** Macro output changed (added `const GRAMMAR_NAME` lines) but
`cargo insta review` was not run to update snapshots.

**Reproduce:**
```bash
cargo test -p rust-sitter-macro
```

**Fix:** Run `cargo insta review` and accept the new snapshots.

### 13. `rust-sitter-glr-core` - 2 doc test compilation failures

**Error:** `binary operation '==' cannot be applied to type 'ParseTable'`

**Root cause:** Doc tests use `assert_eq!` on `ParseTable` which doesn't implement
`PartialEq`. The doc examples in `serialization` are incorrect.

**Reproduce:**
```bash
cargo test -p rust-sitter-glr-core --doc
```

### 14. `rust-sitter-runtime` (runtime2) - `glr_parse_simple` test failure

**Error:** `ParseError: no valid parse paths at byte 1`

**Root cause:** GLR parse table or tokenizer issue in runtime2.

**Reproduce:**
```bash
cargo test -p rust-sitter-runtime
```

---

## Scope

All errors are in **test files only**. Library code passes clippy cleanly:
```bash
cargo clippy -p rust-sitter --lib -- -D warnings  # passes
```

The supported CI lane (`just ci-supported`) excludes the broken packages/targets.

## Feature-Gated Errors (non-default features)

### `runtime/tests/ts_compat_guardrails.rs` (lines 44, 61)

**Feature gate:** `ts-compat` (non-default)

**Error:** `no field 'named' on SymbolMetadata`

**Root cause:** `glr-core`'s `SymbolMetadata` uses `is_named`; this test
references the old field name `named`.

**Reproduce:**
```bash
cargo clippy -p rust-sitter --features ts-compat --all-targets -- -D warnings
```
