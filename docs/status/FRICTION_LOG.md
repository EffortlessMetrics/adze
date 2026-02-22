# Adze Friction Log

**Last updated:** 2026-02-22

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

---

## Detailed Entries

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
**Fix:** Add `DEVELOPER_GUIDE.md` explaining support lanes.
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
