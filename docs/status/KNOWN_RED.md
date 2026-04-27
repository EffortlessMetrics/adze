# Known red

**Last updated:** 2026-04-26

This file tracks intentional exclusions from the supported lane:

- Required PR gate: `just ci-supported` locally, `CI / ci-supported` in GitHub checks

Rule: if something is excluded from the supported lane, it must be listed here with:
- what is excluded
- why
- how it becomes supported (or why it won't)

---

## ✅ Previously broken — now fixed

### `adze` (runtime) crate — RESOLVED
- **Was:** `cargo check -p adze` failed with ~20 errors (lifetime, type, borrow-checker issues).
- **Fixed:** All compile errors resolved. `cargo check -p adze` passes. `cargo fmt` and `cargo clippy` clean.
- **Date:** 2026-03-04

### Core pipeline crates
- `adze-ir`, `adze-glr-core`, `adze-tablegen`, `adze-common`, `adze-macro`, `adze-tool` all pass `cargo check`, `cargo clippy`, and `cargo test`.

---

## What the supported lane covers

`ci-supported` currently checks the **core pipeline** (7 crates: `adze`, `adze-macro`, `adze-tool`, `adze-common`, `adze-ir`, `adze-glr-core`, `adze-tablegen`):

- `cargo fmt --all --check`
- `cargo clippy` (supported crates, `-D warnings`)
- `cargo test` (supported crates: lib, tests, bins)
- `adze-glr-core` doctests with `serialization` feature

This lane is intentionally bounded so it stays reliable and fast enough for day-to-day work.

**Current status:** GREEN — all supported crates compile, lint clean, and tests pass. **2,460+ tests across feature combinations, 0 failures in supported lane.** Feature-combination matrix: 12/12 pass (all green). `cargo-audit` clean (0 vulnerabilities). WASM: all core crates compile for `wasm32-unknown-unknown`.

---

## What is excluded (and why)

### Not in the supported lane (workspace members / tools)
These are intentionally excluded for now because they are prototypes, platform-sensitive, heavier than the supported contract, or still stabilizing:

- `runtime2/` (alt runtime path; still converging)
- `cli/`, `lsp-generator/`, `playground/`, `wasm-demo/` (tooling/prototypes)
- `golden-tests/` (useful contract, but can be heavy and multi-language)
- `benchmarks/` (signal, not merge-blocking)
- `grammars/*` (valuable, but not yet a stable published surface)
- `crates/*` (47 BDD/governance microcrates; structure stable, READMEs added)

### Not in the supported lane (workflows)
These may run as optional signal (nightly/manual/canary), but are not required for merge:

- fuzzing lanes (20 targets exist but run on schedule/manual dispatch)
- wide platform matrices
- workflow_dispatch-only CI lanes and manual opt-ins (e.g. feature-matrix examples/burn-in paths)
- deployment workflows (mdBook / pages)
- performance regression canaries
- All other `.github/workflows/ci.yml` jobs are optional unless explicitly promoted in settings.

---


## Advisory product proof lane (non-blocking)

A broad-surface advisory lane now exists as `.github/workflows/product-proof.yml` and runs `scripts/ci-product.sh` on schedule/manual dispatch.

This lane is **not** part of required merge gates. It provides bounded smoke proof across product surfaces that are outside `ci-supported`.

Current canaries:

- `adze` runtime pure-rust smoke — **compile-only** (`cargo check -p adze --features pure-rust`)
- `adze-cli` smoke — **compile-only** (`cargo check -p adze-cli`)
- `adze-golden-tests` smoke — **compile-only** (`cargo test -p adze-golden-tests --features python-grammar --no-run`)
- `adze-benchmarks` canary — **compile-only** (`cargo bench -p adze-benchmarks --no-run`)
- `wasm-demo` canary — **compile-only** (`cargo check --manifest-path wasm-demo/Cargo.toml --target wasm32-unknown-unknown`)
- grammar smoke (`adze-python`) — **compile-only** (`cargo check -p adze-python`)
- `runtime2` canary — **compile-only** (`cargo test --manifest-path runtime2/Cargo.toml --no-run`)
- governance/BDD microcrate smoke (`adze-bdd-grid-core`) — **compile-only** (`cargo test -p adze-bdd-grid-core --lib --no-run`)

Notes:
- This lane intentionally does not provide full behavior proof; it is bounded canary signal only.
- If one canary is red, the advisory job can fail while remaining non-blocking due to workflow `continue-on-error: true`.

## Known warnings (non-blocking)

- ~~`rustdoc::private_intra_doc_links` warning in `adze` (runtime) crate doc build~~ — **Resolved.** 0 rustdoc warnings across supported crates.
- `unused manifest key` warnings in `lsp-generator/Cargo.toml` and `wasm-demo/Cargo.toml` — these are excluded crates.

---

## How something graduates into the supported lane

To add a crate/workflow to the supported lane, it must be:
- reproducible on a normal dev machine
- stable across the supported toolchain/MSRV
- bounded in time/resources
- documented (how to run it locally; common failure modes)

When you add something to `ci-supported`, update this file in the same PR.
