# Clippy Policy

Adze treats Clippy as a governed engineering surface. The root `Cargo.toml`
contains the active workspace lint baseline, `policy/clippy-lints.toml` is the
machine-readable ledger for active and planned lint levels, and `cargo xtask
check-lint-policy` keeps those surfaces coherent.

## Goals

The standard policy is intentionally workspace-wide:

- panic-free production and test code;
- AST/parser/UTF-8/slice safety by default;
- no silently swallowed futures, locks, `Result`s, or errors;
- explicit suppression governance;
- reviewability lints that reduce allocation and control-flow noise; and
- planned Rust 1.94 / 1.95 lint flips tracked before an MSRV bump.

## Active policy surfaces

- `Cargo.toml` owns the active `[workspace.lints.rust]` and
  `[workspace.lints.clippy]` levels.
- `clippy.toml` is reserved for repo-specific `disallowed-*` policy and must
  not contain test carveouts.
- `policy/clippy-lints.toml` mirrors active lints and records planned flips.
- `policy/clippy-debt.toml` records temporary exceptions with an owner, reason,
  path, lint, and expiry.

## Suppressions

Prefer code changes over suppressions. If a suppression is unavoidable, use a
narrow `#[expect(..., reason = "...")]` at the smallest useful scope so the
compiler and Clippy can prove the exception is still needed.

Do not add broad `#[allow(...)]` attributes or Clippy test carveouts. Existing
legacy suppressions should be migrated in follow-up cleanup PRs and represented
as temporary debt while they are still present.

## Panic-family exceptions

The standard exception shape is semantic TOML in
`policy/no-panic-allowlist.toml`. Identity is:

```text
path + family + selector
```

`last_seen` line and column values are advisory hints only; they are not the
stable identity of an exception.

## Non-Rust file exceptions

Rust is the default implementation language for repo tooling. Non-Rust
programming, fixture, CI, or config surfaces belong in
`policy/non-rust-allowlist.toml` with the owner, reason, surface,
classification, and CI coverage that justify the exception.

## Upgrade ledger

Rust 1.94 and 1.95 lints are tracked as `status = "planned"` entries in
`policy/clippy-lints.toml`. Planned lints must not be activated before their
`activate_when_msrv` unless the ledger and MSRV are updated together.

## Gate

Run:

```bash
cargo xtask check-lint-policy
```

The gate verifies the MSRV ledger, root lint levels, planned lint flips,
`clippy.toml` carveouts, and debt metadata. As the rollout advances, follow-up
PRs should tighten inheritance and suppression checks after the current debt is
explicitly categorized.
