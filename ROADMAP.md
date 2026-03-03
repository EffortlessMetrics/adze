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

## 🚀 Milestone 0.8.0: The Publishable Baseline (Current — ~90% complete)
- ✅ **CI Gate Green**: 1,416 tests passing, 0 failures. Full workspace compiles, clippy clean.
- ✅ **Safety Audit**: SAFETY comments on all `unsafe` blocks in supported crates.
- ✅ **Testing Buildout**: 1,416 tests — property, integration, snapshot, GLR-core, fuzzing. Feature matrix: 11/12 pass.
- ✅ **API Documentation**: Crate-level doc comments; `cargo doc` builds with 0 warnings. Book: 4 new chapters.
- ✅ **WASM Compatibility**: All core crates compile for `wasm32-unknown-unknown`.
- ✅ **Security Audit**: `cargo-audit` clean — 0 known vulnerabilities.
- ✅ **Error Message Quality**: Actionable diagnostics across parser, IR, and tablegen.
- ✅ **Fuzzing Targets**: 20 fuzz targets covering parser, lexer, external scanners, stack pool, and concurrency.
- ✅ **CI Feature Matrix**: Crate × feature-flag test combinations with concurrency caps. Mutation testing configured.
- ✅ **Cargo.toml Metadata**: Publish-ready metadata across workspace.
- ✅ **Workspace Structure**: 47 microcrates in `crates/`, benchmarks, fuzzing, golden-tests, and book scaffolding.
- ✅ **Table Compression**: Optimized parse tables using Tree-sitter format (>10x reduction).
- ✅ **Cross-Platform**: Linux verified, macOS/Windows CI being added.
- 🟡 **Remaining**: `cargo package` dry-run, feature-flag name standardization, doc-drift cleanup (`FR-001`).

## 🚧 Milestone 0.9.0: Ecosystem & Tooling (Next)
- **Publish to crates.io**: Initial release of core crates (`adze`, `adze-ir`, `adze-glr-core`, `adze-tablegen`).
- **CLI Utility**: `adze check`, `adze stats`, `adze fmt` for grammar validation, inspection, and debugging.
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
