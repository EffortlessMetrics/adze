# GLR v1 Completion Contract

**Version**: 1.0.0
**Date**: 2025-11-20
**Status**: ACTIVE
**Branch**: `claude/complete-glr-v1-01W8RVz8tiznbXVTSkWicqPJ`
**Target**: Production-ready GLR parser for rust-sitter

---

## Executive Summary

This contract defines the complete specification for GLR v1, establishing clear acceptance criteria, test coverage requirements, and deliverables. It consolidates all scattered planning documents into a single source of truth using contract-first, BDD, and TDD methodologies.

**Current State** (as of 2025-11-20):
- ✅ GLR core engine implemented and tested
- ✅ Conflict detection and preservation logic complete
- ✅ Parse table generation infrastructure working
- ✅ Runtime2 GLR integration architecture in place
- ⚠️ Parser v4 table loading has known blocker (decoder/compression mismatch)
- ⚠️ BDD scenario tests not yet implemented
- ⚠️ Ambiguous grammar validation incomplete

**Completion Criteria**:
GLR v1 is **complete** when all acceptance criteria in this document are met and all specified tests pass.

---

## I. Scope Definition

### In Scope for GLR v1

1. **GLR Parser Core**
   - Multi-action cell support (shift/reduce, reduce/reduce conflicts)
   - Runtime forking on conflicts
   - Graph-structured stack (GSS) management
   - Parse forest generation
   - Deterministic tree selection from forest

2. **Conflict Handling**
   - Precedence-based conflict ordering (prefer shift, prefer reduce)
   - Associativity support (left, right, non-associative)
   - Fork action for unresolvable conflicts
   - Error reporting for non-associative conflicts

3. **Test Grammars** (minimum viable set)
   - Arithmetic with precedence/associativity (already exists)
   - Dangling-else (ambiguous grammar validation)
   - One "real-world" grammar: simple config language or expression grammar

4. **API Stability**
   - Parser::parse() with GLR backend selection
   - Tree API compatible with existing runtime
   - Feature flag: `glr` (default off for v1, default on for v0.7.0+)
   - Error handling with structured error types

5. **Documentation**
   - GLR architecture overview
   - User guide: when to use GLR vs LR
   - Grammar author guide: precedence and associativity
   - Migration guide from LR to GLR

### Out of Scope for GLR v1

1. **Performance Optimization** - Baseline functionality only
2. **Incremental GLR Parsing** - Deferred to v0.7.0
3. **Full Grammar Ecosystem** - Focus on proof-of-concept grammars
4. **Editor Integration** - Deferred to v1.0
5. **Advanced Disambiguation** - Basic precedence/associativity only

---

## II. Acceptance Criteria

### AC-1: GLR Core Engine Correctness

**Requirement**: GLR engine must correctly handle conflicts and produce valid parse forests.

**Acceptance Tests**:

```gherkin
Scenario: Fork on shift/reduce conflict
  Given a grammar with inherent shift/reduce ambiguity
  When the parser encounters a conflict during parsing
  Then the parser creates multiple parser stacks
  And all valid derivation paths are explored
  And the parse forest contains all valid parse trees
```

**Implementation Location**: `glr-core/src/lib.rs`, `runtime2/src/engine.rs`

**Test Location**: `glr-core/tests/test_fork_merge.rs`

**Success Criteria**:
- [ ] Parser forks correctly on conflicts
- [ ] All derivation paths explored without infinite loops
- [ ] Parse forest structure is valid (no dangling nodes)
- [ ] Memory usage bounded (no memory leaks during forking)

---

### AC-2: Precedence and Associativity

**Requirement**: Precedence and associativity annotations must correctly order conflicting actions.

**Acceptance Tests**:

```gherkin
Scenario: Left-associative subtraction
  Given the grammar rule: Expr → Expr - Expr [prec_left(1)]
  When parsing "20 - 10 - 5"
  Then the result is ((20 - 10) - 5) = 5
  Not (20 - (10 - 5)) = 15

Scenario: Multiplication has higher precedence than subtraction
  Given the grammar rules:
    | Rule          | Precedence |
    | Expr → Expr * Expr | prec_left(2) |
    | Expr → Expr - Expr | prec_left(1) |
  When parsing "1 - 2 * 3"
  Then the result is (1 - (2 * 3)) = -5
  Not ((1 - 2) * 3) = -3
```

**Implementation Location**: `glr-core/src/lib.rs::resolve_shift_reduce_conflict()`

**Test Location**: `example/src/arithmetic.rs`, `runtime2/tests/glr_precedence.rs`

**Success Criteria**:
- [x] Left associativity works for arithmetic operators ✅
- [x] Precedence ordering produces correct parse trees ✅
- [ ] Right associativity works (test with exponentiation: `2 ^ 3 ^ 4 = 2 ^ (3 ^ 4)`)
- [ ] Non-associative operators produce errors when chained

---

### AC-3: Ambiguous Grammar Handling

**Requirement**: GLR must correctly parse inherently ambiguous grammars without panicking.

**Acceptance Tests**:

```gherkin
Scenario: Dangling-else ambiguity
  Given the dangling-else grammar (if-then-else statements)
  When parsing "if a then if b then s1 else s2"
  Then the parser produces 2 valid parse trees:
    | Interpretation | Structure |
    | Shift (nested else) | if a then (if b then s1 else s2) |
    | Reduce (outer else) | (if a then if b then s1) else s2 |
  And the default tree (shift-preferred) is returned
  And both trees are accessible via forest API
```

**Implementation Location**: `example/src/dangling_else.rs` (new grammar)

**Test Location**: `runtime2/tests/test_ambiguous_grammars.rs` (new test file)

**Success Criteria**:
- [ ] Dangling-else grammar implemented and compiles
- [ ] Parser produces multiple trees for ambiguous input
- [ ] Default tree selection uses precedence ordering
- [ ] Forest API allows accessing all parse trees
- [ ] No panics on ambiguous input

---

### AC-4: Table Generation and Loading

**Requirement**: Parse tables must correctly encode multi-action cells and load without data loss.

**Acceptance Tests**:

```gherkin
Scenario: Multi-action cells preserved through encoding
  Given a grammar with shift/reduce conflicts
  When the parse table is generated via tablegen
  Then multi-action cells are created in the action table
  And the table is compressed using Tree-sitter format
  When the table is loaded via decoder
  Then all actions are preserved in correct order
  And no conflicts are lost during encoding/decoding
```

**Implementation Location**:
- Generation: `glr-core/src/lib.rs`, `tablegen/src/compress.rs`
- Loading: `runtime/src/decoder.rs`

**Test Location**: `runtime/tests/test_table_round_trip.rs`

**Success Criteria**:
- [ ] Multi-action cells generated correctly
- [ ] Compression preserves all actions
- [ ] Decoder loads multi-action cells without truncation
- [ ] Round-trip test: generate → compress → decode → verify equality

**Known Blocker**: See [PARSER_V4_TABLE_LOADING_BLOCKER.md](../plans/PARSER_V4_TABLE_LOADING_BLOCKER.md)
- Decoder may not handle GLR multi-action cells correctly
- Resolution required before AC-4 can pass

---

### AC-5: Runtime Integration

**Requirement**: GLR runtime must integrate seamlessly with existing rust-sitter runtime.

**Acceptance Tests**:

```gherkin
Scenario: GLR feature flag routing
  Given a grammar with GLR conflicts
  When compiled with --features glr
  Then the parser uses the GLR backend
  And parsing succeeds without errors
  When compiled without --features glr
  Then the parser uses the LR backend
  And compilation succeeds with conflict resolution warnings

Scenario: Tree API compatibility
  Given a parse tree produced by the GLR engine
  When using the Tree API (node(), child(), kind(), etc.)
  Then all operations work identically to LR-produced trees
  And AST extraction succeeds
```

**Implementation Location**: `runtime/src/__private.rs`, `runtime2/src/parser.rs`

**Test Location**: `runtime/tests/test_parser_routing.rs`, `example/tests/integration.rs`

**Success Criteria**:
- [x] Feature flag routing works correctly ✅
- [x] GLR backend selected when `glr` feature enabled ✅
- [ ] Tree API fully compatible (all methods work)
- [ ] AST extraction works with GLR-produced trees
- [ ] Error handling consistent between backends

---

### AC-6: Documentation Completeness

**Requirement**: All GLR features must be documented with examples and usage guidance.

**Deliverables**:

1. **Architecture Document** (`docs/architecture/GLR_ARCHITECTURE.md`)
   - GLR engine design
   - Parse forest representation
   - Conflict resolution strategy
   - Performance characteristics

2. **User Guide** (`docs/guides/GLR_USER_GUIDE.md`)
   - When to use GLR vs LR
   - How to enable GLR mode
   - Writing grammars for GLR
   - Debugging GLR conflicts

3. **Grammar Author Guide** (`docs/guides/PRECEDENCE_ASSOCIATIVITY.md`)
   - Understanding precedence declarations
   - Using `prec_left`, `prec_right`, `prec`
   - Handling ambiguity intentionally
   - Conflict inspection tools

4. **API Documentation** (inline rustdoc)
   - All public GLR APIs documented
   - Examples for common use cases
   - Migration notes from LR mode

**Success Criteria**:
- [ ] All 4 documentation deliverables created
- [ ] Code examples compile and run
- [ ] API documentation complete (100% coverage)
- [ ] Reviewed by external contributor for clarity

---

## III. Test Strategy

### Test Pyramid

```
          /\
         /  \
        /E2E \         5 tests  - Full GLR integration
       /------\
      /  INT   \       15 tests - Component integration
     /----------\
    /   UNIT     \     40 tests - Unit tests (GLR core, tablegen, decoder)
   /--------------\
```

### Test Categories

#### 1. Unit Tests (40 tests minimum)

**GLR Core** (`glr-core/tests/`)
- [x] Conflict detection (shift/reduce, reduce/reduce) ✅
- [x] Precedence comparison logic ✅
- [x] Fork action creation ✅
- [ ] Associativity handling (left, right, non-assoc)
- [ ] GSS stack merging
- [ ] Forest node creation

**Table Generation** (`tablegen/tests/`)
- [ ] Multi-action cell compression
- [ ] Action priority encoding
- [ ] Round-trip: ParseTable → TSLanguage → ParseTable
- [ ] Large action cell handling (>10 actions)

**Decoder** (`runtime/tests/`)
- [ ] Multi-action cell decoding
- [ ] Action priority preservation
- [ ] Error handling for malformed tables

#### 2. Integration Tests (15 tests minimum)

**GLR Runtime** (`runtime2/tests/`)
- [ ] Fork on conflict
- [ ] Merge identical stacks
- [ ] Parse forest generation
- [ ] Tree selection from forest
- [ ] Memory management during forking

**Grammar Integration** (`example/tests/`)
- [x] Arithmetic with precedence ✅
- [ ] Dangling-else ambiguous grammar
- [ ] Expression grammar (no precedence)
- [ ] Config language grammar (real-world example)

**Feature Flag Integration** (`runtime/tests/`)
- [x] GLR feature enabled → GLR backend ✅
- [ ] GLR feature disabled → LR backend
- [ ] Error messages for unsupported grammars

#### 3. BDD Scenario Tests (5 scenarios minimum)

**Location**: `runtime2/tests/bdd_glr_scenarios.rs` (new file)

**Scenarios** (from [BDD_GLR_CONFLICT_PRESERVATION.md](../plans/BDD_GLR_CONFLICT_PRESERVATION.md)):
1. [ ] Detect shift/reduce conflicts
2. [ ] Preserve conflicts with precedence ordering (prefer shift)
3. [ ] Preserve conflicts with precedence ordering (prefer reduce)
4. [ ] Fork for no precedence information
5. [ ] GLR runtime explores both paths

#### 4. End-to-End Tests (5 tests minimum)

**Location**: `example/tests/integration.rs`

- [x] Arithmetic: Parse "1 - 2 * 3" → "-5" ✅
- [ ] Dangling-else: Parse ambiguous if-then-else
- [ ] Config: Parse TOML-like config file
- [ ] Error recovery: Parse malformed input
- [ ] Performance: Parse large input (1000+ tokens)

---

## IV. Implementation Plan

### Week 1: Foundation & Blocker Resolution

**Priority**: CRITICAL - Resolve parser v4 table loading blocker

**Tasks**:
1. ✅ Create this GLR v1 completion contract
2. [ ] Investigate decoder.rs GLR multi-action cell handling
3. [ ] Fix table loading blocker (see AC-4)
4. [ ] Create diagnostic tests for table round-trip
5. [ ] Document findings in PARSER_V4_TABLE_LOADING_BLOCKER.md

**Deliverables**:
- [ ] Parser v4 successfully loads GLR tables
- [ ] Round-trip test passing: generate → compress → decode → verify
- [ ] Blocker resolution documented

**Success Criteria**: AC-4 tests passing

---

### Week 2: Ambiguous Grammar Validation

**Priority**: HIGH - Validate GLR with real ambiguous grammar

**Tasks**:
1. [ ] Implement dangling-else grammar (`example/src/dangling_else.rs`)
2. [ ] Add BDD scenario tests (5 scenarios from BDD plan)
3. [ ] Implement parse forest API for accessing multiple trees
4. [ ] Test conflict preservation end-to-end
5. [ ] Document ambiguous grammar handling

**Deliverables**:
- [ ] Dangling-else grammar compiles and parses
- [ ] All 5 BDD scenarios passing
- [ ] Parse forest API accessible
- [ ] Documentation updated

**Success Criteria**: AC-3 tests passing

---

### Week 3: API Stabilization & Documentation

**Priority**: HIGH - Lock in GLR API for v1

**Tasks**:
1. [ ] Complete Tree API compatibility tests
2. [ ] Test AST extraction with GLR-produced trees
3. [ ] Write all 4 documentation deliverables (AC-6)
4. [ ] Create code examples and tutorials
5. [ ] External documentation review

**Deliverables**:
- [ ] Tree API 100% compatible
- [ ] All documentation deliverables complete
- [ ] Examples compile and run
- [ ] API frozen for v1

**Success Criteria**: AC-5 and AC-6 tests passing

---

### Week 4: Polish & Release Prep

**Priority**: MEDIUM - Prepare for v1 release

**Tasks**:
1. [ ] Run full test suite (unit + integration + E2E)
2. [ ] Fix any failing tests
3. [ ] Performance baseline measurement
4. [ ] Memory profiling
5. [ ] Create release notes

**Deliverables**:
- [ ] All tests passing (100% pass rate)
- [ ] Performance baseline documented
- [ ] Memory usage profiled
- [ ] Release notes complete

**Success Criteria**: All AC-1 through AC-6 passing

---

## V. Contract Verification

### Automated Verification

```bash
# Run all GLR tests
cargo test --workspace --features glr

# Run BDD scenarios
cargo test --test bdd_glr_scenarios

# Run integration tests
cargo test -p rust-sitter-example --features glr

# Verify documentation builds
cargo doc --no-deps --features glr
```

### Manual Verification Checklist

Before declaring GLR v1 complete, verify:

- [ ] All acceptance criteria (AC-1 through AC-6) met
- [ ] All test categories have minimum required tests
- [ ] All tests passing (100% pass rate)
- [ ] Documentation complete and reviewed
- [ ] API stable and frozen
- [ ] Performance baseline established
- [ ] Memory usage profiled
- [ ] Known issues documented
- [ ] Migration guide written
- [ ] Release notes complete

---

## VI. Success Metrics

### Quantitative Metrics

- **Test Coverage**: ≥60 tests total (40 unit + 15 integration + 5 E2E)
- **Pass Rate**: 100% (no ignored tests in GLR path)
- **Documentation**: 4 documents + 100% API coverage
- **Performance**: Within 5× of LR mode (baseline, not optimized)
- **Memory**: < 10× input size for typical grammars

### Qualitative Metrics

- **API Usability**: External reviewer can enable GLR without confusion
- **Grammar Portability**: Can convert tree-sitter grammars to rust-sitter GLR
- **Error Messages**: Clear, actionable error messages for GLR-specific issues
- **Stability**: No panics on ambiguous grammars or edge cases

---

## VII. Risk Management

### High Risks

1. **Table Loading Blocker** (CRITICAL)
   - **Risk**: Decoder cannot handle multi-action cells
   - **Mitigation**: Prioritize decoder fix in Week 1
   - **Fallback**: Implement pure-Rust table format (JSON/bincode)

2. **API Instability**
   - **Risk**: API changes required late in v1 cycle
   - **Mitigation**: Lock API in Week 3, freeze before Week 4
   - **Fallback**: Mark GLR as experimental if API not stable

### Medium Risks

1. **Performance Issues**
   - **Risk**: GLR too slow for practical use
   - **Mitigation**: Document performance characteristics, defer optimization to v0.7.0
   - **Fallback**: Recommend LR mode for performance-critical applications

2. **Ambiguous Grammar Complexity**
   - **Risk**: Dangling-else or other grammars too complex to implement
   - **Mitigation**: Start with simplest ambiguous grammar, expand gradually
   - **Fallback**: Use simpler test grammar if dangling-else too complex

---

## VIII. Definition of Done

GLR v1 is **DONE** when:

1. ✅ All acceptance criteria (AC-1 through AC-6) met
2. ✅ All tests passing (100% pass rate, no ignored tests)
3. ✅ Documentation complete and reviewed
4. ✅ API stable and frozen
5. ✅ Performance baseline established
6. ✅ Contract verification checklist complete
7. ✅ Release notes written
8. ✅ Tagged in git as `v1.0.0-glr` or merged to main

---

## IX. References

### Related Documents

- [PARSER_V4_TABLE_LOADING_BLOCKER.md](../plans/PARSER_V4_TABLE_LOADING_BLOCKER.md) - Current blocker details
- [BDD_GLR_CONFLICT_PRESERVATION.md](../plans/BDD_GLR_CONFLICT_PRESERVATION.md) - BDD test specifications
- [PHASE_3_PURE_RUST_GLR_RUNTIME.md](./PHASE_3_PURE_RUST_GLR_RUNTIME.md) - Original GLR runtime plan
- [GLR_ENGINE_CONTRACT.md](./GLR_ENGINE_CONTRACT.md) - GLR engine contract
- [STATUS_NOW.md](../../STATUS_NOW.md) - Current project status

### External References

- [GLR Parsing (Wikipedia)](https://en.wikipedia.org/wiki/GLR_parser)
- [Tree-sitter Documentation](https://tree-sitter.github.io/tree-sitter/)
- [Dangling Else Problem](https://en.wikipedia.org/wiki/Dangling_else)
- [LR Parsing Theory](https://en.wikipedia.org/wiki/LR_parser)

---

**Contract Version**: 1.0.0
**Last Updated**: 2025-11-20
**Next Review**: After Week 1 completion
**Owner**: rust-sitter core team

---

**Signatures** (for contract acceptance):

- [ ] Technical Lead: _______________ Date: ___________
- [ ] Quality Assurance: _______________ Date: ___________
- [ ] Documentation Lead: _______________ Date: ___________

---

END OF CONTRACT
