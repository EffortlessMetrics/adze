# Known Red: Test Compilation Errors

Last updated: 2026-02-20

These test files in `rust-sitter` (runtime crate) have **compilation errors**
that prevent `cargo clippy -p rust-sitter --all-targets -- -D warnings`
from passing. They represent API drift in test code, not library code.

The supported CI lane (`just ci-supported`) uses `--lib` for `rust-sitter` to
avoid these test compilation failures. Library code is clean.

---

## Compilation Errors (API drift)

### 1. `runtime/tests/integration_test.rs` (9 errors)

**Errors:** Type annotations needed (`E0282`), missing methods (`E0599`)

**Root cause:** Test relies on API signatures that have changed during
GLR parser refactoring.

### 2. `runtime/tests/unified_parser_test.rs`

**Error:** `no method 'parse' found for struct 'Parser'`

**Root cause:** The unified parser API was refactored; `parse()` no longer
exists with this signature.

### 3. `runtime/tests/tree_node_lifetime_test.rs` (26 errors)

**Error:** `arguments to this method are incorrect` (`E0308`)

**Root cause:** `TreeNodeData` API changed; test passes wrong argument types.

### 4. `runtime/tests/parser_v3_test.rs` (3 errors)

**Error:** Various compilation errors from API drift.

### 5. `runtime/tests/debug_ffi_fix.rs`

**Error:** `no method 'parse' found for struct 'Parser'`

**Root cause:** Same unified parser API drift as item 2.

---

## Separate Crate Errors

### 6. `grammars/python/tests/smoke_test.rs`

**Error:** `no method 'root_kind' found for struct 'Tree'`

**Root cause:** The `Tree` API does not expose a `root_kind()` method; test was
written against a planned but unimplemented interface.

### 7. `grammars/python/tests/incremental_glr_test.rs`

**Error:** `no method 'parse' found for struct 'Parser'`

**Root cause:** Same unified parser API drift as item 2.

### 8. `rust-sitter-runtime` (runtime2) - `glr_parse_simple` test failure

**Error:** `ParseError: no valid parse paths at byte 1`

**Root cause:** GLR parse table or tokenizer issue in runtime2.

---

## Scope

All errors are in **test files only**. Library code passes clippy cleanly:
```bash
cargo clippy -p rust-sitter --lib -- -D warnings  # passes
```

The supported CI lane (`just ci-supported`) excludes the broken test targets.

## Resolved Items

- **`runtime/tests/arena_allocator_test.rs`** — `unexpected cfg condition value: proptest`
  warning already suppressed by `#[allow(unexpected_cfgs)]` (line 418). No action needed.
- **`runtime/tests/ts_compat_guardrails.rs`** — `named` → `is_named` field rename
  applied. Feature-gated behind `ts-compat` + `pure-rust` (non-default).
