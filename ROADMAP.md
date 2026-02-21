# Adze Roadmap

**Last Updated**: February 2026
**Current Version**: v0.6.x
**Status**: Macro-Based Grammar Generation Production-Ready

---

## Quick Status

| Component | Status |
|-----------|--------|
| Core GLR Parser | Complete |
| Macro-Based Grammars | Complete |
| Precedence/Associativity | Validated |
| Build Infrastructure | Mature |
| Documentation | Good |
| Incremental Parsing | Partial - feature-gated, needs completion |
| Query System | Partial - predicates incomplete |
| Performance | Unknown - no benchmarks run |

---

## v0.6.x (Current)

### What Actually Works Today

**Macro-Based Grammar Generation** (Complete)
- Define grammars with `#[adze::grammar]` annotations
- Generate working parsers at compile time
- Correct operator precedence (`1-2*3` -> `1-(2*3)`)
- Left associativity (`20-10-5` -> `(20-10)-5`)
- Text extraction with `text = true`
- Vec<> repetition with `#[repeat]`
- Whitespace handling with `#[extra]`

**Core Parser** (Algorithmically Correct)
- GLR parsing with multi-action cells
- Fork/merge on conflicts
- Error recovery (basic rejection of invalid input)
- Phase-2 re-closure for cascaded reductions
- Accept aggregation (no missed derivations)
- EOF recovery without data loss
- Epsilon loop prevention

**Infrastructure** (Production-Grade)
- CI/CD workflows covering lint, test, fuzz, benchmarks
- Pure-Rust implementation (no C dependencies)
- WASM compilation support
- Comprehensive test suite
- Zero clippy warnings across workspace

**Documentation**
- Getting Started guide with complete examples
- Up-to-date README and CHANGELOG
- API documentation
- Migration guides

### What This Enables

**You can build right now:**
1. Parser for custom DSLs using Rust macros
2. Arithmetic expression evaluators with correct precedence
3. Config file parsers with nested structures
4. Simple programming language parsers
5. WASM-based browser parsers with zero runtime deps

**See**: [docs/GETTING_STARTED.md](./docs/GETTING_STARTED.md) for complete working examples

---

## v0.7.0 - Feature Completion (Target: Q1 2026 -- at risk)

**Target**: January-March 2026
**Focus**: Complete incremental parsing, query system, and performance baseline
**Current Status**: Many items are still in progress. The Q1 2026 timeline is tight and some scope may slip to v0.8.0.

### Scope

**1. Incremental Parsing Completion** (Priority: High)
- [ ] Implement `parse_with_old_tree` functionality
- [ ] Enable ignored incremental tests
- [ ] Benchmark incremental vs full parse performance
- [ ] Document subtree reuse strategies
- **Estimated**: 2-3 weeks

**2. Query System Completion** (Priority: High)
- [ ] Finish predicate implementation: `#eq?`, `#match?`, `#any-of?`, `#is?`, `#is-not?`
- [ ] Enable ignored query tests
- [ ] Document query API with examples
- [ ] Add query cookbook with common patterns
- **Estimated**: 1-2 weeks

**3. Performance Baseline** (Priority: Critical)
- [ ] Run existing benchmarks vs tree-sitter-c
- [ ] Document current performance characteristics
- [ ] Identify optimization opportunities (flamegraphs, profiling)
- [ ] Add performance regression tests to CI
- **Estimated**: 1-2 weeks

**4. Test Maintenance** (Priority: High)
- [ ] Re-enable ignored tests across the workspace
- [ ] Achieve >95% test pass rate (excluding intentional benchmarks)
- **Estimated**: 2-3 weeks

**5. API Stabilization** (Priority: High)
- [ ] Freeze public API surface for v1.0
- [ ] Document breaking vs non-breaking changes
- [ ] Create API stability guarantees document
- [ ] Migration guide for 0.6 -> 0.7
- **Estimated**: Ongoing throughout v0.7 development

### Success Criteria

- Incremental parsing fully operational
- Query system complete with predicates
- Performance baseline documented
- Ignored tests minimized (all documented)
- API stability guarantees published

### Dependencies

- **None** - All work can proceed in parallel

---

## v0.8.0 - Performance & Polish (Q2-Q3 2026)

**Target**: April-September 2026
**Focus**: Performance optimization, developer experience

### Scope

**1. Performance Optimization** (Priority: Critical)
- [ ] Target: Within 3x of tree-sitter-c for typical grammars
- [ ] Implement shared parse-stack pool
- [ ] Arena allocation tuning for parse trees
- [ ] Memory profiling with heaptrack
- [ ] SIMD lexing experiments (if beneficial)

**2. Developer Experience** (Priority: High)
- [ ] Enhanced grammar debugger with fork visualization
- [ ] Improved error messages with suggestions
- [ ] CLI enhancements (format, validate, profile commands)
- [ ] VS Code extension prototype

**3. Grammar Ecosystem** (Priority: Medium)
- [ ] Grammar contribution guide
- [ ] Example grammar repository
- [ ] Testing framework for contributed grammars
- [ ] Community grammar showcase page

**4. Documentation** (Priority: Medium)
- [ ] Video tutorial series (5-10 short videos)
- [ ] Grammar author's cookbook
- [ ] Performance tuning guide
- [ ] Troubleshooting guide

### Success Criteria

- Parse performance within 3x of tree-sitter-c
- Community-contributed grammars
- CLI provides debugging tools
- Complete video tutorial series

---

## v0.9.0 - Community Ready (Q3-Q4 2026)

**Target**: July-December 2026
**Focus**: Ecosystem maturity, community infrastructure

### Scope

**1. Language Server Protocol**
- [ ] LSP implementation using existing generator
- [ ] Syntax highlighting generation
- [ ] Code folding support
- [ ] Outline provider

**2. Web Platform**
- [ ] Interactive playground enhancements
- [ ] Grammar editor with live preview
- [ ] Share/embed functionality
- [ ] WASM size optimization (<500KB)

**3. Advanced Grammar Support**
- [ ] Python: Full parity with tree-sitter-python
- [ ] JavaScript/TypeScript: JSX support
- [ ] Rust: Macro and lifetime support
- [ ] Production-quality grammars for multiple languages

**4. Community Infrastructure**
- [ ] Grammar registry/catalog
- [ ] Automated grammar testing
- [ ] Contribution workflow
- [ ] Governance model

### Success Criteria

- LSP servers for 3+ languages working
- Web playground production-ready
- Community grammars in registry
- Documented governance and contribution process

---

## v1.0.0 - Stable Release (Late 2026 / Early 2027)

**Target**: Q4 2026 or Q1 2027 depending on v0.9.0 readiness
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
- [ ] Production deployment case studies
- [ ] Comprehensive error messages with suggestions
- [ ] Migration guides for all 0.x versions
- [ ] Long-term support plan (LTS)

### Success Criteria

- Production deployments documented
- Community grammars in ecosystem
- Full tree-sitter feature parity
- Security audit passed
- Performance competitive (<3x tree-sitter-c)

---

## What We're NOT Building

**Clear Non-Goals**:

1. **Tree-Sitter Replacement Everywhere**: We target Rust-native use cases, especially WASM. Tree-sitter-c remains the best choice for many scenarios.

2. **All Tree-Sitter Grammars**: Focus is macro-based grammars. Porting existing tree-sitter grammars is possible but not primary.

3. **Faster Than Tree-Sitter**: Goal is "competitive" (within 3x), not "faster". Pure-Rust has overhead; WASM compatibility is the win.

4. **Dynamic Grammar Loading**: Compile-time generation is our strength. Runtime loading adds complexity without clear benefit.

5. **Backward Compatibility Pre-1.0**: Breaking changes may occur in 0.x releases. Post-1.0: strict semver.

---

## Development Philosophy

### Principles

1. **Ship Working Code**: v0.6.x proves macro generation works completely
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

## Success Metrics

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

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development setup and contribution guidelines. Browse [GitHub Issues](https://github.com/EffortlessMetrics/adze/issues) for open tasks, and look for "good first issue" labels to get started.

---

## Questions & Support

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and community chat
- **Discord**: (coming with v1.0)

---

**Maintained by**: adze core team
**Last Review**: February 2026

---

## License

Dual-licensed under MIT OR Apache 2.0
