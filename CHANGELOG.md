# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

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
