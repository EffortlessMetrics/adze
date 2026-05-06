# Clippy Policy

Adze treats Clippy as a governed engineering surface rather than a local style file. The policy target is a single Effortless Metrics Rust platform posture: MSRV 1.93, panic-free production and tests, AST/parser-safe defaults, explicit suppressions, and reviewable temporary debt.

This first policy PR establishes the shared ledger and gate. It does not attempt to clean all existing panic-family, unsafe, indexing, or test helper debt in one change.

## Source of truth

The policy files live under `policy/`:

- `policy/clippy-lints.toml` records the active workspace lint baseline and the strict lints that will be promoted in stacked cleanup PRs.
- `policy/clippy-debt.toml` records temporary exceptions with owner, reason, path, lint, and expiry.
- `policy/no-panic-allowlist.toml` is reserved for semantic panic-family call-site exceptions using `path + family + selector` identity.
- `policy/non-rust-allowlist.toml` records non-Rust files that are intentional repository surfaces.

`Cargo.toml` remains the compiler-enforced lint configuration. `policy/clippy-lints.toml` is the machine-readable governance ledger that lets `xtask` check whether the enforced state and planned ratchets are coherent.

## Current rollout stage

The current stage is `governed-advisory`:

1. MSRV is ratcheted to Rust 1.93.0.
2. Existing workspace Rust lints stay active and gain a few low-risk compiler lints.
3. The full strict Clippy baseline is tracked as planned debt-backed policy.
4. `cargo xtask check-lint-policy` verifies the policy files, MSRV consistency, lint inheritance, lack of test carveouts, and debt expiry metadata.

Follow-up PRs should retire debt and promote planned lints from `policy/clippy-lints.toml` into `[workspace.lints.clippy]` once the affected code is clean.

## Suppression style

Use narrow `#[expect(..., reason = "...")]` suppressions when a lint is intentionally violated for a reviewed reason. Do not add broad `#[allow(...)]` suppressions or Clippy test carveouts.

Preferred pattern:

```rust
#[expect(
    clippy::indexing_slicing,
    reason = "generated parse table indexes are bounded by table construction invariant"
)]
fn generated_table_lookup(row: &[u16], index: usize) -> u16 {
    row[index]
}
```

Avoid:

```rust
#[allow(clippy::indexing_slicing)]
fn generated_table_lookup(row: &[u16], index: usize) -> u16 {
    row[index]
}
```

## No test carveouts

The workspace target is panic-free production and tests. Do not add these `clippy.toml` keys:

```toml
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-indexing-slicing-in-tests = true
allow-dbg-in-tests = true
```

Tests should migrate toward `Result`-returning helpers and explicit assertion/error utilities rather than `unwrap`, `expect`, or panic-driven setup.

## Planned Rust upgrades

The ledger tracks lints before the compiler bump that enables them:

- Rust 1.94: `same_length_and_capacity`, `manual_ilog2`, `decimal_bitwise_operands`, `needless_type_cast`.
- Rust 1.95: `disallowed_fields`, `manual_checked_ops`, `manual_take`, `manual_pop_if`, `duration_suboptimal_units`, `unnecessary_trailing_comma`.

`cargo xtask check-lint-policy` fails if a planned MSRV-gated lint is made active before the workspace MSRV reaches its activation version.

## Local debt rules

Debt is allowed only as structured, expiring policy data. Each `[[debt]]` entry must include:

- `lint`
- `path`
- `owner`
- `reason`
- `expires`

Expired debt fails the policy check. Silent debt is not allowed.
