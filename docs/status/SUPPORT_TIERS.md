# Support Tiers and Proof Surface

**Last updated:** 2026-04-26  
**Source of truth for:** feature support tier, proof command, and CI lane coverage.

This document aligns README capability claims with the currently enforced proof surface.

## Tier definitions

- **Stable**: Included in the required PR gate (`just ci-supported`) with repeatable proof.
- **Experimental**: Implemented and testable, but **not** in the required PR gate.
- **Advisory**: Exists and may have signal CI/workflows, but no merge-blocking proof contract.
- **Intentionally excluded**: Explicitly out of the supported lane today (see `KNOWN_RED`).

## Feature support map

| Feature surface | Tier | Proof command (local) | CI lane | Notes / limitations |
|---|---|---|---|---|
| Typed extraction | **Stable** | `cargo test -p adze --lib --tests --bins` | `CI / ci-supported` | Core user promise; enforced through runtime crate tests in required gate. |
| Pure-Rust parser (default backend) | **Stable** | `cargo test -p adze --lib --tests --bins` | `CI / ci-supported` | Required lane exercises default runtime path in `adze`; broader workspace permutations are outside required gate. |
| GLR parsing | **Stable** | `cargo test -p adze-glr-core --lib --tests --bins` | `CI / ci-supported` | GLR algorithm/tablegen core is in supported 7-crate lane. |
| Serialization | **Stable** (core scope) | `cargo test -p adze-glr-core --features serialization --doc` | `CI / ci-supported` | Required gate currently proves serialization via `adze-glr-core` doctests, not every serialization path in excluded crates. |
| External scanners | **Experimental** | `cargo nextest run --workspace --features external_scanners` | `CI / Test (...)` in `.github/workflows/ci.yml` (non-required on PRs) | Signal exists, but not merge-blocking and not in `ci-supported`. |
| Incremental parsing | **Experimental** | `cargo nextest run --workspace --features incremental_glr` | `CI / Test (...)` in `.github/workflows/ci.yml` (non-required on PRs) | Signal coverage exists; required gate does not enforce it. |
| Tree-sitter interop (`ts-bridge`, parity/smoke lanes) | **Advisory** | `just smoke` (ts-bridge link smoke) | `ts-bridge-smoke`, `ts-bridge-parity` (optional workflows) | Platform/toolchain sensitive; excluded from required supported lane. |
| WASM | **Advisory** | `cargo check --target wasm32-unknown-unknown -p adze --no-default-features` | `microcrate-ci / WASM Build (core crates)` and `pure-rust-ci / Test WASM Build` (non-required) | Useful signal but not required PR gate proof. |
| CLI (`cli/`) | **Intentionally excluded** | `cargo check -p adze-cli` | No required lane | Tooling surface is listed as excluded in `KNOWN_RED`; not currently part of support contract. |
| `runtime2/` | **Intentionally excluded** | `cargo check -p adze-runtime2` | `performance.yml` and other optional lanes | Alternate runtime path is still converging; explicitly excluded from supported lane. |
| `grammars/*` | **Intentionally excluded** | `cargo test -p adze-grammar-python` (example per-grammar) | Optional/non-required workflows only | Valuable reference implementations, but not yet stable published contract. |
| Golden tests (`golden-tests/`) | **Advisory** | `cd golden-tests && cargo test -- --test-threads=2` | `golden-tests.yml` (optional) | Useful parity signal, not required for merge because of runtime/cross-language weight. |
| Benchmarks (`benchmarks/`) | **Advisory** | `cargo bench -p adze-benchmarks --no-run` | `performance.yml`, `criterion-smoke.yml` (optional) | Performance signal only; intentionally non-blocking today. |

## Scope guardrail

If a feature is advertised as stable in README, it should have a matching **Stable** row here and proof in `ci-supported`.  
If not, mark it Experimental/Advisory/Excluded here first, then tighten README wording as needed.
