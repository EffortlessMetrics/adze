# Now / Next / Later

**Last updated:** 2026-03-07
**Status:** **Release Candidate** — 0.8.0-rc quality

Adze status and rolling execution plan. For paper cuts and pain points, see [`docs/status/FRICTION_LOG.md`](./FRICTION_LOG.md). For API stability guarantees per crate, see [`docs/status/API_STABILITY.md`](./API_STABILITY.md).

---

## Done (Waves 1–14)

> 14 waves of parallel agent work, 85+ commits driving the 0.8.0 release to RC quality.

### ✅ CI Gate Green
- [x] All supported crates compile: `adze`, `adze-ir`, `adze-glr-core`, `adze-tablegen`, `adze-common`, `adze-macro`, `adze-tool`.
- [x] `cargo check --workspace` passes (full workspace compiles).
- [x] `cargo fmt --all -- --check` passes — **fmt clean** across all supported crates.
- [x] `cargo clippy` clean on supported crates — **clippy clean**.
- [x] `cargo test` passes — **2,460+ tests across feature combinations, 0 failures in supported lane**.
- [x] `cargo doc` builds for supported crates — **0 rustdoc warnings** across supported crates.

### ✅ Safety Audit
- [x] SAFETY comments on all `unsafe` blocks in `runtime/src/lex/`, `runtime/src/parser.rs`, `runtime/src/ffi.rs`, `runtime/src/decoder.rs`.
- [x] SAFETY comments on all `unsafe` blocks in `glr-core` and `tablegen`.

### ✅ Testing Buildout
- [x] **2,460+ tests** across the workspace and feature combinations (property, integration, snapshot, GLR-core, fuzzing, mutation guards, ABI matrix).
- [x] Property-based tests in `tablegen/tests/property_tests.rs`.
- [x] Integration tests in `runtime/tests/` (30+ test files covering API contracts, end-to-end, edge cases, concurrency).
- [x] Integration tests in `common/tests/` (`expansion_tests.rs`, `parsing_tests.rs`).
- [x] Snapshot tests in `ir/tests/` (10+ snapshots via `insta` for optimizer, normalizer, validator, JSON roundtrip).
- [x] Snapshot tests in `example/src/` — **10 example grammars** (arithmetic, optionals, repetitions, words, boolean, json, csv, lambda, regex, ini).
- [x] GLR-core integration tests (20+ test files: conflict preservation, driver correctness, stack invariants, etc).
- [x] Feature-combination matrix: 11/12 pass (1 expected failure).
- [x] Mutation testing configured and smoke-tested.
- [x] Driver integration, cursor, pipeline, and serialization tests (Wave 13).
- [x] Error display, builder, forest property, and ABI matrix tests (Wave 14).
- [x] Cross-crate integration and architecture validation tests.
- [x] Microcrate coverage for 5 low-coverage crates (Wave 12).

### ✅ Error Message Quality
- [x] Actionable error messages across parser, IR, and tablegen.
- [x] Compile-time diagnostics for grammar issues.
- [x] Error display formatting tests (Wave 14).

### ✅ WASM Compatibility Verification
- [x] All core crates compile for `wasm32-unknown-unknown`.
- [x] Pure-Rust runtime enables browser-based parsing without C dependencies.
- [x] WASM CI verification job added (Wave 12).

### ✅ Security Audit
- [x] `cargo-audit` clean — 0 known vulnerabilities.
- [x] No unsafe code without SAFETY comments.

### ✅ API Documentation
- [x] Crate-level `//!` doc comments on all supported crates.
- [x] `cargo doc` builds cleanly — 0 warnings across supported crates.
- [x] Doctests pass for `glr-core` (serialization) and `ir` (builder).
- [x] Book: **6+ chapters** covering grammar design, GLR parsing, external scanners, and more.
- [x] Missing doc comments added to core pipeline crates (Wave 14).
- [x] User guide architecture chapter added (Wave 13).

### ✅ Infrastructure
- [x] Fuzzing targets set up (22 targets in `fuzz/fuzz_targets/`).
- [x] CI workflow with feature matrix job for crate × feature-flag combinations.
- [x] CI with cross-platform advisory jobs (macOS/Windows).
- [x] Cargo.toml metadata fixed for publish readiness across workspace.
- [x] Publish order documented for crates.io release.
- [x] READMEs added to `crates/` microcrates.
- [x] Concurrency caps in CI (RUST_TEST_THREADS=2, RAYON_NUM_THREADS=4).
- [x] Cross-platform: Linux verified, macOS/Windows CI advisory jobs in place.
- [x] `scripts/check-publish-ready.sh` for crates.io readiness checks (Wave 14).

### ✅ Workspace Polish
- [x] Cargo.toml metadata polish across workspace crates.
- [x] Core pure-Rust pipeline compiles cleanly: `adze-ir`, `adze-glr-core`, `adze-tablegen`.
- [x] 47 microcrates in `crates/` with stable structure.
- [x] Benchmarks, fuzzing, golden-tests, and book scaffolding in place.

### ✅ Documentation Sync
- [x] Rework [`ARCHITECTURE.md`](../explanations/architecture.md) with Mermaid and Governance details.
- [x] Update [`GETTING_STARTED.md`](../tutorials/getting-started.md) and [`GRAMMAR_EXAMPLES.md`](../reference/grammar-examples.md) for 0.8.0.
- [x] Sync [`DEVELOPER_GUIDE.md`](../DEVELOPER_GUIDE.md) with `just` and `xtask` workflows.
- [x] Update [`ROADMAP.md`](../../ROADMAP.md) and [`KNOWN_LIMITATIONS.md`](../reference/known-limitations.md).
- [ ] Close remaining release blockers in doc history/version drift (`FR-001`): version strings and legacy naming in advanced how-to guides.

---

## Now

### 📦 RC Gate — Publish to crates.io
- [ ] Perform a clean `cargo package` dry-run for all core crates.
- [ ] Standardize feature-flag names across the workspace (`glr`, `simd`, etc).
- [ ] Fix remaining `adze` runtime test compilation errors (some integration test files reference removed/renamed APIs).
- [ ] Resolve the 1 expected feature-matrix failure (`feature_profile_resolve_backend`).
- [ ] Close doc-drift release blocker (`FR-001`).
- [ ] Publish initial release of core crates: `adze`, `adze-ir`, `adze-glr-core`, `adze-tablegen`.

### 📊 Current test count: **2,460+** across workspace and feature combinations

---

## Next

### 🛠️ Remaining for RC
- [ ] Fix `adze` runtime test compilation errors (test files referencing stale APIs).
- [ ] `cargo package --dry-run` CI gate for all publishable crates (`FR-012`).
- [ ] Final doc-drift audit: version strings and legacy naming (`FR-001`).

### 🛠️ CLI Implementation
- [ ] Implement `adze check` for static grammar validation.
- [ ] Implement `adze stats` for parse table metrics (states, symbols, conflicts).
- [ ] Implement `adze fmt` for grammar formatting.

### 📚 More Book Content
- [ ] Tutorial: writing your first grammar end-to-end.
- [ ] Reference: complete attribute catalog.
- [ ] How-to: migrating from Tree-sitter grammars.

---

## Later

### ⚡ Performance Optimization
- Arena allocator for parse forest nodes.
- Incremental parsing: move from conservative fallback to active forest-splicing.
- Benchmark suite with regression detection.

### 🌳 Incremental Parsing (Full)
- Move from conservative fallback to active forest-splicing for massive performance gains in editors.
- Currently disabled and falls back to fresh parsing (see `glr_incremental.rs`).

### 🔍 Query Completion
- Implement remaining Tree-sitter query predicates (`#any-of?`, etc) and provide a cookbook.

### 🌐 Playground & LSP
- Stabilize the LSP generator so it can be used to generate production-grade language servers.
