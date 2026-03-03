# Adze Friction Log

**Last updated:** 2026-03-03

If it happens twice, it's not "user error". It's friction we own until we remove it or document it well enough that it stops recurring.

---

## Active Friction

| ID | Area | Symptom | Impact | Status | Link |
|---:|------|---------|--------|--------|------|
| FR-001 | Docs | Docs drift from dev head (README/book/guides disagree) | Users follow dead paths | Open | (issue) |
| FR-002 | CI | Too many workflows fail/cancel simultaneously on PRs | Signal is noisy | Open | (issue) |
| FR-003 | Dev loop | Supported gate is still heavy on constrained machines | Local iteration cost | Mitigated | (issue) |
| FR-004 | Status | Supported-lane exclusions aren't obvious | Confusing contributor loop | Open | (issue) |
| FR-005 | Macro | Leaf `transform` closures are captured but never executed | Type conversions (e.g. string to i32) fail silently | Open | [Issue #74](https://github.com/EffortlessMetrics/adze/issues/74) |
| FR-006 | Macro | `Extract` trait signature mismatch in `pure-rust` mode | Compilation errors (E0053, E0308) in user code | Resolved | - |
| FR-007 | Runtime | Lexer state pointer layout mismatch in `pure-rust` mode | Runtime `UnexpectedToken("end")` errors | Resolved | - |
| FR-008 | Tooling | `just` has permission issues on some systems | Commands fail with `/run/user/1000/just` errors | Open | - |
| FR-009 | Dev loop | Workspace build is very slow (10+ min for full check) | Developers avoid full validation locally | Open | - |
| FR-010 | Runtime | `runtime/src/pure_parser.rs` has parse errors | Blocks `cargo fmt` on entire workspace | Open | - |

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
**Status:** Open

### FR-002 - CI Workflow Noise

**Area:** ci
**Symptom:** PRs trigger dozens of overlapping workflows (benchmarks, tests, lints) that often conflict or cancel each other.
**Expected:** Clear, non-redundant signal on PR status.
**Actual:** Hard to tell if a failure is real or a CI glitch.
**Fix:** Consolidate workflows and use concurrency groups.
**Status:** Open

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
**Fix:** Add [`DEVELOPER_GUIDE.md`](../DEVELOPER_GUIDE.md) explaining support lanes.
**Status:** Open

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
**Fix:** Document workaround in DEVELOPER_GUIDE; consider switching to `cargo xtask` as primary entry point.
**Status:** Open

### FR-009 - Slow Workspace Build

**Area:** dev loop
**Symptom:** `cargo check --workspace` or `cargo build` takes 10+ minutes on standard hardware due to 47 microcrates in `crates/` plus the full core pipeline.
**Expected:** Developers can iterate quickly on individual crates.
**Actual:** Full workspace builds are prohibitively slow for local development.
**Fix:** Use per-crate `cargo check -p <crate>` for iteration; consider workspace partitioning.
**Status:** Open

### FR-010 - `pure_parser.rs` Parse Errors

**Area:** runtime
**Symptom:** `runtime/src/pure_parser.rs` contains Rust parse errors that prevent `cargo fmt` from formatting the file (and potentially the entire workspace if fmt is run with `--all`).
**Expected:** All `.rs` files parse cleanly.
**Actual:** The file has syntax-level issues that must be fixed before formatting or compilation can succeed.
**Fix:** Fix parse errors in `pure_parser.rs` as part of the runtime compile error remediation.
**Status:** Open

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
