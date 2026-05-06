# Clippy policy

The shared lint baseline for the adze workspace lives in
`policy/clippy-lints.toml`. The root `Cargo.toml` carries a small `active`
subset; the rest are **planned** activations tracked alongside the MSRV at
which they become safe to flip.

This document is the human-readable counterpart.

## Identity

A lint entry is identified by:

```
identity = name + level + activate_when_msrv
```

`name` uses `prefix::lint`, where `prefix` is `rust` or `clippy`.

## Status of an entry

| Field             | Meaning                                                        |
| ----------------- | -------------------------------------------------------------- |
| `[[active]]`      | Currently set in `Cargo.toml` `[workspace.lints]`.             |
| `[[planned]]`     | Slated to activate; not yet in `Cargo.toml`.                   |
| `activate_when_msrv` | Earliest MSRV at which the lint compiles cleanly.            |
| `level`           | `deny` / `warn` / `allow`.                                     |
| `reason`          | Why the lint is here — what failure mode it catches.           |

`cargo xtask check-lint-policy` reads this file and verifies:

* MSRV in the file matches `rust-version` in the workspace `Cargo.toml`.
* Every workspace crate inherits via `[lints]` `workspace = true`.
* Each `active` entry actually appears in the workspace `[lints]` block.
* No `planned` entry is active before its `activate_when_msrv`.
* No `clippy.toml` carries a test carveout (`allow-*-in-tests`).
* No bare `#[allow(...)]` is committed; suppressions must use
  `#[expect(..., reason = "...")]`.
* Every `clippy-debt.toml` entry has an unexpired `expires`.

## Stage progression

```
Stage 1 (today):
  panic-family Clippy lints stay where they are
  semantic check-no-panic-family runs as advisory
  exceptions tracked in policy/no-panic-allowlist.toml

Stage 2:
  baseline allowlist filled and reviewed
  every entry has owner / reason / expiry
  semantic checker required on protected branches

Stage 3:
  for families with zero exceptions, flip Clippy to deny
  for families with intentional exceptions, require:
    - TOML allowlist entry, AND
    - #[expect(..., reason = "policy:no-panic:<id>")] at the call site
```

The dual-rail design (Clippy + semantic checker) deliberately keeps both
gates: Clippy gives fast IDE feedback close to the code, while the
semantic checker owns the durable receipt with owner, reason, selector,
expiry, and drift.

## What we do *not* set

* `allow-unwrap-in-tests = true`
* `allow-expect-in-tests = true`
* `allow-panic-in-tests = true`
* `allow-indexing-slicing-in-tests = true`
* `allow-dbg-in-tests = true`

Tests are production code for the purposes of panic-family debt. They
also have the same right to fallible helpers (see
[NO_PANIC_POLICY.md](./NO_PANIC_POLICY.md)).

## Suppression style

```rust
// Wrong — bare allow with no reason:
#[allow(clippy::unwrap_used)]
fn foo() { ... }

// Wrong — opaque reason:
#[expect(clippy::unwrap_used, reason = "fix later")]
fn foo() { ... }

// Right — receipted with policy ID:
#[expect(
    clippy::unwrap_used,
    reason = "policy:no-panic:panic-0042 — fixture setup; see policy/no-panic-allowlist.toml",
)]
fn foo() { ... }
```

The receipt ID format `policy:no-panic:<id>` lets readers (and the
checker) cross-reference the receipt.

## Planned 1.94 / 1.95 flips

The full list lives in `policy/clippy-lints.toml`. Highlights:

* `clippy::same_length_and_capacity` — catches raw-parts reconstruction.
* `clippy::manual_checked_ops` — prefers checked arithmetic.
* `clippy::manual_take` — prefers `mem::take` over manual reimplementation.
* `clippy::duration_suboptimal_units` — improves duration legibility.
* `clippy::unnecessary_trailing_comma` — keeps format-macro args clean.

These ride along automatically when the workspace MSRV reaches the
specified Rust release.
