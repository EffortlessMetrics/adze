# Adze Friction Log

**Last updated:** 2026-03-16

If it happens twice, it's not "user error". It's friction we own until we remove it or document it well enough that it stops recurring.

---

## Active Friction

| ID | Area | Symptom | Impact | Status | Link |
|---:|------|---------|--------|--------|------|
| FR-001 | Docs | Docs drift from dev head (README/book/guides disagree) | Users follow dead paths | Mitigated | (issue) |
| FR-002 | CI | Too many workflows fail/cancel simultaneously on PRs | Signal is noisy | Mitigated | (issue) |
| FR-003 | Dev loop | Supported gate is still heavy on constrained machines | Local iteration cost | Mitigated | (issue) |
| FR-004 | Status | Supported-lane exclusions aren't obvious | Confusing contributor loop | Mitigated | (issue) |
| FR-005 | Macro | Leaf `transform` closures are captured but never executed | Type conversions (e.g. string to i32) fail silently | Open | [Issue #74](https://github.com/EffortlessMetrics/adze/issues/74) |
| FR-006 | Macro | `Extract` trait signature mismatch in `pure-rust` mode | Compilation errors (E0053, E0308) in user code | Resolved | - |
| FR-007 | Runtime | Lexer state pointer layout mismatch in `pure-rust` mode | Runtime `UnexpectedToken("end")` errors | Resolved | - |
| FR-008 | Tooling | `just` has permission issues on some systems | Commands fail with `/run/user/1000/just` errors | Mitigated | - |
| FR-009 | Dev loop | Workspace build is very slow (10+ min for full check) | Developers avoid full validation locally | Open | - |
| FR-010 | Runtime | `runtime/src/pure_parser.rs` has parse errors | Blocks `cargo fmt` on entire workspace | Resolved | - |
| FR-011 | Docs | `rustdoc::private_intra_doc_links` warning in runtime | Cosmetic noise in doc build | Resolved | - |
| FR-012 | Publishing | No `cargo package` dry-run in CI | Broken publishes not caught early | Open | - |
| FR-013 | Tooling | No CLI binary yet (`adze check`, `adze stats`) | Grammar validation requires writing Rust | Open | - |
| FR-014 | Runtime | Some `adze` runtime integration tests fail to compile | Stale API references in test files (Node, etc) | Open | - |
| FR-015 | Testing | Feature matrix expected failure (`feature_profile_resolve_backend`) | 11/12 pass, 1 expected failure | Open | - |

---

## Detailed Entries

### FR-006 - Extract Trait Signature Mismatch

**Area:** macro
**Symptom:** Users enabling the `pure-rust` feature encounter compilation errors like `method extract has an incompatible type for trait`.
**Expected:** The macro automatically generates the correct signature based on enabled features.
**Actual:** The macro was emitting `Option<Node>` instead of `Option<&ParsedNode>` because it wasn't correctly detecting the target crate's features.
**Fix:** Updated `macro/src/expansion.rs` to use `cfg!(feature = "pure-rust")` at macro-expansion time to choose the correct tokens.
**Status:** Resolved

### FR-007 - Lexer State Pointer Mismatch

**Area:** runtime
**Symptom:** Parsers built with `ADZE_USE_PURE_RUST=1` fail at runtime with `UnexpectedToken("end")` even for valid input.
**Expected:** The generated lexer correctly tokenizes the input.
**Actual:** The `adze-tool` was generating a lexer that cast the state pointer to a custom `LexerState` struct that didn't match the `TsLexer` struct passed by the runtime.
**Fix:** Updated `tablegen/src/lexer_gen.rs` to generate a lexer that uses the standard `TsLexer` ABI (function pointers for lookahead/advance).
**Status:** Resolved

### FR-001 - Documentation Drift

**Area:** docs
**Symptom:** README.md and book examples refer to old `rust-sitter` naming or outdated macro syntax.
**Expected:** Documentation matches the current `adze` 0.8.0-dev state.
**Actual:** Users encounter compilation errors when copying examples.
**Fix:** Perform a repository-wide documentation audit and sync.
**Progress:**
- **Priority 1 (Fixed):** Version references updated from 0.5.0-beta/0.6 to 0.8; feature names corrected (glr-core → glr, incremental → incremental_glr)
- **Priority 2/3 (Remaining):** Book content sync, tutorial updates, advanced examples
**Status:** Mitigated

### FR-002 - CI Workflow Noise

**Area:** ci
**Symptom:** PRs trigger dozens of overlapping workflows (benchmarks, tests, lints) that often conflict or cancel each other.
**Expected:** Clear, non-redundant signal on PR status.
**Actual:** Hard to tell if a failure is real or a CI glitch.
**Fix:** Added concurrency groups (`cancel-in-progress`) and feature matrix job. Lint/test jobs gated by event type to reduce noise.
**Status:** Mitigated

### FR-003 - Heavy Local Dev Loop

**Area:** tooling
**Symptom:** Running the full test suite (`xtask test`) requires significant memory and CPU, causing OOMs on 16GB machines.
**Expected:** Developers can iterate on features without crashing their environment.
**Actual:** CI is often the only place to run full validation.
**Fix:** Optimized `pure-rust` builder and split tests into smaller bundles.
**Status:** Mitigated

### FR-004 - Undocumented Support Lanes

**Area:** publishing
**Symptom:** Some crates are excluded from the default workspace build (via `exclude` in Cargo.toml) but the reason isn't documented.
**Expected:** Contributors know which crates require special toolchains (Node.js, C compilers).
**Actual:** Confusion when `cargo build --workspace` skips important crates.
**Fix:** [`DEVELOPER_GUIDE.md`](../DEVELOPER_GUIDE.md) added. [`KNOWN_RED.md`](./KNOWN_RED.md) documents exclusions. READMEs added to `crates/` microcrates.
**Status:** Mitigated

### FR-005 - Transform Closure Capture Bug

**Area:** macro
**Symptom:** Using `#[adze::leaf(transform = ...)]` has no effect; the raw string is always returned.
**Expected:** The closure is executed during the `extract` phase to convert the value.
**Actual:** `adze-macro` generates code that captures the closure but never calls it.
**Repro:** Define a leaf with `transform = |s| s.len()`, observe it still returns the string.
**Fix:** Update `macro/src/expansion.rs` to generate call sites for captured closures.
**Status:** Open
**Links:** [Issue #74](https://github.com/EffortlessMetrics/adze/issues/74)

### FR-008 - `just` Permission Issues

**Area:** tooling
**Symptom:** Running `just` commands fails with permission errors related to `/run/user/1000/just` on some Linux systems.
**Expected:** `just` recipes execute without filesystem permission issues.
**Actual:** Users see permission denied errors; workaround is to set `JUST_TMPDIR` or use `cargo` directly.
**Fix:** Workaround documented. `just` runtime dir permission fix applied. `cargo` commands work as primary alternative.
**Status:** Mitigated

### FR-009 - Slow Workspace Build

**Area:** dev loop
**Symptom:** `cargo check --workspace` or `cargo build` takes 10+ minutes on standard hardware due to 47 microcrates in `crates/` plus the full core pipeline.
**Expected:** Developers can iterate quickly on individual crates.
**Actual:** Full workspace builds are prohibitively slow for local development.
**Fix:** Use per-crate `cargo check -p <crate>` for iteration; consider workspace partitioning.
**Status:** Open

### FR-010 - `pure_parser.rs` Parse Errors

**Area:** runtime
**Symptom:** `runtime/src/pure_parser.rs` contained Rust parse errors that prevented `cargo fmt` from formatting the file.
**Expected:** All `.rs` files parse cleanly.
**Actual:** The file had syntax-level issues blocking formatting and compilation.
**Fix:** All 20 compile errors in the runtime crate resolved. `cargo fmt` and `cargo check` now pass.
**Status:** Resolved

### FR-011 - Rustdoc Private Intra-Doc Links Warning

**Area:** docs
**Symptom:** `cargo doc -p adze` emits a `rustdoc::private_intra_doc_links` warning.
**Expected:** Clean doc build with no warnings.
**Actual:** One warning about private intra-doc links in the runtime crate.
**Fix:** Doc links updated to reference public items only.
**Status:** Resolved (Wave 6)

### FR-012 - No `cargo package` Dry-Run in CI

**Area:** publishing
**Symptom:** Publishing errors (missing files, bad metadata) are only discovered at `cargo publish` time.
**Expected:** CI catches packaging issues before merge.
**Actual:** No `cargo package --dry-run` step in the CI pipeline.
**Fix:** Add `cargo package --dry-run` for core crates to CI.
**Status:** Open

### FR-013 - No CLI Binary

**Area:** tooling
**Symptom:** To validate a grammar, users must write a full Rust program with `build.rs` integration.
**Expected:** A CLI command like `adze check grammar.rs` validates grammars without a full project.
**Actual:** No CLI binary exists yet.
**Fix:** Implement `adze check` and `adze stats` subcommands.
**Status:** Open

### FR-014 - Stale Runtime Test API References

**Area:** runtime
**Symptom:** Several `adze` runtime integration test files fail to compile with `use of undeclared type Node` and similar errors.
**Expected:** All test files compile and run.
**Actual:** Tests like `lexer_tests`, `simd_lexer_test`, `test_glr_integration`, `test_abi_contract`, `error_recovery_tests` reference APIs (`Node`, etc.) that were removed or renamed during the pure-Rust runtime refactor.
**Fix:** Update test files to use current API surface or remove tests that duplicate coverage already in the supported lane.
**Status:** Open
**Discovered:** Wave 14

### FR-015 - Feature Matrix Expected Failure

**Area:** testing
**Symptom:** `feature_profile_resolve_backend` test in `adze-feature-policy-contract` panics with "Grammar has shift/reduce or reduce/reduce conflicts, but the GLR feature is not enabled."
**Expected:** Feature matrix: 12/12 pass.
**Actual:** 11/12 pass; 1 expected failure due to intentional GLR feature gating logic being tested without the GLR feature enabled.
**Fix:** Either mark the test as `#[ignore]` with a reason, or adjust the test to correctly handle the feature-absent case.
**Status:** Open
**Discovered:** Wave 14

---

## Entry Template

### FR-XXX - <short title>

**Area:** docs / ci / tooling / runtime / publishing
**Symptom:** what the user experiences
**Expected:** what they thought would happen
**Actual:** what happened
**Repro:** exact commands + environment
**Fix:** what removes this friction
**Status:** Open / Mitigated / Resolved
**Links:** issue, PR, related docs
