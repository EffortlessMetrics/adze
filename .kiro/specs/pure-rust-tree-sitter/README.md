# Pure-Rust Tree-sitter Ecosystem Specification

This specification outlines the complete design and implementation plan for evolving rust-sitter into a pure-Rust Tree-sitter language generator ecosystem that eliminates all C dependencies while maintaining 100% compatibility with the existing Tree-sitter ecosystem.

## 📋 Specification Documents

### [Requirements](requirements.md)
Comprehensive requirements covering 13 key areas:
- Pure-Rust GLR parser generation
- Complete grammar IR system
- Static Language generation
- External scanner integration
- Build system requirements
- Backward compatibility
- Performance and memory efficiency
- Developer experience
- Ecosystem integration
- Testing and quality assurance
- Security and licensing
- Build system reliability
- ABI compatibility and versioning

### [Design](design.md)
Detailed technical architecture including:
- GLR-aware component design
- Data models and interfaces
- Table compression strategies
- External scanner FFI bridge
- Error handling and diagnostics
- Performance optimizations
- Security considerations
- Migration strategy

### [Implementation Plan](tasks.md)
12-week phased implementation roadmap:
- **Phase 0**: Research & Macro Hardening (Week 1)
- **Phase 1-2**: GLR Core Implementation (Weeks 2-6)
- **Phase 3-6**: Integration & Optimization (Weeks 7-9)
- **Phase 7-9**: Testing, Documentation & Release (Weeks 10-12)

### [Implementation Strategy](IMPLEMENTATION_STRATEGY.md)
Executive summary with key research findings and strategic priorities.

## 🔍 Key Research Findings

### Critical Discovery: Tree-sitter is GLR, not LR(1)
Tree-sitter's power comes from its GLR (Generalized LR) algorithm with compile-time conflict resolution, not simple LR(1) parsing. This fundamentally changes our implementation approach.

### Macro System Fragility
The existing rust-sitter macro system has critical debuggability issues that must be resolved before GLR implementation can proceed.

### Performance Target Precision
The 4-8x performance improvement target is realistic when framed as improvement over FFI-based Rust bindings, not specialized compiler frontends.

### Table Compression Criticality
Must replicate Tree-sitter's "small table" optimization bit-for-bit for ecosystem compatibility.

## 🎯 Strategic Priorities

1. **GLR State Machine Fidelity** - Support multiple actions per (state, lookahead) pair
2. **Conflict Resolution Logic** - Port Tree-sitter's exact precedence/associativity rules
3. **Parse Table Compression** - Bit-for-bit compatibility with C output
4. **Macro System Hardening** - Fix debuggability and IDE experience
5. **ABI 15 Conformance** - Match struct layout and function table exactly
6. **WASM Target Robustness** - Self-contained artifacts with no external dependencies

## 📊 Success Metrics

- **Compatibility**: 100% corpus compatibility with bit-for-bit table matching
- **Performance**: 4-8x faster than FFI-based Rust bindings
- **Size**: ≤70 kB gzipped WASM bundles
- **Quality**: Comprehensive fuzzing and golden-file testing
- **Developer Experience**: Reliable debugging and IDE integration

## 🚀 Getting Started

1. **Review the Requirements** - Understand the 13 key requirement areas
2. **Study the Design** - Familiarize yourself with the GLR-aware architecture
3. **Follow the Implementation Plan** - Start with Phase 0 macro hardening
4. **Reference the Strategy** - Use research findings to guide decisions

## 🤝 Contributing

This specification provides a complete roadmap for building the pure-Rust Tree-sitter ecosystem. Contributors should:

1. Start with Phase 0 tasks (macro system debugging)
2. Focus on GLR fidelity over optimization initially
3. Maintain strict compatibility with Tree-sitter's behavior
4. Use golden-file testing throughout development
5. Validate against real-world grammars continuously

## 📚 Additional Resources

- [Tree-sitter Documentation](https://tree-sitter.github.io/tree-sitter/)
- [rust-sitter Repository](https://github.com/EffortlessMetrics/rust-sitter)
- [Tree-sitter Grammar Development Guide](https://tree-sitter.github.io/tree-sitter/creating-parsers)

---

**Status**: Specification Complete - Ready for Implementation  
**Target**: rust-sitter 0.6.0 MVP Release  
**Timeline**: 12 weeks with disciplined execution