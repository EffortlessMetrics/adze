# Support Tiers and Proof Surface

**Last updated:** 2026-05-06
**Source of truth for:** README feature claims, `docs/status/KNOWN_RED.md`, and CI expectations.

This document maps major Adze surfaces to five tiers:

- **Stable** — part of the required support contract (`just ci-supported` / `CI / ci-supported`).
- **Stabilizing** — implemented and tested, but missing one or more product-level proofs before it should be marketed as a stable user contract.
- **Experimental** — implemented, but not part of the required merge gate; behavior may change.
- **Advisory** — useful signal exists (optional CI lane, smoke, benchmark, etc.) but is non-blocking.
- **Intentionally excluded** — tracked in `KNOWN_RED`; not currently a merge requirement.

## Feature-to-proof map

| Surface | Tier | Proof command | CI lane | Notes / limitations |
|---|---|---|---|---|
| Typed extraction | **Stable** | `cargo test -p adze --lib --tests --bins` (via `just ci-supported`) | `CI / ci-supported` | Core user contract via `adze` runtime tests in supported lane. |
| Pure-Rust parser | **Stable** | `just ci-supported` | `CI / ci-supported` | Supported gate exercises pure-Rust core crates as the required contract. |
| Operator precedence | **Stable** | `cargo test -p adze --lib --tests --bins`; `cargo test -p adze-glr-core --lib --tests --bins` (via `just ci-supported`) | `CI / ci-supported` | Stable for proven expression grammar shapes. |
| GLR conflict routing | **Stabilizing** | `cargo test -p adze-glr-core conflict ambiguity -- --nocapture`; target: `cargo test -p adze --features "pure-rust,glr,runtime-e2e" glr_ -- --nocapture` | `CI / ci-supported` plus future product lane | GLR core is in the required gate, but full product proof still needs conflict-preserving end-to-end typed extraction and ambiguity metadata/selection coverage. |
| Tablegen `TSLanguage` ABI | **Stabilizing** | `cargo test -p adze-tablegen --all-features`; target: `cargo test -p adze --features "pure-rust,glr,ts-compat" --test ts_compat_equiv -- --nocapture` | `CI / ci-supported` plus future product lane | Symbol/state invariants are core-gated; conflict encoding/routing and real field maps need stronger ABI proof before broader Tree-sitter compatibility is claimed. |
| Structured parse errors | **Stabilizing** | `cargo test -p adze --features "pure-rust,glr" parse_error -- --nocapture` | Future product lane | Error paths exist, but the stable contract needs spans, expected sets, line/column, excerpts, and no-panic coverage across LR and GLR. |
| Serialization | **Stable for core table serialization** | `cargo test -p adze-glr-core --features serialization --doc` (via `just ci-supported`) | `CI / ci-supported` | Supported proof is currently `adze-glr-core` serialization doctests, not a full workspace serialization matrix or generated-AST persistence contract. |
| External scanners | **Experimental** | `cargo test -p adze --features external_scanners` | `CI / feature-matrix`, `CI / miri` (non-blocking/optional) | Not in required gate; coverage exists but is broad-lane signal. |
| Incremental parsing | **Experimental** | `cargo test --workspace --features incremental_glr` | `CI / feature-matrix-extras` (non-blocking/optional) | Exists and tested in broad CI; still outside supported merge contract. |
| Tree-sitter interop | **Advisory** | `./scripts/smoke-link.sh ts-bridge` | `smoke-ts-bridge / smoke`, `ts-bridge-smoke` | Interop/bridge proof is smoke-level and optional; not part of required lane. |
| WASM | **Advisory** | `cargo check --target wasm32-unknown-unknown -p adze` (and core crates) | `CI / wasm-check`, `microcrate-ci / wasm-check` | Compile-check signal exists, but WASM is not in required branch-protection gate. |
| CLI (`cli/`) | **Advisory** | `cargo test -p adze-cli -- --nocapture` | Product-proof advisory lane target | CLI is outside `ci-supported`; supported commands must be behavior-tested and unsupported modes must fail explicitly before promotion. |
| runtime2 (`runtime2/`) | **Intentionally excluded** | `cargo test --manifest-path runtime2/Cargo.toml language_smoke_exposes_metadata_queries -- --nocapture` | Product-proof advisory lane target | Experimental proving ground; not the public-primary runtime contract. |
| Grammars (`grammars/*`) | **Advisory** | `cargo test -p adze-python -- --nocapture` and peer grammar smokes | Product-proof advisory lane target | Valuable examples/integration surfaces, but not yet a stable published support contract. |
| Golden tests | **Advisory** | `cd golden-tests && cargo test --features <lang>-grammar -- --nocapture` | `golden-tests / golden-tests`, `pure-rust-ci / golden-tests` | High-value parity signal, but intentionally non-blocking for merges. |
| Benchmarks | **Advisory** | `cargo bench -p adze --bench glr_parser_bench --no-run` | `CI / benchmarks`, `benchmarks / benchmark`, `performance` | Performance signal only; never treated as merge-blocking proof of correctness. |

## How to use this file

- If you change support scope, update this file and `docs/status/KNOWN_RED.md` in the same PR.
- If README capability wording changes, ensure each claim maps to a row here.
- If a surface lacks a repeatable proof command and lane, it must not be labeled **Stable**.
- Use `docs/status/CORRECTNESS_PUSH.md` for the current merge/proof sequence.
