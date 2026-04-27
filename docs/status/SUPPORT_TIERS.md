# Support Tiers and Proof Surface

**Last updated:** 2026-04-26
**Source of truth for:** README feature claims, `docs/status/KNOWN_RED.md`, and CI expectations.

This document maps major Adze surfaces to four tiers:

- **Stable** — part of the required support contract (`just ci-supported` / `CI / ci-supported`).
- **Experimental** — implemented, but not part of the required merge gate; behavior may change.
- **Advisory** — useful signal exists (optional CI lane, smoke, benchmark, etc.) but is non-blocking.
- **Intentionally excluded** — tracked in `KNOWN_RED`; not currently a merge requirement.

## Feature-to-proof map

| Surface | Tier | Proof command | CI lane | Notes / limitations |
|---|---|---|---|---|
| Typed extraction | **Stable** | `cargo test -p adze --lib --tests --bins` (via `just ci-supported`) | `CI / ci-supported` | Core user contract via `adze` runtime tests in supported lane. |
| Pure-Rust parser | **Stable** | `just ci-supported` | `CI / ci-supported` | Supported gate exercises pure-Rust core crates as the required contract. |
| GLR parsing | **Stable** | `cargo test -p adze-glr-core --lib --tests --bins` (via `just ci-supported`) | `CI / ci-supported` | GLR generation/runtime core is in required gate. |
| Serialization | **Stable** | `cargo test -p adze-glr-core --features serialization --doc` (via `just ci-supported`) | `CI / ci-supported` | Supported proof is currently `adze-glr-core` serialization doctests, not a full workspace serialization matrix. |
| External scanners | **Experimental** | `cargo test -p adze --features external_scanners` | `CI / feature-matrix`, `CI / miri` (non-blocking/optional) | Not in required gate; coverage exists but is broad-lane signal. |
| Incremental parsing | **Experimental** | `cargo test --workspace --features incremental_glr` | `CI / feature-matrix-extras` (non-blocking/optional) | Exists and tested in broad CI; still outside supported merge contract. |
| Tree-sitter interop | **Advisory** | `./scripts/smoke-link.sh ts-bridge` | `smoke-ts-bridge / smoke`, `ts-bridge-smoke` | Interop/bridge proof is smoke-level and optional; not part of required lane. |
| WASM | **Advisory** | `cargo check --target wasm32-unknown-unknown -p adze` (and core crates) | `CI / wasm-check`, `microcrate-ci / wasm-check` | Compile-check signal exists, but WASM is not in required branch-protection gate. |
| CLI (`cli/`) | **Intentionally excluded** | `cargo check -p adze-cli` | No required lane | Tooling/prototype surface; excluded from supported lane per `KNOWN_RED`. |
| runtime2 (`runtime2/`) | **Intentionally excluded** | `cargo check -p adze-runtime2` | No required lane | Alternate runtime path still converging; excluded from supported lane. |
| Grammars (`grammars/*`) | **Intentionally excluded** | `cargo test -p adze-grammar-python` (and peers) | No required lane | Valuable examples/integration surfaces, but not yet a stable supported contract. |
| Golden tests | **Advisory** | `cd golden-tests && cargo test --features <lang>-grammar -- --nocapture` | `golden-tests / golden-tests`, `pure-rust-ci / golden-tests` | High-value parity signal, but intentionally non-blocking for merges. |
| Benchmarks | **Advisory** | `cargo bench -p adze --bench glr_parser_bench --no-run` | `CI / benchmarks`, `benchmarks / benchmark`, `performance` | Performance signal only; never treated as merge-blocking proof of correctness. |

## How to use this file

- If you change support scope, update this file and `docs/status/KNOWN_RED.md` in the same PR.
- If README capability wording changes, ensure each claim maps to a row here.
- If a surface lacks a repeatable proof command and lane, it must not be labeled **Stable**.
