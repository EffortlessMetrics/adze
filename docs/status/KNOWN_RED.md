# Known red

**Last updated:** 2026-03-03

This file tracks intentional exclusions from the supported lane:

- Required PR gate: `just ci-supported` locally, `CI / ci-supported` in GitHub checks

Rule: if something is excluded from the supported lane, it must be listed here with:
- what is excluded
- why
- how it becomes supported (or why it won't)

---

## 🔴 Currently broken

### `adze` (runtime) crate does not compile on `main`
- **What:** `cargo check -p adze` fails with ~20 errors (lifetime, type, borrow-checker issues in `runtime/src/`).
- **Why:** Accumulated breakage from pure-Rust integration work; `pure_parser.rs` also has parse errors blocking `cargo fmt`.
- **Impact:** `ci-supported` gate is **red**. No workspace-wide `cargo check` or `cargo test` passes.
- **Graduation:** All compile errors fixed, `cargo check -p adze` passes, `ci-supported` returns to green.

### Core pipeline crates compile cleanly
- `adze-ir`, `adze-glr-core`, `adze-tablegen` all pass `cargo check` as of 2026-03-03.
- These are **not** blocked by the runtime breakage.

---

## What the supported lane covers

`ci-supported` currently checks the **core pipeline**:

- `cargo fmt --check`
- `cargo clippy` (core crates)
- `cargo test` (core crates)
- `glr-core` doctests with `serialization`

This lane is intentionally bounded so it stays reliable and fast enough for day-to-day work.

**Current status:** RED — blocked by runtime compile errors (see above).

---

## What is excluded (and why)

### Not in the supported lane (workspace members / tools)
These are intentionally excluded for now because they are prototypes, platform-sensitive, heavier than the supported contract, or still stabilizing:

- `runtime/` (`adze` crate — currently broken, being fixed; see above)
- `runtime2/` (alt runtime path; still converging)
- `cli/`, `lsp-generator/`, `playground/`, `wasm-demo/` (tooling/prototypes)
- `golden-tests/` (useful contract, but can be heavy and multi-language)
- `benchmarks/` (signal, not merge-blocking)
- `grammars/*` (valuable, but not yet a stable published surface)
- `crates/*` (47 BDD/governance microcrates; still stabilizing structure, most lack READMEs)

### Not in the supported lane (workflows)
These may run as optional signal (nightly/manual/canary), but are not required for merge:

- fuzzing lanes
- wide platform matrices
- workflow_dispatch-only CI lanes and manual opt-ins (e.g. feature-matrix examples/burn-in paths)
- deployment workflows (mdBook / pages)
- performance regression canaries
- All other `.github/workflows/ci.yml` jobs are optional unless explicitly promoted in settings.

---

## How something graduates into the supported lane

To add a crate/workflow to the supported lane, it must be:
- reproducible on a normal dev machine
- stable across the supported toolchain/MSRV
- bounded in time/resources
- documented (how to run it locally; common failure modes)

### Runtime crate graduation criteria
The `adze` runtime crate returns to the supported lane when:
1. `cargo check -p adze` passes with zero errors
2. `cargo test -p adze` passes (existing tests, no regressions)
3. `cargo fmt -- --check` passes (requires fixing `pure_parser.rs` parse errors)
4. `cargo clippy -p adze` passes with no warnings

When you add something to `ci-supported`, update this file in the same PR.
