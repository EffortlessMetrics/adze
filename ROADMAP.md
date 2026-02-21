# Adze Roadmap

**Published: 0.6.x (beta) · Dev head: 0.8.0-dev (unreleased) · MSRV: 1.92**
**Status**: GLR Parser Usable, Stabilization In Progress

---

## Quick Status

| Component | Status |
|-----------|--------|
| Macro-Based Grammars | Usable — `example/` has working demos |
| Core GLR Parser | Algorithmic tests pass |
| GLR Runtime (`runtime2/`) | Passes own test suite, not yet default |
| Tree-sitter Interop | Validated via golden tests for selected grammars |
| Grammar Crates | Python, JavaScript, Go compile; not published |
| CLI Tool | Exists (`cli/`), early stage |
| LSP Generator | Prototype (`lsp-generator/`) |
| Golden Tests | Working — validates against Tree-sitter reference |
| Documentation | mdBook + guides in `book/` |
| Incremental Parsing | Infrastructure exists, conservative fallback |
| Query System | Partial — predicates incomplete |
| Performance | Benchmark infra exists, baselines not published |
| BDD/Governance Contracts | In development (`crates/`) |

---

## v0.6.x — Foundation (Completed)

### What Works

**Macro-Based Grammar Generation**
- Define grammars with `#[adze::grammar]` annotations
- Generate working parsers at compile time
- Correct operator precedence (`1-2*3` -> `1-(2*3)`)
- Left associativity (`20-10-5` -> `(20-10)-5`)
- Text extraction with `text = true`
- Vec<> repetition with `#[repeat]`
- Whitespace handling with `#[extra]`

**Core Parser**
- GLR parsing with multi-action cells
- Fork/merge on conflicts
- Error recovery (basic rejection of invalid input)
- Phase-2 re-closure for cascaded reductions

**Infrastructure**
- CI/CD workflows covering lint, test, fuzz, benchmarks
- Pure-Rust implementation (no C dependencies)
- WASM compilation support
- Zero clippy warnings across workspace

**See**: [docs/GETTING_STARTED.md](./docs/GETTING_STARTED.md) for working examples

---

## v0.7.0 — Absorbed Into 0.8.0-dev

v0.7.0 was never formally released. Planned scope was absorbed into the current 0.8.0-dev branch.

**Done** (now in dev head):
- GLR runtime (`runtime2/`) with Tree-sitter compatible API
- Grammar crates for Python, JavaScript, Go
- Golden test framework validating against Tree-sitter reference
- mdBook documentation, CLI prototype, playground prototype
- External scanner integration (pure-Rust `ExternalScanner` trait)
- ts-bridge tool for extracting Tree-sitter parse tables

**Carried forward to 0.8.0**:
- Incremental parsing enablement
- Query system predicate completion
- Performance baseline publication

---

## v0.8.0 — Stabilize & Publish (Current Target)

**Focus**: Get what exists into publishable shape.

**1. Publish Pipeline**
- [ ] Resolve `publish = false` on workspace crates
- [ ] Dry-run publish with `cargo publish --dry-run`
- [ ] Ensure version consistency across workspace

**2. Documentation Polish**
- [ ] Update version references throughout docs
- [ ] Ensure mdBook builds clean
- [ ] Review and update API documentation

**3. Performance Baseline**
- [ ] Run existing benchmarks, document results
- [ ] Add performance regression checks to CI

**4. Incremental Parsing**
- [ ] Decide: enable conservative path or defer to 0.9.0
- [ ] If enabled, validate against golden tests

**5. Query System**
- [ ] Finish predicate implementation (`#eq?`, `#match?`, etc.)
- [ ] Document query API with examples

---

## v0.9.0 — Harden for Broader Use

Prototypes exist for several ecosystem tools. Focus is hardening what's already built.

- LSP generator: move from prototype to usable for selected grammars
- Playground: improve interactive experience, WASM size optimization
- Grammar crates: publish Python, JavaScript, Go if quality bar is met
- CLI: add grammar validation and debugging commands
- Community infrastructure: contribution guide, grammar testing framework

---

## v1.0.0 — Stable Release (2027)

**Focus**: API stability, production hardening, long-term support.

### API Guarantees
- Semantic versioning strictly enforced
- No breaking changes to public API
- Grammar macro syntax frozen (only additions allowed)
- Clear deprecation policy

### Production Checklist
- [ ] API frozen and fully documented
- [ ] Performance benchmarks in CI (regression tests)
- [ ] Security audit complete
- [ ] Comprehensive error messages with suggestions
- [ ] Migration guides for all 0.x versions

---

## What We're NOT Building

1. **Tree-Sitter Replacement Everywhere**: We target Rust-native use cases, especially WASM. Tree-sitter-c remains the best choice for many scenarios.

2. **All Tree-Sitter Grammars**: Focus is macro-based grammars. Porting existing tree-sitter grammars is possible but not primary.

3. **Faster Than Tree-Sitter**: Goal is "competitive" (within 3x), not "faster". Pure-Rust has overhead; WASM compatibility is the win.

4. **Dynamic Grammar Loading**: Compile-time generation is our strength. Runtime loading adds complexity without clear benefit.

5. **Backward Compatibility Pre-1.0**: Breaking changes may occur in 0.x releases. Post-1.0: strict semver.

---

## Development Philosophy

### Principles

1. **Ship Working Code**: v0.6.x proves macro generation works; dev head adds GLR runtime and grammar crates
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

---

**Maintained by**: adze core team
**Last Review**: February 2026

---

## License

Dual-licensed under MIT OR Apache 2.0
