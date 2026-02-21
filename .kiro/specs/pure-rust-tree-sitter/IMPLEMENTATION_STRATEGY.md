# Pure-Rust Tree-sitter Implementation Strategy

## Executive Summary

Based on comprehensive technical research, this document outlines the strategy for evolving adze into a complete, pure-Rust Tree-sitter ecosystem. The research revealed critical insights that fundamentally change our approach from a simple LR(1) parser generator to a sophisticated GLR (Generalized LR) system with compile-time conflict resolution.

## Key Research Findings

### 1. Tree-sitter is Fundamentally GLR, Not LR(1)

**Critical Discovery**: Tree-sitter's power comes from its GLR algorithm with compile-time conflict resolution, not simple LR(1) parsing. The system must support:
- Multiple actions per (state, lookahead) pair
- Fork/merge logic for handling ambiguous grammars
- Compile-time conflict resolution using precedence and associativity

**Impact**: This changes our entire architecture from a simple table-driven LR(1) automaton to a GLR parser generator that encodes fork/merge logic.

### 2. Macro System Fragility is a Blocking Issue

**Critical Problem**: The existing adze macro system has severe debuggability issues:
- `ADZE_EMIT_ARTIFACTS=true` causes build failures (Issue #63)
- Procedural macros fail on incomplete code, breaking IDE experience
- No reliable way to debug grammar generation process

**Impact**: Must be fixed in Phase 0 before proceeding with GLR implementation.

### 3. Performance Target Must Be Precisely Framed

**Research Finding**: The 4-8x performance improvement target is realistic when framed as improvement over FFI-based Rust bindings, not over specialized compiler frontends like rustc.

**Evidence**:
- rustc parses 2,157-line file in ~2ms
- tree-sitter-rust parses same file in 6.48ms (2-3x slower)
- Scala/Java projects saw 27x-134x speedups by eliminating JVM/IPC overhead
- Our target: 4-8x faster than current Rust FFI bindings by eliminating FFI overhead

### 4. Table Compression is Critical for Compatibility

**Technical Requirement**: Must replicate Tree-sitter's "small table" optimization bit-for-bit:
- `ts_small_parse_table`: Compressed one-dimensional arrays
- `ts_small_parse_table_map`: Index mapping for state lookup
- Table factoring and run-length encoding
- Python WASM parser: 465 kB uncompressed → 69 kB gzipped

### 5. WebAssembly Offers Architectural Advantages

**Key Insight**: Pure-Rust WASM implementation solves "external scanner dependency hell" that affects C/Emscripten-based web-tree-sitter by creating self-contained artifacts with no implicit external dependencies.

## Strategic Implementation Priorities

### Phase 0: Foundation (Week 1) - BLOCKING
1. **Fix ADZE_EMIT_ARTIFACTS** - Restore debugging capability
2. **Harden macro system** - Handle incomplete input gracefully for IDE compatibility
3. **Set up GLR-aware project structure** - Support multiple actions per state

### Phase 1-2: GLR Core (Weeks 2-6) - CRITICAL
1. **GLR state machine fidelity** - Multiple actions per (state, lookahead) pair
2. **Conflict resolution logic** - Port Tree-sitter's exact precedence/associativity rules
3. **Table compression** - Bit-for-bit compatibility with C output

### Phase 3-6: Integration & Optimization (Weeks 7-9)
1. **ABI 15 compliance** - #[repr(C)] Language struct with exact field layout
2. **External scanner FFI** - Bridge Rust scanners to Tree-sitter interface
3. **WASM optimization** - Target 70 kB gzipped size with full optimization pipeline

## Technical Architecture

### Core Components

```
adze/
├── ir/             # Grammar IR with GLR support
├── glr-core/       # GLR state machine generation
├── tablegen/       # Table compression (small table optimization)
├── scanner-bridge/ # External scanner FFI utilities
└── tool/           # Build.rs integration
```

### GLR-Specific Data Structures

```rust
pub struct ParseTable {
    // Support multiple actions per (state, lookahead)
    pub action_table: Vec<Vec<Action>>,
    pub goto_table: Vec<Vec<StateId>>,
    pub fork_points: Vec<ForkPoint>,
    pub merge_points: Vec<MergePoint>,
}

pub enum Action {
    Shift(StateId),
    Reduce(RuleId),
    Fork(Vec<Action>), // GLR fork point
    Accept,
    Error,
}
```

### Table Compression Strategy

```rust
pub struct CompressedTable {
    pub small_table: &'static [u16],      // ts_small_parse_table
    pub table_map: &'static [u16],        // ts_small_parse_table_map
    pub default_actions: &'static [Action],
}
```

## Success Metrics

### Phase-Level Gates
- **Phase 0**: ADZE_EMIT_ARTIFACTS works + macro handles incomplete input
- **Phase 1**: Grammar IR supports GLR + conflict resolution matches C exactly
- **Phase 2**: GLR parse tables bit-for-bit identical to Tree-sitter CLI output
- **Phase 6**: 4-8x performance improvement over FFI-based Rust bindings
- **Phase 7**: 100% corpus compatibility across major grammars

### Performance Targets
- **Parsing Speed**: 4-8x faster than current Rust FFI bindings
- **WASM Size**: ≤70 kB gzipped (matching tree-sitter-python)
- **Build Time**: <200ms for unchanged grammars (incremental cache)
- **Memory Usage**: ≤110% of C implementation

## Risk Mitigation

### Technical Risks
1. **GLR Complexity**: Extensive unit testing with ambiguous grammars + golden file comparison
2. **Table Compression**: Bit-for-bit validation against Tree-sitter CLI
3. **Macro Fragility**: Phase 0 focus on debugging and error recovery
4. **ABI Drift**: Continuous testing against multiple Tree-sitter versions

### Project Risks
1. **Scope Creep**: Strict phase boundaries with measurable success criteria
2. **Timeline Pressure**: Focus on correctness over optimization initially
3. **Community Adoption**: Early beta release and feedback integration

## Implementation Recommendations

### Immediate Actions (Week 1)
1. **Investigate adze Issue #63** - Critical blocking issue
2. **Set up golden-test pipeline** - C output vs Rust IR comparison
3. **Create GLR-aware project structure** - Support multiple actions per state

### Critical Path (Weeks 2-6)
1. **Port conflict resolution logic exactly** - No shortcuts on precedence/associativity
2. **Implement GLR state machine** - Fork/merge points and multiple actions
3. **Replicate table compression** - Bit-for-bit compatibility with C

### Quality Gates
1. **Round-trip testing** - C → Rust IR → C output identical
2. **Corpus validation** - 100% compatibility with major grammars
3. **Performance benchmarking** - Continuous measurement against FFI baseline

## Conclusion

The research provides a clear, data-driven path to success. The key insight that Tree-sitter is fundamentally GLR, not LR(1), changes our entire approach but makes the project more valuable. By focusing on GLR state machine fidelity, exact conflict resolution logic, and bit-for-bit table compression compatibility, we can deliver a pure-Rust Tree-sitter ecosystem that is faster, safer, and more robust than the current C-based system.

The 12-week timeline is aggressive but achievable with disciplined execution and early focus on the blocking macro system issues. Success depends on not shortcutting the GLR core implementation and maintaining strict compatibility with Tree-sitter's existing behavior.