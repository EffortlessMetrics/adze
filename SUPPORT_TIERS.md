# Support tiers and proof surface

This document is the source of truth for what Adze supports today, how each
surface is proven, and where that proof runs in CI.

If a capability is not proven by a required lane, it must not be described as
stable.

## Tier definitions

- **Stable**: Proven by the required PR gate (`just ci-supported` / `CI / ci-supported`).
- **Experimental**: Implemented and useful, but proven only in optional lanes or partial checks.
- **Advisory**: Documented and/or available for local use, but intentionally excluded from merge-blocking proof.

## Feature-to-proof map

| Surface | Tier | Proof command (local) | CI lane | Notes / limitations |
|---|---|---|---|---|
| Typed extraction | Stable | `just ci-supported` | `CI / ci-supported` (required) | Covered by runtime tests in supported crates (`adze`). |
| Pure-Rust parser | Stable | `just ci-supported` | `CI / ci-supported` (required) | Default backend path is part of supported crate tests. |
| GLR parsing | Stable | `just ci-supported` | `CI / ci-supported` (required) | Covered by `adze` + `adze-glr-core` tests and clippy in supported lane. |
| Serialization | Experimental | `cargo test -p adze-glr-core --features serialization --doc` | `CI / ci-supported` (required, partial) | Required lane proves `adze-glr-core` serialization doctests; full end-to-end serialization behavior across non-supported surfaces is not a merge gate. |
| External scanners | Experimental | `cargo test -p adze --features external_scanners` | Optional feature-matrix jobs in `CI` | API and grammar usage exist, but this is not in the required gate. |
| Incremental parsing | Advisory | `cargo test -p adze --features incremental_glr` | Optional feature-matrix jobs in `CI` | Current path may fall back to fresh parse in complex cases; not merge-blocking. |
| Tree-sitter interop (`ts-bridge`) | Advisory | `cargo build -p ts-bridge --release` | `ts-bridge-smoke`, `ts-bridge-parity` (optional) | Tooling is useful but excluded from required lane and workspace-default proof. |
| WASM | Experimental | `cargo check --target wasm32-unknown-unknown -p adze -p adze-ir -p adze-glr-core -p adze-tablegen` | `CI / wasm-check` (optional) | Core crates are checked for wasm target in optional lane, not required gate. |
| CLI (`cli/`) | Advisory | `cargo check -p adze-cli` | Optional/non-required jobs only | Developer tooling surface; excluded from supported lane. |
| `runtime2/` | Advisory | `cargo test --manifest-path runtime2/Cargo.toml` | Optional/non-required jobs only | Alternate runtime path; explicitly excluded in Known Red until converged. |
| Grammars (`grammars/*`) | Advisory | `cargo test -p adze-python -p adze-javascript -p adze-go` | Optional/non-required jobs only | Reference/validation grammars; not currently a stable published merge-gated surface. |
| Golden tests | Advisory | `cargo test -p adze-golden-tests` | `golden-tests` workflow (optional) | Valuable compatibility contract, but intentionally excluded from required lane. |
| Benchmarks | Advisory | `cargo bench -p adze-glr-core --no-run && cargo bench -p adze --lib --no-run` | `benchmarks`, `performance`, `criterion-smoke` (optional) | Performance signal only; non-blocking by policy. |

## Policy

When changing support claims in `README.md` or release notes:

1. Update this table first.
2. Ensure the proof command is reproducible locally.
3. Promote a surface to **Stable** only after it is covered by the required lane.

Related docs:
- `README.md`
- `docs/status/KNOWN_RED.md`
