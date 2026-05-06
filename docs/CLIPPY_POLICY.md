# Clippy Policy

Adze treats Clippy as a governed engineering surface, not as a local taste file.
The root workspace owns the active lint levels, `policy/clippy-lints.toml` records
why each active lint exists and which future lints are planned, and `cargo xtask
check-lint-policy` verifies that the manifest, policy ledger, and suppression
posture stay coherent.

## Active baseline

The active workspace baseline is in the root `Cargo.toml` under
`[workspace.lints.rust]` and `[workspace.lints.clippy]`. This first policy PR
lands the shared surface and advisory inheritance report; follow-up PRs should
ratchet workspace packages into enforcement with:

```toml
[lints]
workspace = true
```

The baseline is intentionally workspace-wide: production code, examples, and tests
all follow the same panic-free and silent-failure rules. In particular, do not add
Clippy test carveouts such as `allow-unwrap-in-tests`, `allow-expect-in-tests`,
`allow-panic-in-tests`, `allow-indexing-slicing-in-tests`, or
`allow-dbg-in-tests`.

## Suppression style

Use narrow `#[expect(..., reason = "...")]` attributes when a lint must be
suppressed at a specific site. Do not use broad `#[allow(...)]` attributes unless
a future policy file explicitly permits the exception.

```rust
#[expect(
    clippy::indexing_slicing,
    reason = "generated parse table is bounded by tablegen invariants"
)]
fn lookup_generated_table(row: &[u16], idx: usize) -> u16 {
    row[idx]
}
```

Suppression debt belongs in `policy/clippy-debt.toml` when it is broader than one
reviewed call site. Debt entries must include the lint, path, owner, reason, and
expiry date so the exception remains temporary and attributable.

## Policy ledger

`policy/clippy-lints.toml` is the machine-readable source of truth for:

- active Rust and Clippy lints;
- the expected MSRV for the lint profile;
- panic-free test posture;
- suppression style;
- planned Rust 1.94 and 1.95 lint flips.

Planned lints stay in the ledger before the MSRV bump. They must not be activated
in `Cargo.toml` until `activate_when_msrv` is reached.

## Repo-local `clippy.toml`

`clippy.toml` is intentionally not a second lint-level file. It is reserved for
Clippy configuration that cannot be expressed in `[workspace.lints]`, such as
future `disallowed-methods`, `disallowed-types`, or parser-specific policy data.
It must not weaken the workspace posture or add test carveouts.

## Checks

Run the policy gate with:

```bash
cargo xtask check-lint-policy
cargo xtask check-no-panic-family
cargo xtask check-file-policy
cargo xtask policy-report
```

The check verifies that the workspace MSRV matches the policy ledger, reports
which workspace members still need lint inheritance, checks that active lints
match `Cargo.toml`, ensures planned 1.94/1.95 lints are not active early, rejects
`clippy.toml` test carveouts, and validates that lint debt is complete and
unexpired. Set `ADZE_ENFORCE_LINT_INHERITANCE=1` in a follow-up ratchet to make
missing `[lints] workspace = true` entries fail.
