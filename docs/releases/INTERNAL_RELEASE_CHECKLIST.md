# Internal Release Checklist - rust-sitter v0.6.0-beta.2

## 🎯 Goal
Full Tree-sitter compatibility with GLR incremental parsing support for internal testing.

## ✅ Configuration Status

| Component | Status | Notes |
|-----------|--------|-------|
| **ir, glr-core, tablegen** | `publish = false` | Internal only, not for crates.io |
| **runtime** | `publish = false` (recommended) | Keep internal until ready for public beta |
| **Workspace builds** | ✅ | All default features compile |
| **Tree-sitter compatibility** | ✅ | FFI-compatible Language struct generation |

## 🏗️ Architecture Summary

### Core GLR Components (Internal)
- **rust-sitter-ir**: Grammar intermediate representation
- **rust-sitter-glr-core**: GLR parser generation algorithms  
- **rust-sitter-tablegen**: Table compression & Language struct generation

### Runtime Features
- **Default**: Standard Tree-sitter C runtime
- **pure-rust**: WASM-compatible pure Rust implementation
- **incremental_glr**: Feature-gated incremental reparsing

## 📋 Pre-Release Validation

### Build Verification
```bash
# Minimal build
cargo build --workspace --no-default-features

# Default features
cargo build --workspace

# With incremental GLR
cargo build --workspace --features incremental_glr
```

### Test Matrix
```bash
# Core tests
cargo test -p rust-sitter
cargo test -p rust-sitter --features incremental_glr

# Equivalence tests (when enabled)
cargo test -p rust-sitter --features incremental_glr -- tests::incremental_equiv

# Integration tests
cargo test -p rust-sitter-example
```

### Benchmark Validation (Optional)
```bash
# Quick benchmark run
QUICK_BENCH=1 cargo bench -p rust-sitter-benchmarks --bench incremental_bench
```

## 🔧 Known Issues

1. **wasm-demo**: Temporarily disabled arithmetic parser (needs API update)
2. **tablegen example**: `debug_artifacts.rs` needs Grammar API update

These are non-blocking for internal use.

## 🚀 Next Steps

### Immediate (This PR)
- [x] Fix publish flags for internal use
- [x] Verify workspace builds
- [x] Create this checklist

### Follow-up PRs
1. **Equivalence test expansion**: Use real arithmetic grammar
2. **Fast-path optimization**: Re-enable subtree reuse
3. **Fork-budget heuristic**: Add --fork-budget flag
4. **Documentation**: mdBook chapter on incremental GLR

### Performance Goals
- Single-char edits: 2x faster than full reparse
- Line-level edits: 1.5x faster than full reparse  
- Fork budget: Default 64, configurable

## 📊 Integration Status

| Feature | Implementation | Testing | Docs |
|---------|---------------|---------|------|
| GLR Parser | ✅ Complete | ✅ Basic | 🔄 In progress |
| Incremental | ✅ Feature-gated | ✅ Equivalence | 📝 Planned |
| Python Grammar | ✅ Compiles | 🔄 Manual | 📝 Planned |
| WASM Support | ✅ Builds | ⚠️ Partial | 📝 Planned |

## 🔒 Internal Use Only

This release is for internal testing and validation only. Do not publish to crates.io until:
1. Performance targets are met
2. API stability is confirmed
3. Documentation is complete
4. Public beta criteria are satisfied

---

**Version**: 0.6.0-beta.2-internal  
**Date**: January 2025  
**Status**: Ready for internal testing