# Known Red: Test Compilation Errors

Last updated: 2026-02-20

## Open Items

### 8. `adze-runtime` (runtime2) - `glr_parse_simple` test failure

**Error:** `ParseError: no valid parse paths at byte 1`

**Root cause:** GLR parse table or tokenizer issue in runtime2.

---

### 9. `adze` (runtime) — `test_python_decoder_roundtrip`

**Error:** `Reduce mapping failed: no rule for (lhs=771, rhs_len=3)`

**Root cause:** Decoder production-id mapping for the Python grammar is not wired
correctly. The decode_parse_table path cannot resolve all reduce actions to grammar
rules, causing a panic during roundtrip verification.

**Tracked in:** `runtime/tests/decoder_regression.rs` (`#[ignore]`)

---

## Scope

Library code and test targets now pass clippy cleanly:
```bash
cargo clippy -p adze --all-targets -- -D warnings  # passes
```

The supported CI lane (`just ci-supported`) includes test targets for `adze`.

## Resolved Items

- **Items 1-7** (2026-02-20): Fixed all test compilation errors from API drift:
  - **`runtime/src/unified_parser.rs`** — Added `parse()` convenience method delegating to `parse_with_old_tree`
  - **`runtime/tests/integration_test.rs`** — Replaced `tree.root_kind()` with `tree.root_node().symbol()`
  - **`runtime/tests/unified_parser_test.rs`** — Compiles after `parse()` method was added
  - **`runtime/tests/tree_node_lifetime_test.rs`** — Switched from `TreeNodeData` to `TreeNode` to match `TreeArena::alloc` API
  - **`runtime/tests/parser_v3_test.rs`** — Changed import to `parser_v4::Parser`, switched to `parse_tree()` and `root.symbol.0`
  - **`runtime/tests/debug_ffi_fix.rs`** — Compiles after `parse()` method was added
  - **`runtime/tests/end_to_end.rs`** — Changed import to `parser_v4::Parser`, switched to `parse_tree()` and `root.symbol`
  - **`grammars/python/tests/smoke_test.rs`** — Replaced `tree.root_kind()` with `tree.root_node().symbol()`
  - **`grammars/python/tests/incremental_glr_test.rs`** — Fixed borrow checker issues with scoped tree lifetimes
  - **`runtime/tests/arena_allocator_test.rs`** — Added `#![allow(unexpected_cfgs)]` for `proptest` feature
  - **`runtime/tests/tree_node_data_test.rs`** — Added `#![allow(unexpected_cfgs)]` for `proptest` feature
- **`runtime/tests/arena_allocator_test.rs`** — `unexpected cfg condition value: proptest`
  warning suppressed by `#![allow(unexpected_cfgs)]`.
- **`runtime/tests/ts_compat_guardrails.rs`** — `named` → `is_named` field rename
  applied. Feature-gated behind `ts-compat` + `pure-rust` (non-default).
