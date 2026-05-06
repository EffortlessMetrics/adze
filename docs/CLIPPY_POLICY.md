# Effortless Metrics Clippy Policy

Adze treats Clippy as a governed engineering surface, not as a local taste file.
The root workspace owns the strict lint baseline, `policy/clippy-lints.toml`
records the active and planned lint state, and temporary exceptions must be
represented as structured policy data instead of broad configuration carveouts.

## Baseline

The workspace baseline is panic-free, parser-safe, suppression-governed, and
reviewability-oriented:

- no panic-family shortcuts in production or tests (`unwrap`, `expect`,
  `panic!`, `todo!`, `unimplemented!`, `unreachable!`, and related lints);
- no silent failure or swallowed work (`let _` futures/must-use values,
  ignored `map_err`, ignored `Result::ok`, and line iteration footguns);
- AST/parser-safe string, byte, and index defaults;
- async, locking, memory, numeric, filesystem, and process footguns called out
  explicitly; and
- no broad suppression culture.

The policy is intentionally workspace-wide: tests are part of the quality
surface. Do not add Clippy test carveouts such as `allow-unwrap-in-tests`,
`allow-expect-in-tests`, `allow-panic-in-tests`, `allow-indexing-slicing-in-tests`,
or `allow-dbg-in-tests` to `clippy.toml`.

## Source of truth

- `Cargo.toml` contains the active workspace lint block.
- `clippy.toml` is reserved for repo-specific Clippy configuration such as
  disallowed methods or disallowed types; it must not contain test carveouts.
- `policy/clippy-lints.toml` is the machine-readable ledger for active lints and
  planned Rust 1.94/1.95 flips.
- `policy/clippy-debt.toml` records temporary lint debt with owner, path, lint,
  reason, and expiry.
- `policy/no-panic-allowlist.toml` records semantic panic-family exceptions when
  a follow-up migration needs a temporary receipt.
- `policy/non-rust-allowlist.toml` records why non-Rust source surfaces exist and
  what covers them in CI.

## Suppressions

Use narrow `#[expect(..., reason = "...")]` suppressions only when a lint is
wrong for a local, reviewed reason. Avoid `#[allow(...)]` for new code. An
`expect` should be placed as close as possible to the reviewed expression or item
and the reason must explain why the exception is safe, not merely that Clippy is
noisy.

```rust
#[expect(
    clippy::indexing_slicing,
    reason = "generated parse table bounds are validated during table construction"
)]
fn generated_table_lookup(table: &[u16], index: usize) -> u16 {
    table[index]
}
```

## Planned Rust upgrades

Adze tracks future lint flips before the MSRV bump. The policy ledger currently
records Rust 1.94 candidates (`same_length_and_capacity`, `manual_ilog2`,
`decimal_bitwise_operands`, `needless_type_cast`) and Rust 1.95 candidates
(`disallowed_fields`, `manual_checked_ops`, `manual_take`, `manual_pop_if`,
`duration_suboptimal_units`, `unnecessary_trailing_comma`). Planned lints must
remain in the ledger until the workspace MSRV reaches their activation version.

## Local checks

Run the policy check after editing the lint block, Clippy configuration, or
policy ledgers:

```bash
cargo xtask check-lint-policy
```

The check verifies the MSRV ratchet, required policy files, root lint block,
planned upgrade ledger, debt metadata, and the absence of Clippy test carveouts.
As strict lint inheritance is rolled out crate-by-crate, temporary exceptions
must move into policy debt rather than weakening the root lint block.
