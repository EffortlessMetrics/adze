# Now / Next / Later

**Last updated:** 2026-03-04

Adze status and rolling execution plan. For paper cuts and pain points, see [`docs/status/FRICTION_LOG.md`](./FRICTION_LOG.md).

---

## Now

### ✅ CI Gate Green (Complete)
- [x] All supported crates compile: `adze`, `adze-ir`, `adze-glr-core`, `adze-tablegen`, `adze-common`, `adze-macro`, `adze-tool`.
- [x] `cargo check --workspace` passes (full workspace compiles).
- [x] `cargo fmt --all -- --check` passes.
- [x] `cargo clippy` clean on supported crates (only Cargo.toml manifest warnings remain from `lsp-generator` and `wasm-demo`).
- [x] `cargo test` passes on supported crates.
- [x] `cargo doc` builds for supported crates (one `rustdoc::private_intra_doc_links` warning in `adze`).

### ✅ Safety Audit (Complete)
- [x] SAFETY comments on all `unsafe` blocks in `runtime/src/lex/`, `runtime/src/parser.rs`, `runtime/src/ffi.rs`, `runtime/src/decoder.rs`.
- [x] SAFETY comments on all `unsafe` blocks in `glr-core` and `tablegen`.

### ✅ Testing Buildout (Complete)
- [x] Property-based tests in `tablegen/tests/property_tests.rs`.
- [x] Integration tests in `runtime/tests/` (30+ test files covering API contracts, end-to-end, edge cases, concurrency).
- [x] Integration tests in `common/tests/` (`expansion_tests.rs`, `parsing_tests.rs`).
- [x] Snapshot tests in `ir/tests/` (10+ snapshots via `insta` for optimizer, normalizer, validator, JSON roundtrip).
- [x] Snapshot tests in `example/src/` (arithmetic, optionals, repetitions, words grammars).
- [x] GLR-core integration tests (20+ test files: conflict preservation, driver correctness, stack invariants, etc).

### ✅ API Documentation (Complete)
- [x] Crate-level `//!` doc comments on all supported crates.
- [x] `cargo doc` builds cleanly for `adze-ir`, `adze-glr-core`, `adze-tablegen`, `adze-common`.
- [x] Doctests pass for `glr-core` (serialization) and `ir` (builder).

### ✅ Infrastructure (Complete)
- [x] Fuzzing targets set up (20 targets in `fuzz/fuzz_targets/`).
- [x] CI workflow with feature matrix job for crate × feature-flag combinations.
- [x] Cargo.toml metadata fixed for publish readiness across workspace.
- [x] READMEs added to `crates/` microcrates.
- [x] Concurrency caps in CI (RUST_TEST_THREADS=2, RAYON_NUM_THREADS=4).

### ✅ Workspace Polish (Complete)
- [x] Cargo.toml metadata polish across workspace crates.
- [x] Core pure-Rust pipeline compiles cleanly: `adze-ir`, `adze-glr-core`, `adze-tablegen`.
- [x] 47 microcrates in `crates/` with stable structure.
- [x] Benchmarks, fuzzing, golden-tests, and book scaffolding in place.

### ✅ Documentation Sync (Complete)
- [x] Rework [`ARCHITECTURE.md`](../explanations/architecture.md) with Mermaid and Governance details.
- [x] Update [`GETTING_STARTED.md`](../tutorials/getting-started.md) and [`GRAMMAR_EXAMPLES.md`](../reference/grammar-examples.md) for 0.8.0.
- [x] Sync [`DEVELOPER_GUIDE.md`](../DEVELOPER_GUIDE.md) with `just` and `xtask` workflows.
- [x] Update [`ROADMAP.md`](../../ROADMAP.md) and [`KNOWN_LIMITATIONS.md`](../reference/known-limitations.md).
- [ ] Close remaining release blockers in doc history/version drift (`FR-001`): version strings and legacy naming in advanced how-to guides.

---

## Next

### 📦 Publishable Baseline
- [ ] Perform a clean `cargo package` dry-run for all core crates.
- [ ] Standardize feature-flag names across the workspace (`glr`, `simd`, etc).
- [ ] Resolve the one `rustdoc::private_intra_doc_links` warning in `adze`.

### 🛠️ CLI Refinement
- [ ] Implement `adze check` for static grammar validation.
- [ ] Implement `adze stats` for parse table metrics (states, symbols, conflicts).

---

## Later

### 🌳 Incremental Parsing
- Move from conservative fallback to active forest-splicing for massive performance gains in editors.
- Currently disabled and falls back to fresh parsing (see `glr_incremental.rs`).

### 🔍 Query Completion
- Implement remaining Tree-sitter query predicates (`#any-of?`, etc) and provide a cookbook.

### 🌐 Playground & LSP
- Stabilize the LSP generator so it can be used to generate production-grade language servers.
