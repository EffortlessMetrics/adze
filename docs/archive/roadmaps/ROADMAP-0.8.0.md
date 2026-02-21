# Adze 0.8.0 Development Roadmap

## Overview

Version 0.8.0 focuses on production readiness, performance optimization, and preparing for public release.

## Status: In Active Development

- **0.7.0**: Internal release complete (2025-01-12)
- **0.8.0**: Development started (targeting Q1 2025)

## Core Priorities

### 1. Performance Optimization ⚡
- [ ] **Benchmark Infrastructure**
  - [x] Criterion benchmarks for table compression
  - [ ] Parse performance benchmarks
  - [ ] Memory usage profiling
  - [ ] CI performance regression detection

- [ ] **Table Compression Improvements**
  - [ ] Investigate alternative compression algorithms
  - [ ] Profile and optimize hot paths
  - [ ] Reduce memory allocations
  - [ ] Target: 20% reduction in compressed size

- [ ] **GLR Parser Optimization**
  - [ ] Fork/merge performance tuning
  - [ ] GSS memory pooling
  - [ ] Parallel fork processing (optional)
  - [ ] Target: < 2x overhead vs deterministic parsing

### 2. API Stabilization 🔒
- [x] **Public API Surface**
  - [x] Hide internal modules with `#[doc(hidden)]`
  - [x] Create prelude modules for stable imports
  - [x] Remove deprecated APIs

- [ ] **Documentation**
  - [ ] Complete rustdoc for all public APIs
  - [ ] Add usage examples for each module
  - [ ] Create cookbook with common patterns
  - [ ] API stability guarantees document

- [ ] **Error Handling**
  - [ ] Consistent error types across crates
  - [ ] Rich error context with suggestions
  - [ ] Error recovery strategies documentation

### 3. Real Grammar Support 🌍
- [ ] **Python Grammar**
  - [ ] Full test corpus passing
  - [ ] External scanner integration
  - [ ] Performance benchmarks
  - [ ] Integration with Python LSP

- [ ] **JavaScript/TypeScript**
  - [ ] Grammar port from tree-sitter
  - [ ] JSX support
  - [ ] Type annotations handling
  - [ ] Test corpus validation

- [ ] **Rust Grammar**
  - [ ] Complete macro support
  - [ ] Lifetime handling
  - [ ] Procedural macro integration
  - [ ] rust-analyzer integration prototype

### 4. Tooling & Developer Experience 🛠️
- [ ] **CLI Improvements**
  - [ ] Interactive grammar debugging
  - [ ] Performance profiling commands
  - [ ] Grammar validation with detailed reports
  - [ ] Migration tool from tree-sitter grammars

- [ ] **IDE Support**
  - [ ] VS Code extension prototype
  - [ ] Language server protocol implementation
  - [ ] Syntax highlighting configuration generator
  - [ ] Tree-sitter query compatibility layer

- [ ] **Testing Infrastructure**
  - [ ] Property-based testing with proptest
  - [ ] Differential testing against C tree-sitter
  - [ ] Grammar fuzzing framework
  - [ ] Automated corpus testing

### 5. WASM & Web Support 🌐
- [ ] **WASM Optimization**
  - [ ] Size reduction (target: < 500KB)
  - [ ] Streaming compilation support
  - [ ] Web worker integration
  - [ ] Memory management improvements

- [ ] **Web Demo**
  - [ ] Interactive playground
  - [ ] Grammar editor with live preview
  - [ ] Performance visualization
  - [ ] Share/embed functionality

### 6. Production Hardening 🛡️
- [ ] **Safety & Correctness**
  - [ ] Complete unsafe code audit
  - [ ] Formal verification of core algorithms (stretch goal)
  - [ ] Comprehensive fuzzing coverage
  - [ ] Memory leak detection in CI

- [ ] **Compatibility**
  - [ ] Tree-sitter C ABI compatibility tests
  - [ ] Migration guides from C tree-sitter
  - [ ] Backward compatibility policy
  - [ ] Version compatibility matrix

## Technical Debt Cleanup

- [ ] Remove `#[allow(dead_code)]` annotations
- [ ] Fix all clippy warnings in non-tablegen crates
- [ ] Consolidate duplicate code patterns
- [ ] Standardize error handling patterns
- [ ] Remove or document all TODO/FIXME comments

## Release Criteria

Before 0.8.0 public release:

1. ✅ All tests passing (including integration tests)
2. ✅ No critical clippy warnings
3. ⏳ Documentation coverage > 90%
4. ⏳ Benchmark suite with baselines
5. ⏳ At least 2 real language grammars working
6. ⏳ WASM build under 1MB
7. ⏳ Public API stability commitment

## Stretch Goals

- [ ] Incremental GLR parsing
- [ ] Grammar inference from examples
- [ ] Visual debugging tools
- [ ] Academic paper on pure-Rust GLR implementation
- [ ] Integration with popular Rust web frameworks

## Timeline

- **Phase 1** (Weeks 1-2): Performance optimization and benchmarking
- **Phase 2** (Weeks 3-4): Real grammar support and testing
- **Phase 3** (Weeks 5-6): Tooling and developer experience
- **Phase 4** (Weeks 7-8): Production hardening and release prep

## How to Contribute

1. Pick an unchecked item from the roadmap
2. Create an issue to discuss the approach
3. Submit a PR with tests
4. Update documentation as needed

## Success Metrics

- Parse performance within 2x of C tree-sitter
- Memory usage within 1.5x of C tree-sitter
- > 95% test corpus pass rate for supported languages
- < 1 second parse time for 10MB source files
- Zero panics in production use cases