# Known red

**Last updated:** 2026-02-21

This file tracks intentional exclusions from the supported lane:

- `just ci-supported`
- GitHub required check: `CI / ci-supported`

Rule: if something is excluded from the supported lane, it must be listed here with:
- what is excluded
- why
- how it becomes supported (or why it won't)

---

## What the supported lane covers

`ci-supported` currently checks the **core pipeline**:

- `cargo fmt --check`
- `cargo clippy` (core crates)
- `cargo test` (core crates)
- `glr-core` doctests with `serialization`

This lane is intentionally bounded so it stays reliable and fast enough for day-to-day work.

---

## What is excluded (and why)

### Not in the supported lane (workspace members / tools)
These are intentionally excluded for now because they are prototypes, platform-sensitive, heavier than the supported contract, or still stabilizing:

- `runtime2/` (alt runtime path; still converging)
- `cli/`, `lsp-generator/`, `playground/`, `wasm-demo/` (tooling/prototypes)
- `golden-tests/` (useful contract, but can be heavy and multi-language)
- `benchmarks/` (signal, not merge-blocking)
- `grammars/*` (valuable, but not yet a stable published surface)
- `crates/*` (BDD/governance microcrates; still stabilizing structure)

### Not in the supported lane (workflows)
These may run as optional signal (nightly/manual/canary), but are not required for merge:

- fuzzing lanes
- wide platform matrices
- deployment workflows (mdBook / pages)
- performance regression canaries

---

## How something graduates into the supported lane

To add a crate/workflow to the supported lane, it must be:
- reproducible on a normal dev machine
- stable across the supported toolchain/MSRV
- bounded in time/resources
- documented (how to run it locally; common failure modes)

When you add something to `ci-supported`, update this file in the same PR.
