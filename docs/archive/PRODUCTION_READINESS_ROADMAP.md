# Production Readiness Roadmap

**Date**: 2025-11-19
**Status**: ACTIVE
**Goal**: Transform adze from "strong beta" to "production-ready v1.0"

---

## Executive Summary

This roadmap takes adze from its current state (strong beta, macro path working, GLR algorithmically correct but not fully integrated) to a production-ready v1.0 release suitable for pointing at hiring managers and real-world use.

### Current State (v0.6.1-beta)
- ✅ Macro-based grammar generation: 100% working
- ✅ GLR table generation: Algorithmically correct
- ✅ Pure-Rust: Zero C dependencies, WASM compatible
- ✅ Type-safe ASTs: Compile-time validation
- ⚠️ GLR runtime: Infrastructure complete, integration blocked
- ⚠️ CI: Test failures emit warnings instead of errors
- ⚠️ Production polish: Debug statements, TODO comments in hot paths

### Target State (v1.0.0)
- ✅ GLR runtime: Fully integrated, tested, validated
- ✅ CI: Enforces test passage, blocks regressions
- ✅ Code quality: Production-grade, no debug noise
- ✅ Documentation: Complete, accurate, professional
- ✅ Test coverage: E2E scenarios, BDD, contract tests
- ✅ Performance: Benchmarked, optimized, profiled
- ✅ Stability: No known blockers, regression suite passing

---

## Phase 1: Critical Correctness Fixes (Week 1)

**Goal**: Fix bugs that would cause production issues
**Effort**: 6-8 hours
**Priority**: CRITICAL

### 1.1 Runtime Correctness Bugs

#### parser_v4 error_count plumbing
- **File**: `runtime/src/parser_v4.rs`
- **Issue**: `parse()` always returns `error_count: 0`, losing real error information
- **Fix**: Return `(ParseNode, u32)` from `parse_internal`, plumb to `Tree`
- **Test**: Parse erroneous input, verify `error_count > 0`
- **Acceptance**: All error recovery tests report correct error counts

#### GLR symbol metadata (is_named, is_extra)
- **File**: `runtime/src/__private.rs`
- **Issue**: Hardcoded `is_named: true`, `is_extra: false` in `convert_parse_node_v4_to_pure`
- **Fix**: Use `TSLanguage.symbol_metadata` for correct node properties
- **Test**: Parse grammar with punctuation and extras, verify node properties
- **Acceptance**: Visitor patterns work correctly, query selectors match properly

#### GLR grammar name for external scanners
- **File**: `runtime/src/__private.rs`
- **Issue**: Hardcoded `"grammar"` instead of actual grammar name
- **Fix**: Add `GRAMMAR_NAME` const to `Extract` trait, emit from macro/tool
- **Test**: Parse grammar with external scanner (e.g., Python indentation)
- **Acceptance**: External scanners loaded correctly by name

### 1.2 CI Enforcement

#### test-policy.yml: fail on test failures
- **File**: `.github/workflows/test-policy.yml`
- **Issue**: Non-timeout test failures emit warning instead of error
- **Fix**: Change `echo "::warning::"` to `echo "::error::"` and `exit $exit_code`
- **Test**: Commit failing test, verify CI blocks merge
- **Acceptance**: CI accurately reflects test health

### 1.3 Code Hygiene

#### Remove debug eprintln! statements
- **Files**: `tool/src/expansion.rs`, `tool/src/grammar_js/converter.rs`
- **Issue**: Debug prints ship in production binary
- **Fix**: Delete debug prints, or gate behind `debug_grammar` feature
- **Test**: Build release binary, verify no debug output
- **Acceptance**: Clean stderr on all parsers

### Milestone 1 Definition of Done
- [ ] All runtime correctness bugs fixed
- [ ] CI enforces test passage
- [ ] No debug noise in production builds
- [ ] All existing tests pass
- [ ] Documentation updated with fixes

**Timeline**: 2-3 days (part-time) or 1 day (full-time)

---

## Phase 2: GLR Conflict Preservation Validation (Week 1-2)

**Goal**: Prove GLR conflict preservation works end-to-end
**Effort**: 8-12 hours
**Priority**: HIGH

### 2.1 Create Conflict-Generating Test Grammars

#### Ambiguous expression grammar
- **File**: `example/src/ambiguous_expr.rs`
- **Grammar**: Expression without precedence annotations
  ```rust
  enum Expr {
      Binary(Box<Expr>, Op, Box<Expr>),  // Intentionally ambiguous
      Number(i32),
  }
  ```
- **Expected**: Multiple conflicts in parse table
- **Test**: Parse "1 + 2 + 3", verify GLR creates multiple parse trees
- **Acceptance**: Conflict count > 0, GLR forks on ambiguity

#### Dangling else grammar
- **File**: `example/src/dangling_else.rs`
- **Grammar**: Classic if/if-else ambiguity
  ```rust
  enum Stmt {
      If(Box<Expr>, Box<Stmt>),
      IfElse(Box<Expr>, Box<Stmt>, Box<Stmt>),
  }
  ```
- **Expected**: Shift/reduce conflict on "else" token
- **Test**: Parse "if a if b c else d", verify both parse trees exist
- **Acceptance**: GLR preserves both interpretations

#### Operator precedence without annotations
- **File**: `example/src/precedence_free.rs`
- **Grammar**: Arithmetic without `#[prec_left]` or `#[prec_right]`
- **Expected**: Multiple shift/reduce conflicts
- **Test**: Parse "1 + 2 * 3", verify multiple parse trees
- **Acceptance**: All valid associativities preserved

### 2.2 Validate Conflict Preservation in Tables

#### GLR conflict inspection test
- **File**: `runtime/tests/test_glr_conflict_inspection.rs`
- **Test**: Load parse table, verify multi-action cells exist
- **Validation**:
  ```rust
  let multi_action_cells = count_multi_action_cells(&parse_table);
  assert!(multi_action_cells > 0, "Expected GLR conflicts");
  ```
- **Acceptance**: All ambiguous grammars report >0 conflicts

#### Action encoding/decoding roundtrip
- **File**: `tablegen/tests/test_action_encoding_roundtrip.rs`
- **Test**: Encode multi-action cell, decode, verify preservation
- **Validation**: Fork actions maintain order and precedence
- **Acceptance**: No information loss in encoding/decoding

### 2.3 GLR Runtime Integration Tests

#### E2E GLR parsing with conflicts
- **File**: `runtime/tests/test_e2e_ambiguous_grammar_glr.rs`
- **Test**: Parse ambiguous input, extract all valid ASTs
- **Scenarios**:
  1. Parse ambiguous expression, verify forest contains multiple trees
  2. Extract typed AST, verify first valid parse returned
  3. Verify error recovery maintains multiple paths
- **Acceptance**: GLR successfully parses ambiguous inputs

### Milestone 2 Definition of Done
- [ ] 3+ ambiguous test grammars created
- [ ] All test grammars generate conflicts (verified in tables)
- [ ] GLR runtime successfully parses ambiguous inputs
- [ ] Conflict preservation validated end-to-end
- [ ] BDD scenarios pass for ambiguous grammars

**Timeline**: 3-4 days (part-time) or 2 days (full-time)

---

## Phase 3: Decoder and Table Loading Validation (Week 2)

**Goal**: Ensure decoder correctly interprets GLR tables
**Effort**: 6-10 hours
**Priority**: HIGH

### 3.1 Decoder Compatibility Audit

#### Inspect decoder for GLR support
- **File**: `runtime/src/decoder.rs`
- **Tasks**:
  1. Trace `decode_parse_table()` logic
  2. Verify multi-action cell handling
  3. Check action decoding matches schema
  4. Validate precedence/associativity preservation
- **Documentation**: Create `DECODER_GLR_AUDIT.md` with findings

#### Create decoder validation tests
- **File**: `runtime/tests/test_decoder_glr_tables.rs`
- **Tests**:
  1. Decode table with multi-action cells
  2. Verify action priority ordering preserved
  3. Test Fork action decoding
  4. Validate error action handling
- **Acceptance**: All decodings match tablegen output exactly

### 3.2 Table Loading End-to-End

#### Load GLR table in parser_v4
- **File**: `runtime/src/parser_v4.rs`
- **Test**: Load arithmetic grammar, verify table structure
- **Validation**:
  ```rust
  let parse_table = decoder::decode_parse_table(language);
  assert_eq!(parse_table.states.len(), EXPECTED_STATE_COUNT);
  assert!(parse_table.has_multi_action_cells());
  ```
- **Acceptance**: parser_v4 successfully initializes with GLR tables

#### Parse simple input with loaded table
- **File**: `runtime/tests/test_parser_v4_table_loading.rs`
- **Test**: Parse "1 + 2", verify successful parse tree
- **Validation**: No error nodes, correct AST structure
- **Acceptance**: parser_v4 successfully parses with decoded tables

### 3.3 Blocker Resolution

#### If decoder needs fixes
- **Action**: Implement fixes based on audit findings
- **Testing**: Validate with ambiguous grammar test cases
- **Documentation**: Update `PARSER_V4_TABLE_LOADING_BLOCKER.md` with resolution

#### If decoder is correct
- **Action**: Mark blocker as resolved, close issue
- **Validation**: All E2E tests pass with current decoder
- **Documentation**: Document why previous investigation was misleading

### Milestone 3 Definition of Done
- [ ] Decoder GLR compatibility audited and documented
- [ ] All decoder validation tests pass
- [ ] parser_v4 successfully loads and uses GLR tables
- [ ] Table loading blocker resolved (either fixed or closed)
- [ ] E2E parsing works with decoded tables

**Timeline**: 2-3 days (part-time) or 1-2 days (full-time)

---

## Phase 4: Complete GLR Runtime Integration (Week 3)

**Goal**: Wire GLR runtime as default for conflicting grammars
**Effort**: 6-8 hours
**Priority**: HIGH

### 4.1 Feature Flag and Routing (Already Complete ✅)

From GLR_RUNTIME_WIRING_PLAN.md, Steps 1-3 are done:
- ✅ Feature flag architecture
- ✅ Parser backend selection API
- ✅ Parser routing infrastructure

### 4.2 Grammar Metadata Generation

#### Emit HAS_CONFLICTS in Extract impl
- **File**: `tool/src/pure_rust_builder.rs`
- **Logic**: Detect multi-action cells in generated parse table
- **Generation**:
  ```rust
  impl Extract for Grammar {
      const HAS_CONFLICTS: bool = #has_conflicts;
      const GRAMMAR_NAME: &'static str = #grammar_name;
      // ...
  }
  ```
- **Test**: Verify arithmetic has `HAS_CONFLICTS = false`, ambiguous_expr has `true`
- **Acceptance**: Metadata accurately reflects grammar properties

### 4.3 Parser Selection Logic

#### Wire backend selection to use metadata
- **File**: `runtime/src/__private.rs`
- **Logic**:
  ```rust
  pub fn parse<T: Extract>(source: &str) -> Result<T> {
      let backend = ParserBackend::select(T::HAS_CONFLICTS);
      match backend {
          ParserBackend::TreeSitter => parse_with_tree_sitter(source),
          ParserBackend::PureRust => parse_with_pure_parser(source),
          ParserBackend::GLR => parse_with_glr::<T>(source),
      }
  }
  ```
- **Test**: Parse with different feature flags, verify correct backend used
- **Acceptance**: GLR automatically selected for conflicting grammars

### 4.4 Re-enable Disabled Tests

#### Arithmetic tests
- **File**: `example/src/arithmetic.rs`
- **Action**: Remove `#[ignore]` from associativity tests
- **Test**: Run with `--features glr`
- **Acceptance**: All tests pass, correct precedence/associativity

#### Other disabled test files
- **Search**: Find all `.rs.disabled` or `#[ignore]` with GLR-related comments
- **Process**: Re-enable one by one, fix if needed, commit incrementally
- **Tracking**: Update TEST_INVENTORY.md with re-enabled tests

### Milestone 4 Definition of Done
- [ ] Grammar metadata (HAS_CONFLICTS, GRAMMAR_NAME) emitted correctly
- [ ] Parser backend selection uses metadata automatically
- [ ] GLR path used for conflicting grammars
- [ ] All arithmetic tests re-enabled and passing
- [ ] At least 5 previously-disabled tests re-enabled
- [ ] Integration tests validate full pipeline

**Timeline**: 2-3 days (part-time) or 1-2 days (full-time)

---

## Phase 5: BDD Scenario Implementation (Week 3-4)

**Goal**: Implement and validate BDD scenarios from feature files
**Effort**: 4-6 hours
**Priority**: MEDIUM

### 5.1 GLR Runtime Integration Scenarios

#### Implement scenarios from glr_runtime_integration.feature
- **File**: `runtime/tests/glr_bdd_scenarios.rs`
- **Scenarios** (minimum 10):
  1. Left-associative multiplication
  2. Right-associative exponentiation
  3. Mixed precedence (addition and multiplication)
  4. Nested expressions with correct precedence
  5. Ambiguous parse with multiple valid trees
  6. Error recovery in GLR mode
  7. Incremental parsing with GLR
  8. External scanner integration
  9. Large file performance
  10. Unicode handling in GLR

#### Scenario structure
```rust
/// BDD Scenario: Left-associative multiplication
/// Given a grammar with left-associative multiplication at precedence 2
/// When I parse "1 * 2 * 3"
/// Then the result should be ((1 * 2) * 3)
#[test]
#[cfg(feature = "glr")]
fn scenario_left_associative_multiplication() {
    use arithmetic::*;
    let result = parse("1 * 2 * 3").expect("Parse should succeed");

    match result {
        Expr::Mul(
            box Expr::Mul(box Expr::Num(1), _, box Expr::Num(2)),
            _,
            box Expr::Num(3)
        ) => {
            // Correct: ((1 * 2) * 3)
        }
        _ => panic!("Expected left-associative tree, got: {:?}", result),
    }
}
```

### 5.2 Contract Test Scenarios

#### Grammar extraction contract validation
- **File**: `tool/tests/test_grammar_extraction_contract.rs`
- **Scenarios**:
  1. Enum inlining produces correct CHOICE members
  2. Precedence annotations preserved
  3. Field names propagated correctly
  4. External scanner registration
  5. Symbol metadata accurate

### 5.3 Performance Scenarios

#### GLR performance benchmarks
- **File**: `benches/glr_performance.rs`
- **Benchmarks**:
  1. Simple grammar parse speed
  2. Ambiguous grammar fork overhead
  3. Large file parsing (1MB+)
  4. Incremental edit performance
- **Baselines**: Compare to tree-sitter C runtime
- **Acceptance**: GLR within 2x of LR for non-ambiguous inputs

### Milestone 5 Definition of Done
- [ ] 10+ BDD scenarios implemented and passing
- [ ] All contract tests validate grammar extraction
- [ ] Performance benchmarks established
- [ ] Regression suite comprehensive
- [ ] CI runs all BDD scenarios

**Timeline**: 2-3 days (part-time) or 1 day (full-time)

---

## Phase 6: Documentation and Polish (Week 4)

**Goal**: Professional-grade documentation and final polish
**Effort**: 6-10 hours
**Priority**: MEDIUM

### 6.1 API Documentation

#### Complete rustdoc coverage
- **Target**: 100% public API documented
- **Files**: All `pub` items in `runtime/`, `macro/`, `tool/`
- **Quality**:
  - Examples for all major types
  - Links to relevant concepts
  - Safety documentation for unsafe code
  - Panic documentation
- **Test**: `cargo doc --workspace --no-deps`
- **Acceptance**: No missing doc warnings

### 6.2 User-Facing Documentation

#### Update README.md
- **Sections**:
  1. Status: Change to "v1.0 - Production Ready"
  2. Parser Modes: Update with actual feature flag behavior
  3. Quick Start: Verify all examples work
  4. Performance: Add benchmark results
- **Accuracy**: No misleading claims about GLR status

#### Update GETTING_STARTED.md
- **Content**:
  1. Step-by-step tutorial with GLR example
  2. When to use each parser backend
  3. How to handle ambiguous grammars
  4. Common pitfalls and solutions
- **Test**: Walk through tutorial in clean environment

#### Create MIGRATION_GUIDE.md
- **Audience**: v0.6 users upgrading to v1.0
- **Content**:
  1. Breaking changes in enum variant inlining
  2. How to opt out with `#[no_inline]`
  3. Feature flag changes
  4. API changes in Extract trait
- **Examples**: Before/after code snippets

### 6.3 Architecture Documentation

#### Update ARCHITECTURE.md
- **Sections**:
  1. GLR runtime architecture
  2. Parse table generation pipeline
  3. Decoder implementation
  4. Backend selection logic
- **Diagrams**: Data flow, component relationships
- **Status**: Mark GLR wiring as complete

#### ADR updates
- **ADR 0003**: Mark as "Implemented and Validated"
- **New ADRs**:
  - ADR 0004: GLR Runtime Integration Strategy
  - ADR 0005: Parser Backend Selection Model
  - ADR 0006: Table Encoding Schema Validation

### 6.4 Final Code Polish

#### Code quality audit
- **Clippy**: Zero warnings on `--all-targets --all-features`
- **Rustfmt**: Consistent formatting
- **Naming**: No TODO/FIXME/HACK comments in production paths
- **Safety**: All `unsafe` has justification comments
- **Error handling**: No `unwrap()` in library code

#### Performance audit
- **Profiling**: Run on representative grammars
- **Bottlenecks**: Identify and document hot paths
- **Optimizations**: Low-hanging fruit only (don't over-optimize)
- **Documentation**: Profile results in PERFORMANCE.md

### Milestone 6 Definition of Done
- [ ] 100% rustdoc coverage
- [ ] All user-facing docs accurate and helpful
- [ ] Migration guide complete
- [ ] Architecture docs reflect reality
- [ ] Zero clippy warnings
- [ ] No debug/TODO/HACK in production code
- [ ] Performance profiled and documented

**Timeline**: 3-4 days (part-time) or 2 days (full-time)

---

## Phase 7: Release Preparation (Week 5)

**Goal**: v1.0.0 release ready
**Effort**: 4-6 hours
**Priority**: MEDIUM

### 7.1 Version Bumps

#### Update Cargo.toml versions
- **Version**: 0.6.1-beta → 1.0.0
- **Files**: All workspace members
- **Changelog**: Comprehensive CHANGELOG.md

#### Dependency audit
- **Security**: `cargo audit`
- **Updates**: Conservative updates only
- **MSRV**: Verify Rust 1.89.0 requirement

### 7.2 Release Checklist

#### Pre-release validation
- [ ] All tests pass on: Linux, macOS, Windows
- [ ] All feature combinations compile
- [ ] Examples compile and run
- [ ] Documentation builds without warnings
- [ ] Changelog complete and accurate
- [ ] License and copyright up to date
- [ ] Git tags follow semver

#### Crates.io preparation
- [ ] crates.io metadata complete
- [ ] Keywords and categories accurate
- [ ] README renders correctly on crates.io
- [ ] No local path dependencies
- [ ] Publish dry-run succeeds

### 7.3 Release Communication

#### Release announcement
- **Platforms**: Blog, Reddit, HN, Twitter
- **Content**:
  1. What adze does
  2. Why GLR matters
  3. Use cases and examples
  4. Migration guide link
  5. Community involvement

#### GitHub release
- **Tag**: v1.0.0
- **Notes**: Comprehensive release notes
- **Assets**: Pre-built binaries (optional)
- **Examples**: Links to example repositories

### Milestone 7 Definition of Done
- [ ] Version 1.0.0 tagged and released
- [ ] Published to crates.io
- [ ] Release announcement posted
- [ ] Documentation live
- [ ] Community notified

**Timeline**: 1-2 days

---

## Success Metrics

### Quantitative
- **Test Coverage**: >80% line coverage
- **Test Count**: >200 tests across workspace
- **CI Time**: <10 minutes for full suite
- **Benchmark**: GLR within 2x of LR for simple grammars
- **Docs**: 100% public API documented
- **Warnings**: Zero clippy warnings

### Qualitative
- **User Experience**: Clear error messages, helpful docs
- **Code Quality**: Easy to read, well-structured
- **Maintainability**: New contributors can understand
- **Professional**: No "alpha software" feel
- **Trustworthy**: Hiring manager would be impressed

---

## Risk Management

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Decoder incompatible with GLR tables | Medium | High | Audit in Phase 3, fix before integration |
| Performance regression | Low | Medium | Benchmark continuously, optimize incrementally |
| Breaking changes needed | Low | Medium | Version 1.0 allows breaking changes |
| External scanner integration broken | Medium | Medium | Test with Python grammar |

### Schedule Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Phases take longer than estimated | High | Low | Timelines are part-time friendly, can extend |
| Blocker discovered late | Medium | Medium | Early validation in Phases 2-3 |
| Scope creep | Medium | Low | Stick to roadmap, defer nice-to-haves |

---

## Post-1.0 Roadmap (v1.1+)

### Features for Future Releases
- **Incremental Parsing**: GLR-compatible incremental edits
- **Query System**: Complete query language support
- **CLI Tools**: Code generation, grammar inspection
- **Editor Integration**: LSP server example
- **Performance**: Parallel parsing, SIMD optimizations
- **Ecosystem**: More example grammars, language packs

---

## Timeline Summary

| Phase | Effort | Duration (Part-Time) | Duration (Full-Time) |
|-------|--------|----------------------|----------------------|
| Phase 1: Critical Fixes | 6-8h | 2-3 days | 1 day |
| Phase 2: GLR Validation | 8-12h | 3-4 days | 2 days |
| Phase 3: Decoder Validation | 6-10h | 2-3 days | 1-2 days |
| Phase 4: Runtime Integration | 6-8h | 2-3 days | 1-2 days |
| Phase 5: BDD Scenarios | 4-6h | 2-3 days | 1 day |
| Phase 6: Documentation | 6-10h | 3-4 days | 2 days |
| Phase 7: Release | 4-6h | 1-2 days | 1 day |
| **TOTAL** | **40-60h** | **4-5 weeks** | **2-3 weeks** |

**Target v1.0.0 Release**: 4-5 weeks from start (part-time) or 2-3 weeks (full-time)

---

## Next Steps

1. ✅ Review and approve this roadmap
2. → Start Phase 1: Critical Correctness Fixes
3. → Execute phases sequentially with validation gates
4. → Track progress in STATUS_NOW.md
5. → Celebrate v1.0.0 release! 🎉

---

**Let's build production-ready adze! 🚀**
