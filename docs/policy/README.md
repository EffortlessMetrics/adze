# Adze policy stack

This directory documents the **policy stack** that governs lint rigor,
panic-family debt, and the non-Rust surface in the adze repository.

The principle is simple:

> **Deny by default. Allow by receipt. Expire exceptions. Measure drift.**

Five policy surfaces are governed here, each with one source-of-truth TOML
file under `/policy/`:

| Surface              | Policy file                          | Doc                       | Checker                                  |
| -------------------- | ------------------------------------ | ------------------------- | ---------------------------------------- |
| Clippy / rustc lints | `policy/clippy-lints.toml`           | [CLIPPY_POLICY.md]        | `cargo xtask check-lint-policy`          |
| Clippy debt          | `policy/clippy-debt.toml`            | [CLIPPY_POLICY.md]        | `cargo xtask check-lint-policy`          |
| Panic-family code    | `policy/no-panic-allowlist.toml`     | [NO_PANIC_POLICY.md]      | `cargo xtask check-no-panic-family`      |
| Non-Rust files       | `policy/non-rust-allowlist.toml`     | [FILE_POLICY.md]          | `cargo xtask check-file-policy`          |
| All of the above     | (aggregate)                          | [POLICY_ALLOWLISTS.md]    | `cargo xtask policy-report`              |

[CLIPPY_POLICY.md]: ./CLIPPY_POLICY.md
[NO_PANIC_POLICY.md]: ./NO_PANIC_POLICY.md
[FILE_POLICY.md]: ./FILE_POLICY.md
[POLICY_ALLOWLISTS.md]: ./POLICY_ALLOWLISTS.md

## Stage 1 — current

Adze is starting at **Stage 1** of the rollout:

1. The semantic checkers (`check-no-panic-family`, `check-file-policy`,
   `check-lint-policy`) are wired up and runnable.
2. Allowlists are mostly empty so the proposal flow can populate them
   without losing fidelity.
3. CI runs the checkers as **advisory** (`continue-on-error: true`). They
   produce reports under `target/policy/reports/` but do not block merges.
4. The shared Clippy block remains unchanged in `Cargo.toml`. The expanded
   profile lives in `policy/clippy-lints.toml` and is **planned**, not yet
   enforced.

This lets debt be made visible before it is made blocking.

## Stage progression

```
Stage 1 (today):
  Clippy panic lints = unchanged (warn / inactive)
  xtask checkers     = advisory
  exceptions         = TOML allowlist (sparse)

Stage 2:
  baselines populated and reviewed
  every entry has owner / reason / expiry
  checkers flip to required on protected branches

Stage 3:
  for families with zero exceptions, flip Clippy to deny
  for families with intentional exceptions, require both:
    - TOML allowlist entry
    - #[expect(..., reason = "policy:no-panic:<id>")] at the call site
```

See [CLIPPY_POLICY.md](./CLIPPY_POLICY.md#stage-progression) for the full
ladder, including planned 1.94 / 1.95 lint flips.

## Daily workflow

Most contributors only need three commands:

```bash
cargo xtask check-no-panic-family        # check panic-family debt
cargo xtask check-file-policy            # check non-Rust file surface
cargo xtask policy-report                # rolled-up report for review
```

If a checker reports a finding that should be allowed, do **not** add a
bare `#[allow(...)]`. Instead:

```bash
cargo xtask no-panic propose             # writes target/policy/reports/...
```

Review the proposal, copy the entries you intend to keep into
`policy/no-panic-allowlist.toml`, and fill in `owner`, `explanation`, and
`expires`. The receipt is the contract — not the suppression.
