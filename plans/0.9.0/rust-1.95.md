# 0.9 Rust 1.95 MSRV Bump

Status: complete
Owner: release/toolchain
Created: 2026-05-14
Linked proposal: ../../docs/proposals/ADZE-PROP-0001-0.9-contract-convergence.md
Linked plan: ./implementation-plan.md
Active goal: ../../.adze/goals/active.toml

## Goal

Bump the workspace MSRV and pinned toolchain from Rust 1.92.0 to Rust 1.95.0
after the microcrate-to-SRP package collapse reduced the release surface.

## Production Delta

- `rust-toolchain.toml` pins `1.95.0`.
- `[workspace.package].rust-version` is `1.95.0`.
- `policy/clippy-lints.toml` records policy MSRV `1.95`.
- `xtask doctor` checks Rust 1.95.0.
- CI jobs with fixed MSRV toolchains use Rust 1.95.0.
- User/operator docs advertise Rust 1.95.0.

## Non-Goals

- No Clippy lint promotion in this PR.
- No parser/runtime behavior changes.
- No support-tier product claim promotion.

## Acceptance

All workspace MSRV declarations agree. CI uses the new pinned MSRV where a
fixed toolchain is required. The Clippy policy refresh becomes the next ready
work item.

## Proof Commands

```bash
cargo metadata --format-version 1 --no-deps
cargo run -q -p xtask -- check-lint-policy
cargo test -p xtask doctor -j 1 -- --nocapture
cargo test -p xtask lint_policy -j 1 -- --nocapture
cargo test -p adze-parsetable-metadata -- --test-threads=2
cargo clippy -p adze -p adze-macro -p adze-tool -p adze-common -p adze-ir -p adze-glr-core -p adze-tablegen --all-targets -- -D warnings
cargo run -q -p xtask -- check-package-boundary --release-gate
cargo fmt -p adze -- --check
cargo fmt -p adze-tool -- --check
cargo fmt -p adze-ir -- --check
cargo fmt -p adze-glr-core -- --check
cargo fmt -p adze-macro -- --check
cargo fmt -p adze-tablegen -- --check
cargo fmt -p xtask -p adze-parsetable-metadata -- --check
git diff --check
```

The hosted required gate remains:

```bash
just ci-supported
```

## Rollback

Revert the MSRV/toolchain PR and restore all `1.92.0` / `1.92` policy
references.
