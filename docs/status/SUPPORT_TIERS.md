# Support Tiers and Proof Surface

**Last updated:** 2026-04-26

This is the source-of-truth mapping between advertised features and what is currently proven in CI.

## Tier definitions

- **Stable**: Proven by the required PR gate (`just ci-supported` / `CI / ci-supported`).
- **Experimental**: Has automated proof in CI, but outside the required PR gate.
- **Advisory**: Exists and can be checked manually (or in optional lanes), but is not currently part of a reliable merge-blocking contract.
- **Intentionally excluded**: Explicitly outside the supported lane until promoted.

## Feature-to-proof matrix

| Surface | Tier | Proof command | CI lane | Notes / limitations |
|---|---|---|---|---|
| Typed extraction | **Stable** | `cargo test -p adze --lib --tests --bins` (via `just ci-supported`) | `CI / ci-supported` | User-facing core contract in the supported lane. |
| Pure-Rust parser | **Stable** | `cargo test -p adze --lib --tests --bins` (default features) | `CI / ci-supported` | Default backend in supported crates only. |
| GLR parsing | **Stable** | `cargo test -p adze-glr-core --lib --tests --bins` + `cargo test -p adze` (via `just ci-supported`) | `CI / ci-supported` | Core GLR crates are in the required gate. |
| Serialization | **Stable** (core) | `cargo test -p adze-glr-core --features serialization --doc` | `CI / ci-supported` | Required-gate proof is currently `adze-glr-core` doctests with `serialization`; broader serialization surface remains outside required gate. |
| External scanners | **Experimental** | `cargo nextest run --workspace --features external_scanners` | `CI / Test (... / --features external_scanners / ...)` | Automated, but not in required PR gate; lane only runs in non-supported/full CI paths. |
| Incremental parsing | **Experimental** | `cargo nextest run --workspace --features incremental_glr` | `CI / Test (... / --features incremental_glr / ...)` | Automated, but not in required PR gate; fallback behavior and broader runtime integration still evolving. |
| Tree-sitter interop | **Experimental** | `cargo test -p adze --features "ts-compat pure-rust" --test ts_compat_equiv` | `CI / Tree-sitter Compatibility API` | Feature-specific CI exists, but not required for merge. |
| WASM | **Experimental** | `cargo check --target wasm32-unknown-unknown -p adze-ir -p adze-glr-core -p adze-tablegen -p adze-common` | `CI / WASM Build Verification` | Lane is optional (`continue-on-error`) and not in required gate; proof is compile-focused. |
| CLI (`cli/`) | **Advisory** | `cargo run -p adze-cli -- --help` | *(none required)* | Tooling surface exists but is intentionally outside supported lane today. |
| runtime2 (`runtime2/`) | **Advisory** | `cargo test -p adze-runtime --features "test-utils,glr"` | *(none required)* | Alternate runtime path; explicitly excluded from supported lane in `KNOWN_RED`. |
| Grammars (`grammars/*`) | **Advisory** | `cargo test -p adze-python` / `cargo test -p adze-javascript` / `cargo test -p adze-go` | Optional broad CI (`CI / Test` workspace lanes) | Valuable validation surface, but not a stable published support contract yet. |
| Golden tests | **Advisory** | `cargo test -p adze --test golden_tests -- --ignored` (from `runtime/`) | `Pure Rust Implementation CI / Golden Tests` | Requires Tree-sitter CLI/tooling; useful parity signal, intentionally outside required gate. |
| Benchmarks | **Advisory** | `cargo bench -p adze-benchmarks --bench incremental_bench --no-run` | `CI / Benchmark Compilation`, `benchmarks.yml` | Signal for performance only; not merge-blocking correctness proof. |

## Relationship to `KNOWN_RED`

Surfaces marked **Advisory** here should also appear in [`docs/status/KNOWN_RED.md`](./KNOWN_RED.md) until they are promoted into the supported lane.
