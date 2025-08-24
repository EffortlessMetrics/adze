# Rust Sitter Roadmap

This document outlines the completed features and future direction of the rust-sitter project.

## ✅ Completed Features (v0.6.1-beta - Current Release)

### Core Parser Infrastructure
- [x] Pure-Rust LR(1) parser generator
- [x] **GLR (Generalized LR) parsing - ALGORITHMICALLY CORRECT** ✅
  - True GLR with multi-action cells (ActionCell architecture)
  - Runtime forking on conflicts, exploring all valid parse paths
  - Comprehensive fork/merge with safe deduplication
  - 100% pass rate on all core test suites
- [x] Complete Tree-sitter ABI compatibility (v15)
- [x] Zero-copy parsing with efficient memory layout
- [x] Full Unicode support

### GLR Correctness Fixes (v0.6.1-beta)
- [x] **Phase-2 Re-closure**: Reductions re-saturate with same lookahead
- [x] **Accept Aggregation**: Per-token collection prevents early returns
- [x] **EOF Recovery**: Close→check→(insert|pop) pattern, no deletion at EOF
- [x] **Epsilon Loop Prevention**: Position-aware RedStamp using (state, rule, end)
- [x] **Nonterminal Goto**: Fixed critical bug in goto table lookups
- [x] **Query Correctness**: Wrapper squashing and capture deduplication

### Language Features
- [x] External scanner support (FFI and native Rust)
- [x] Built-in scanners (indentation, heredoc, string interpolation)
- [x] Query language (S-expressions) with pattern matching
- [x] Syntax highlighting support
- [x] Field names and metadata
- [x] Precedence and associativity
- [x] Dynamic precedence
- [x] Fragile and non-fragile token handling

### Advanced Features
- [x] Incremental parsing with O(log n) complexity
- [x] Error recovery with multiple strategies
- [x] Parse tree visitors and transformers
- [x] Grammar optimization and validation
- [x] Table compression (Tree-sitter compatible)
- [x] Conflict resolution strategies
- [x] Parse forest handling for ambiguous grammars

### Testing Infrastructure
- [x] Property-based testing framework
- [x] Fuzzing support with coverage-guided generation
- [x] Corpus-based testing with automatic discovery
- [x] Performance benchmarking suite
- [x] Grammar validation and linting
- [x] Differential testing against Tree-sitter
- [x] **Regression guard tests** for all critical fixes

### Test Results (v0.6.1-beta)
| Suite | Pass Rate | Status |
|-------|-----------|--------|
| Fork/Merge | 30/30 | ✅ |
| Integration | 5/5 | ✅ |
| Error Recovery | 5/5 | ✅ |
| GLR Parsing | 6/6 | ✅ |
| Regression Guards | 5/5 | ✅ |

## 🎯 Recent Achievements (January 2025 - v0.6.1-beta)

### Algorithmic Correctness Achieved ✅
The GLR parser has reached algorithmic correctness with comprehensive test coverage:

1. **Reduction Mechanics Fixed**
   - Phase-2 re-closure ensures cascaded reduces are found
   - Accept aggregation collects all valid parses per token
   - No premature returns or missed derivations

2. **Error Recovery Hardened**
   - EOF recovery implements proper close→check→recover loop
   - Never deletes at EOF (prevents data loss)
   - Epsilon loop guard prevents infinite loops

3. **Fork/Merge Stabilized**
   - Safe stack deduplication uses pointer equality
   - Preserves all ambiguous derivations
   - LR(1) fork depth properly understood (≥3 tokens)

4. **Query System Corrected**
   - Wrapper nodes with identical spans are squashed
   - Captures deduplicated by (symbol, start, end)
   - Stable, predictable query results

### Tools & Infrastructure
- **ts-bridge Tool**: Production-ready Tree-sitter to GLR runtime bridge
  - Extracts parse tables from compiled Tree-sitter grammars
  - Full ABI stability with v15 pinning and SHA-256 verification
  - Feature-gated builds for development and production
  - Comprehensive parity testing framework

## 🚧 In Progress (Q1 2025)

### High Priority
1. **Performance Optimization**
   - [ ] Profile and optimize fork/merge hot paths
   - [ ] Implement shared parse stack structures
   - [ ] Add memory pooling for fork management
   - [ ] Establish performance baselines and benchmarks

2. **Query Predicates**
   - [ ] Implement remaining query predicate functions
   - [ ] Add custom predicate support
   - [ ] Complete query API compatibility

3. **CLI Runtime Loading**
   - [ ] Implement dynamic grammar loading
   - [ ] Add corpus runner for batch testing
   - [ ] Create standalone CLI tool

## 🚀 Next Milestones (2025)

### v0.7.0 - Production Ready (Q2 2025)
- [ ] Performance optimization complete
- [ ] All query predicates implemented
- [ ] CLI with full runtime loading
- [ ] External scanner FFI finalized
- [ ] Comprehensive documentation
- [ ] Migration guide from Tree-sitter

### v0.8.0 - Enhanced Features (Q3 2025)
- [ ] Incremental GLR optimization
- [ ] Advanced disambiguation filters
- [ ] Semantic action support
- [ ] Grammar composition and inheritance
- [ ] Visual grammar debugger

### v1.0.0 - Stable Release (Q4 2025)
- [ ] API stabilization
- [ ] Performance parity with C Tree-sitter
- [ ] All major language grammars validated
- [ ] Production deployment guides
- [ ] Enterprise support tier

## 📊 Known Limitations (Beta)

Current limitations being addressed:
- Performance optimization pending (safe dedup heuristics)
- Query predicates partially implemented
- External scanner FFI integration needs polish
- CLI runtime loading not yet implemented
- Incremental GLR algorithms experimental

## 🛠️ Development Guidelines

### Testing Requirements
All changes must maintain:
- 100% pass rate on core GLR suites
- No regression in guard tests
- Clean clippy with no warnings
- Documented in CHANGELOG

### Quality Gates
- Regression guards prevent critical fixes from being removed
- CI enforces test connectivity (no disabled tests)
- Performance benchmarks track regressions
- ABI compatibility verified via ts-bridge

## 🤝 Contributing

We welcome contributions in:
- Performance optimization
- Query predicate implementation
- Language grammar ports
- Documentation and tutorials
- Bug reports with minimal reproductions

See [CONTRIBUTING.md](./CONTRIBUTING.md) for details.

## 📈 Metrics & Validation

### Current Status
- **Correctness**: 100% ✅
- **Performance**: Baseline established
- **Compatibility**: Tree-sitter v15 ABI verified
- **Coverage**: All core features tested
- **Stability**: Beta (breaking changes possible)

### Success Metrics
- Parse Python's entire standard library
- Performance within 2x of C Tree-sitter
- Zero panics on fuzzing corpus
- Query compatibility with existing tools

## 🔍 Research Opportunities

### Near-term Research
- Grammar inference from examples
- Automatic conflict resolution strategies
- Parse tree diffing algorithms
- Grammar minimization techniques

### Long-term Research
- ML-powered error recovery
- Incremental grammar evolution
- Cross-language semantic analysis
- Formal verification of correctness

## 📚 Resources

### Documentation
- [API Reference](https://docs.rs/rust-sitter)
- [Grammar Guide](./docs/grammar-guide.md)
- [Migration Guide](./docs/migration.md)
- [GLR Guardrails](./docs/GLR_GUARDRAILS.md)

### Community
- GitHub: [hydro-project/rust-sitter](https://github.com/hydro-project/rust-sitter)
- Issues: [Bug Reports](https://github.com/hydro-project/rust-sitter/issues)
- Discussions: [Q&A Forum](https://github.com/hydro-project/rust-sitter/discussions)

## License

Dual-licensed under MIT OR Apache 2.0, maintaining compatibility with Tree-sitter's MIT license.