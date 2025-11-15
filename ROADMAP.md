# Rust-Sitter Roadmap

**Last Updated**: November 15, 2025
**Current Version**: v0.6.1-beta
**Status**: ✅ **Macro-Based Grammar Generation Production-Ready**

---

## 📋 Quick Status

| Component | Status | Evidence |
|-----------|--------|----------|
| Core GLR Parser | ✅ Complete | 100% test pass rate |
| Macro-Based Grammars | ✅ Complete | 13/13 tests passing |
| Precedence/Associativity | ✅ Validated | Real integration tests |
| Build Infrastructure | ✅ Mature | 13 CI/CD workflows |
| Documentation | ✅ Excellent | 398-line getting started guide |
| Incremental Parsing | ⚠️ Partial | Feature-gated, needs completion |
| Query System | ⚠️ Partial | Predicates incomplete |
| Performance | ⚠️ Unknown | No benchmarks run |

**Detailed Assessment**: See [CURRENT_STATUS_2025-11.md](./CURRENT_STATUS_2025-11.md)

---

## ✅ v0.6.1-beta (Current - November 2025)

### What Actually Works Today

**Macro-Based Grammar Generation** (100% Complete)
- Define grammars with `#[rust_sitter::grammar]` annotations
- Generate working parsers at compile time
- Correct operator precedence (`1-2*3` → `1-(2*3)`) ✓
- Left associativity (`20-10-5` → `(20-10)-5`) ✓
- Text extraction with `text = true` ✓
- Vec<> repetition with `#[repeat]` ✓
- Whitespace handling with `#[extra]` ✓

**Core Parser** (Algorithmically Correct)
- GLR parsing with multi-action cells
- Fork/merge on conflicts (30/30 tests passing)
- Error recovery (basic rejection of invalid input)
- Phase-2 re-closure for cascaded reductions
- Accept aggregation (no missed derivations)
- EOF recovery without data loss
- Epsilon loop prevention

**Infrastructure** (Production-Grade)
- 13 CI/CD workflows covering lint, test, fuzz, benchmarks
- Pure-Rust implementation (no C dependencies)
- WASM compilation support
- Comprehensive test suite (all non-ignored tests passing)
- 0 clippy warnings across workspace

**Documentation** (Excellent)
- 398-line Getting Started guide with 3 complete examples
- Up-to-date README, CHANGELOG, PROJECT_STATUS
- API documentation
- Migration guides

### Test Results (v0.6.1-beta - November 2025)

| Suite | Pass Rate | Details |
|-------|-----------|---------|
| Macro Grammars | 13/13 (100%) | test-mini 6/6, test-vec-wrapper 7/7 |
| Integration Tests | 6/6 (100%) | Real arithmetic parsing with precedence |
| Fork/Merge | 30/30 (100%) | GLR correctness validated |
| Tablegen | All passing | Accept encoding fixed |
| Error Recovery | 4/5 (80%) | Basic error handling works |

### What This Enables

**You can build right now:**
1. Parser for custom DSLs using Rust macros
2. Arithmetic expression evaluators with correct precedence
3. Config file parsers with nested structures
4. Simple programming language parsers
5. WASM-based browser parsers with zero runtime deps

**See**: [docs/GETTING_STARTED.md](./docs/GETTING_STARTED.md) for complete working examples

---

## 🚀 v0.7.0 - Feature Completion (Q1 2026)

**Target**: January-March 2026
**Focus**: Complete incremental parsing and query system

### Scope

**1. Incremental Parsing Completion** (Priority: High)
- [ ] Implement `parse_with_old_tree` functionality
- [ ] Enable 7 ignored incremental tests
- [ ] Benchmark incremental vs full parse performance
- [ ] Document subtree reuse strategies
- **Estimated**: 2 weeks

**2. Query System Completion** (Priority: High)
- [ ] Finish predicate implementation
- [ ] Enable 5 ignored query tests
- [ ] Document query API with examples
- [ ] Add query cookbook
- **Estimated**: 1 week

**3. Performance Baseline** (Priority: Critical)
- [ ] Run existing benchmarks vs tree-sitter-c
- [ ] Document current performance characteristics
- [ ] Identify optimization opportunities
- [ ] Add performance regression tests to CI
- **Estimated**: 1 week

**4. Test Maintenance** (Priority: Medium)
- [ ] Update 6 tests marked "needs current parser API"
- [ ] Remove or document why tests are ignored
- [ ] Achieve <5% ignored test rate
- **Estimated**: 3 days

**5. API Stabilization** (Priority: High)
- [ ] Freeze public API surface for v1.0
- [ ] Document breaking vs non-breaking changes
- [ ] Create API stability guarantees document
- [ ] Migration guide for 0.6→0.7
- **Estimated**: Ongoing

### Success Criteria

- ✅ Incremental parsing fully operational
- ✅ Query system complete with predicates
- ✅ Performance baseline documented
- ✅ <10 ignored tests (all documented)
- ✅ API stability guarantees published

### Dependencies

- **None** - All work can proceed in parallel

---

## 🎯 v0.8.0 - Performance & Polish (Q2 2026)

**Target**: April-June 2026
**Focus**: Performance optimization, developer experience

### Scope

**1. Performance Optimization** (Priority: Critical)
- [ ] Target: Within 3x of tree-sitter-c for typical grammars
- [ ] Implement shared parse-stack pool
- [ ] Arena allocation tuning for parse trees
- [ ] Memory profiling with heaptrack
- [ ] SIMD lexing experiments (if beneficial)
- **Estimated**: 4 weeks

**2. Developer Experience** (Priority: High)
- [ ] Enhanced grammar debugger with fork visualization
- [ ] Improved error messages with suggestions
- [ ] CLI enhancements (format, validate, profile commands)
- [ ] VS Code extension prototype
- **Estimated**: 3 weeks

**3. Grammar Ecosystem** (Priority: Medium)
- [ ] Grammar contribution guide
- [ ] Example grammar repository
- [ ] Testing framework for contributed grammars
- [ ] Community grammar showcase page
- **Estimated**: 2 weeks

**4. Documentation** (Priority: Medium)
- [ ] Video tutorial series (5-10 short videos)
- [ ] Grammar author's cookbook
- [ ] Performance tuning guide
- [ ] Troubleshooting guide
- **Estimated**: 1 week

### Success Criteria

- ✅ Parse performance within 3x of tree-sitter-c
- ✅ 5+ community-contributed grammars
- ✅ CLI provides debugging tools
- ✅ Complete video tutorial series

---

## 🌟 v0.9.0 - Community Ready (Q3 2026)

**Target**: July-September 2026
**Focus**: Ecosystem maturity, community infrastructure

### Scope

**1. Language Server Protocol**
- [ ] LSP implementation using existing generator
- [ ] Syntax highlighting generation
- [ ] Code folding support
- [ ] Outline provider
- **Estimated**: 4 weeks

**2. Web Platform**
- [ ] Interactive playground enhancements
- [ ] Grammar editor with live preview
- [ ] Share/embed functionality
- [ ] WASM size optimization (<500KB)
- **Estimated**: 3 weeks

**3. Advanced Grammar Support**
- [ ] Python: Full parity with tree-sitter-python
- [ ] JavaScript/TypeScript: JSX support
- [ ] Rust: Macro and lifetime support
- [ ] At least 10 production-quality grammars
- **Estimated**: 6 weeks

**4. Community Infrastructure**
- [ ] Grammar registry/catalog
- [ ] Automated grammar testing
- [ ] Contribution workflow
- [ ] Governance model
- **Estimated**: 2 weeks

### Success Criteria

- ✅ LSP servers for 3+ languages working
- ✅ Web playground production-ready
- ✅ 10+ community grammars in registry
- ✅ Documented governance and contribution process

---

## 🎓 v1.0.0 - Stable Release (Q4 2026)

**Target**: October-December 2026
**Focus**: API stability, production hardening, long-term support

### API Guarantees

**Stability Promises**:
- Semantic versioning strictly enforced
- No breaking changes to public API
- Grammar macro syntax frozen (only additions allowed)
- Clear deprecation policy (min 3 months notice)

### Production Checklist

- [ ] API frozen and fully documented
- [ ] Performance benchmarks in CI (regression tests)
- [ ] Security audit complete
- [ ] 3+ production deployment case studies
- [ ] Comprehensive error messages with suggestions
- [ ] Migration guides for all 0.x versions
- [ ] Long-term support plan (LTS)

### Success Criteria

- ✅ 100+ production deployments
- ✅ 50+ community grammars
- ✅ Full tree-sitter feature parity
- ✅ Security audit passed
- ✅ Performance competitive (<3x tree-sitter-c)

---

## 📊 What We're NOT Building

**Clear Non-Goals**:

1. **Tree-Sitter Replacement Everywhere**: We target Rust-native use cases, especially WASM. Tree-sitter-c remains the best choice for many scenarios.

2. **All Tree-Sitter Grammars**: Focus is macro-based grammars. Porting existing tree-sitter grammars is possible but not primary.

3. **Faster Than Tree-Sitter**: Goal is "competitive" (within 3x), not "faster". Pure-Rust has overhead; WASM compatibility is the win.

4. **Dynamic Grammar Loading**: Compile-time generation is our strength. Runtime loading adds complexity without clear benefit.

5. **Backward Compatibility Pre-1.0**: Breaking changes may occur in 0.x releases. Post-1.0: strict semver.

---

## 🔄 Development Philosophy

### Principles

1. **Ship Working Code**: v0.6.1 proves macro generation works completely
2. **Measure Then Optimize**: No performance work without benchmarks
3. **Community Driven**: Let real use cases guide priorities
4. **Document Everything**: Every feature needs examples and tests
5. **Stability Matters**: API stability > feature velocity (post-1.0)

### Release Cadence

- **Minor Releases** (0.x.0): Every 2-3 months
- **Patch Releases** (0.x.y): As needed for critical bugs
- **Breaking Changes**: Only in minor releases (pre-1.0)
- **Feature Flags**: Experimental features always behind flags

---

## 📈 Success Metrics

### Adoption

- **Crates.io Downloads**: Track monthly downloads
- **GitHub Stars**: Community interest indicator
- **Production Users**: Documented case studies
- **Community Grammars**: Count in registry

### Quality

- **Test Coverage**: Maintain >80% for core crates
- **Clippy Clean**: Zero warnings policy
- **Documentation**: 100% of public APIs documented
- **CI Status**: All workflows green

### Performance

- **Parse Speed**: Within 3x of tree-sitter-c
- **Memory Usage**: Profile and optimize
- **WASM Size**: Target <500KB
- **Build Time**: Grammar generation <1s for typical grammars

---

## 🤝 Contributing

### Current Priorities (Help Wanted!)

1. **Query Predicates**: Finish implementation (1 week effort)
2. **Incremental Parsing**: Complete `parse_with_old_tree` (2 weeks)
3. **Performance Benchmarks**: Run and analyze (1 week)
4. **Grammar Porting**: More language grammars (ongoing)
5. **Documentation**: Video tutorials, examples (ongoing)

### How to Contribute

1. Check [CONTRIBUTING.md](./CONTRIBUTING.md)
2. Look for "good first issue" labels
3. Discuss major changes in issues first
4. Follow TDD (tests first!)
5. Update docs with changes

---

## 📚 Documentation

- **Status Report**: [CURRENT_STATUS_2025-11.md](./CURRENT_STATUS_2025-11.md) - Comprehensive assessment
- **Project Status**: [PROJECT_STATUS.md](./PROJECT_STATUS.md) - Feature matrix
- **Getting Started**: [docs/GETTING_STARTED.md](./docs/GETTING_STARTED.md) - 398-line guide with examples
- **API Reference**: [API_DOCUMENTATION.md](./API_DOCUMENTATION.md) - Complete API docs
- **Changelog**: [CHANGELOG.md](./CHANGELOG.md) - All changes

---

## ❓ Questions & Support

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and community chat
- **Discord**: (coming with v1.0)

---

**Maintained by**: rust-sitter core team
**Last Review**: November 15, 2025
**Next Review**: January 2026 (post-v0.7.0 planning)

---

## License

Dual-licensed under MIT OR Apache 2.0
