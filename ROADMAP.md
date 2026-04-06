# Adze Roadmap

**Current Version:** 0.8.0-dev
**MSRV:** 1.92 (Rust 2024 edition)

Adze (formerly `rust-sitter`) is a Rust-native grammar toolchain that turns Rust type definitions into high-performance GLR parse machinery.

---

## ✅ Milestone 0.6.0: Core Stability (Completed)
- **Pure-Rust Runtime**: Initial zero-dependency parsing engine.
- **Precedence & Associativity**: Basic support for operator binding.
- **Tree-sitter Parity**: Core grammar features parity.

## ✅ Milestone 0.7.0: GLR & Ambiguity (Completed)
- **GLR Engine**: Generalized LR parsing for inherently ambiguous grammars (C++, JS).
- **Conflict Handling**: Automatic stack forking and merging (SPPF).
- **External Scanners**: Support for custom lexing (e.g. Python indentation).

## 🚀 Milestone 0.8.0: The Publishable Baseline (Current — baseline achieved on `main`)
- ✅ **Supported CI Contract Green**: `just ci-supported` / `CI / ci-supported` is green on `main`. Full workspace status docs and supported-lane boundaries are documented in `docs/status/`.
- ✅ **Safety Audit**: SAFETY comments on all `unsafe` blocks in supported crates.
- ✅ **Testing Buildout**: 2,460+ tests — property, integration, snapshot, GLR-core, fuzzing, mutation guards, ABI matrix. Supported feature matrix is green. Mutation testing is configured.
- ✅ **Example Grammars**: 10 example grammars (arithmetic, optionals, repetitions, words, boolean, json, csv, lambda, regex, ini).
- ✅ **API Documentation**: Crate-level doc comments; `cargo doc` builds with 0 warnings. Book: 6+ chapters. Architecture chapter added.
- ✅ **WASM Compatibility**: All core crates verified for `wasm32-unknown-unknown`. WASM CI verification job.
- ✅ **Security Audit**: `cargo-audit` clean — 0 known vulnerabilities.
- ✅ **Error Message Quality**: Actionable diagnostics across parser, IR, and tablegen. Error display formatting tests.
- ✅ **Fuzzing Targets**: 22 fuzz targets covering parser, lexer, external scanners, stack pool, and concurrency.
- ✅ **CI Feature Matrix**: Crate × feature-flag test combinations with concurrency caps. Cross-platform advisory jobs for macOS/Windows.
- ✅ **Cargo.toml Metadata**: Publish-ready metadata across workspace. Publish order documented. `check-publish-ready.sh` script.
- ✅ **Workspace Structure**: 47 microcrates in `crates/`, benchmarks, fuzzing, golden-tests, and book scaffolding.
- ✅ **Table Compression**: Optimized parse tables using Tree-sitter format (>10x reduction).
- ✅ **Cross-Platform**: Linux verified, macOS/Windows CI advisory jobs in place.
- ✅ **Parallel Agent Work**: 14 waves of parallel agent work, 85+ commits driving the 0.8.0 release.
- ✅ **Backlog Convergence**: Final live branch [#264](https://github.com/EffortlessMetrics/adze/pull/264) merged into `main` on 2026-04-03.
- 🟡 **Remaining hardening**: worktree cleanup/documentation hardening ([#268](https://github.com/EffortlessMetrics/adze/issues/268)).

## 🚧 Milestone 0.9.0: Ecosystem & Tooling (Next)
- **Publish to crates.io**: Turn the now-green baseline on `main` into a release checklist and publishable core crate set.
- **CI Hardening Beyond the Supported Gate**: Reduce advisory-lane churn and make broader workflow behavior easier to interpret.
- **CLI Polish**: Improve the already-landed CLI surface (`adze check`, `adze stats`, etc.) instead of treating it as unimplemented.
- **Performance Optimization**: Arena allocator for parse forest nodes; benchmark suite with regression detection.
- **Incremental Parsing**: Stabilize forest-splicing for real-time editor performance.
- **Query Predicates**: Full compatibility with Tree-sitter `.scm` query files.
- **LSP Refinement**: Move LSP generator from prototype to "useful for production".
- **More Book Content**: End-to-end tutorials, attribute reference, migration guide from Tree-sitter.

## 🎯 Milestone 1.0.0: The Stability Contract
- **API Freeze**: Stable public API surface for `adze` and `adze-macro`.
- **Performance Baseline**: Documented benchmarks and complexity envelopes.
- **Multi-platform Stability**: Tier 1 support for Linux, macOS, Windows, and WASM.

---

## Non-Goals
- **Replacing Tree-sitter**: Adze aims for interoperability, not total replacement of the ecosystem.
- **Universal Grammar Support**: Focus is on a repeatable, safe pipeline for Rust developers.
