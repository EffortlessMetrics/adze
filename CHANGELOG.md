# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **Complete README Coverage**: Added README.md files for all 60+ workspace crates, including core crates, grammar crates, microcrates, and infrastructure crates.
- **Enhanced Rustdoc**: Improved crate-level documentation for `adze`, `adze-ir`, `adze-glr-core`, and `adze-tablegen` with quick-start guides, architecture overviews, and feature flag tables.
- **Doc Tests**: Added 6 executable doc tests to `adze-ir` public API items (`SymbolId`, `Symbol`, `Grammar`, `GrammarBuilder`, `normalize()`).
- **Integration Tests**: Added 7 integration tests exercising the full Grammar → IR → FIRST/FOLLOW → LR(1) → Parse Tables pipeline in `adze-glr-core`.
- **Property Tests**: Added 17 proptest-based property tests for IR (symbol equality, serde roundtrip, normalization idempotence) and GLR core (FIRST/FOLLOW computation, terminal containment, nullable detection, determinism).
- **Table Compression Property Tests**: Added 5 property tests for tablegen compression (determinism, goto roundtrip, action roundtrip, single-token grammar, empty table rejection).
- **Crate Descriptions**: Added `description` fields to 13 workspace crates missing them.

### Fixed
- **Manifest Warnings**: Removed invalid `license` fields from dependency tables in `lsp-generator` and `wasm-demo` Cargo.toml files.
- **Duplicate Dependencies**: Resolved duplicate `clap` dependency in `lsp-generator/Cargo.toml`.
- **Crate Metadata**: Fixed missing workspace fields (edition, rust-version, authors) and added homepage, keywords, and categories for `adze-cli`, `adze-macro`, `adze-tool`, and `adze-common`.
- **Rustdoc Links**: Fixed broken intra-doc links in `adze-ir`, `adze-tablegen`, and `adze` runtime crates.
- **Package Includes**: Added `tests/**` to package include lists for core crates to ensure tests ship with published packages.

## [0.8.0] - 2026-02-22

**Focus**: Publishable baseline, documentation sync, and governance-as-code.

### Added
- **Governance-as-Code**: Integrated policy enforcement for backend selection (Pure-Rust vs GLR) and progress tracking via 25+ micro-crates in `crates/`.
- **Table Compression**: Optimized parse tables using Tree-sitter's format, achieving >10x reduction in binary size for large grammars.
- **Improved Macro Extraction**: Enhanced `Extract` trait for more robust typed AST construction from both LR(1) and GLR trees.
- **Standardized CI Lane**: Defined the "Supported Lane" (`just ci-supported`) ensuring core reliability across platforms.
- **Comprehensive Documentation**: Complete overhaul of all guides and READMEs to reflect the transition from `rust-sitter` to Adze.

### Changed
- **Project Rename**: Formally transitioned from `rust-sitter` to **Adze**.
- **MSRV**: Updated Minimum Supported Rust Version to **1.92** (Rust 2024 edition).
- **Default Backend**: Pure-Rust LR(1) is now the default and primary recommended path.

### Fixed
- **Precedence Disambiguation**: Corrected operator precedence conflicts in the GLR runtime.
- **EOF Handling**: Fixed proper end-of-input processing in the pure-Rust parser.
- **FFI Hardening**: Eliminated potential segmentation faults in the legacy C bridge.

---

## [0.7.0] - 2025-12-20

**Focus**: GLR Engine Completion and Ambiguity Handling.

### Added
- **GLR Fork/Merge**: Implementation of stack forking and merging (SPPF) for ambiguous grammars.
- **External Scanners**: Support for custom lexing logic in Rust (implemented for Python indentation).
- **Initial Query Support**: Basic Tree-sitter query pattern matching (`.scm` files).

---

## [0.6.1-beta] - 2025-01-22

### Fixed
- Fixed Accept action encoding (0x7FFF to 0xFFFF).
- Corrected decoder check order.
- Fixed token_count calculation to include EOF symbol.
- Added missing GOTO table entries to compressed parse tables.
