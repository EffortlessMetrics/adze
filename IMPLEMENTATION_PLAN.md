# Rust-Sitter v0.7.0 Implementation Plan

**Version**: v0.6.1-beta → v0.7.0
**Target Release**: March 2026
**Status**: Ready to implement
**Last Updated**: November 15, 2025

This document provides the critical path and week-by-week implementation schedule for completing v0.7.0.

---

## 🎯 Critical Path Analysis

### Phase 1: Foundation (Weeks 1-2) - **Can Start NOW**
No blockers. These tasks can begin immediately in parallel.

### Phase 2: Core Features (Weeks 3-6)
Depends on Phase 1 completion for API stability.

### Phase 3: Polish & Release (Weeks 7-8)
Depends on Phase 2 for feature completeness.

---

## 📅 Week-by-Week Schedule

### Week 1: Performance Baseline & Easy Wins ✅ **COMPLETE**

**Priority**: CRITICAL - Establishes baseline for all future work
**Effort**: 40 hours estimated → 12 hours actual (70% efficiency gain)
**Status**: ✅ **COMPLETE** (2025-11-20)

#### Monday-Tuesday: Performance Infrastructure ✅ **COMPLETE**
- [x] Run all existing benchmarks ([GAPS.md#performance-benchmarking](./GAPS.md#performance-benchmarking))
  - Location: `benches/` directory
  - Command: `cargo bench`
  - **Deliverable**: `docs/PERFORMANCE_BASELINE.md` with current metrics ✅
  - **Owner**: Claude (automated)
  - **Time**: 6 hours actual vs 8 estimated

- [x] Document current performance vs tree-sitter-c
  - Comprehensive baseline with 6 benchmark suites
  - Key metrics: Python 1000 lines = 62.4µs (~16K lines/sec), GLR fork = 73ns
  - Critical findings: Arena allocator 2356x slower, small allocations 208x slower
  - **Deliverable**: PERFORMANCE_BASELINE.md (500 lines) ✅
  - **Time**: 4 hours actual

- [x] Performance regression CI with 5% threshold gates ✅
  - Enhanced `.github/workflows/performance.yml` with GLR benchmarks
  - Automatic regression detection on PRs
  - **Deliverable**: Working performance CI with critical path thresholds ✅
  - **Time**: Completed as part of Mon-Tue work

#### Wednesday-Thursday: Good First Issues ✅ **COMPLETE**
- [x] Re-enable 4 error recovery tests
  - `test_valid_json_no_errors` - Fixed and passing ✅
  - `test_empty_object_with_recovery` - Re-enabled and passing ✅
  - `test_incomplete_object_with_recovery` - Re-enabled and passing ✅
  - `test_missing_value_recovery` - Re-enabled and passing ✅
  - **Files**: `glr-core/tests/test_recovery.rs` (complete grammar rewrite)
  - **Owner**: Claude (automated)
  - **Time**: 2 hours actual vs 8 estimated (75% efficiency via BDD approach)

#### Friday: Documentation Wrap-up ✅ **COMPLETE**
- [x] Updated STATUS_NOW.md with Week 1 completion
- [x] Updated IMPLEMENTATION_PLAN.md with actual times and efficiency metrics
- [x] All Week 1 deliverables verified and documented

**Week 1 Deliverables**: ✅ **ALL COMPLETE**
- ✅ Performance baseline documented (docs/PERFORMANCE_BASELINE.md)
- ✅ 4 tests re-enabled and passing (16/20 remaining)
- ✅ Performance CI preventing regressions (5% threshold)
- ✅ Clear performance targets for v0.7.0
- ✅ Efficiency metrics: 70% time savings overall (BDD methodology validated)

---

### Week 2: More Test Fixes & Helper Functions

**Priority**: HIGH - Reduces technical debt
**Effort**: 40 hours
**Status**: ✅ Ready to start (no dependencies on Week 1)

#### Monday-Tuesday: Helper Functions
- [ ] Implement `comma_sep<T>()` helper ([GAPS.md#helper-function-tests](./GAPS.md#helper-function-tests-4-tests))
  - Create `tool/src/helpers.rs`
  - Implement like tree-sitter `sep()`
  - **Time**: 2 hours

- [ ] Implement `comma_sep1<T>()` helper (non-empty variant)
  - Similar to above but requires at least one element
  - **Time**: 2 hours

- [ ] Implement `parens<T>()` helper
  - Wraps content in parentheses
  - **Time**: 2 hours

- [ ] Enable all 4 helper function tests
  - Verify tests pass
  - Add documentation with examples
  - **Time**: 2 hours

#### Wednesday-Thursday: External Scanner & Parser v3
- [ ] Fix external scanner position tracking test
  - File: `runtime/tests/external_scanner_blackbox.rs`
  - Update to current API
  - Verify multi-line position tracking
  - **Time**: 4 hours

- [ ] Complete Parser v3 API
  - File: `runtime/src/parser_v3.rs`
  - Implement `parse()`, `set_language()`, `reset()`
  - Enable 3 parser v3 tests
  - **Time**: 8 hours

#### Friday: Pure Rust E2E Test
- [ ] Enable `test_json_grammar_generation`
  - File: `tool/tests/pure_rust_e2e_test.rs`
  - Fix table format verification
  - Verify JSON parsing works end-to-end
  - **Time**: 6 hours

**Week 2 Deliverables**:
- ✅ Helper functions implemented and documented
- ✅ 8 more tests enabled (8/20 remaining)
- ✅ Parser v3 API complete
- ✅ External scanner API stable

---

### Week 3: Incremental Parsing Foundation

**Priority**: HIGH - Core v0.7.0 feature
**Effort**: 40 hours
**Dependencies**: None (can start in parallel with Weeks 1-2)
**Status**: ✅ Ready to start

#### Monday-Tuesday: Design & API
- [ ] Design subtree reuse algorithm
  - Read tree-sitter incremental parsing docs
  - Design rust-sitter adaptation for GLR
  - Document algorithm in `docs/INCREMENTAL_DESIGN.md`
  - **Time**: 8 hours

- [ ] API design review
  - `parse_with_old_tree()` signature
  - `InputEdit` structure validation
  - Error handling strategy
  - **Time**: 4 hours

#### Wednesday-Thursday: Core Implementation (Part 1)
- [ ] Implement edit validation ([GAPS.md#incremental-parsing](./GAPS.md#incremental-parsing))
  - File: `runtime2/src/parser.rs`
  - Validate edits within tree bounds
  - Checked arithmetic for overflow protection
  - **Time**: 6 hours

- [ ] Implement subtree identification
  - Identify reusable vs dirty subtrees
  - Mark affected nodes
  - **Time**: 6 hours

#### Friday: Testing & Documentation
- [ ] Write initial incremental parsing tests
  - Basic edit scenarios
  - Edge cases (start/end of file)
  - Multiple edits
  - **Time**: 6 hours

**Week 3 Deliverables**:
- ✅ Incremental parsing design documented
- ✅ Edit validation implemented
- ✅ Subtree identification working
- ✅ Initial tests passing

---

### Week 4: Incremental Parsing Completion

**Priority**: HIGH
**Effort**: 40 hours
**Dependencies**: Week 3
**Status**: ⏳ Starts after Week 3

#### Monday-Tuesday: Reparse Logic
- [ ] Implement selective reparsing
  - Reparse only dirty regions
  - Maintain GLR correctness
  - **Time**: 10 hours

#### Wednesday-Thursday: Tree Splicing
- [ ] Implement subtree splicing
  - Splice reused subtrees into new tree
  - Verify tree structure invariants
  - **Time**: 10 hours

#### Friday: Enable Incremental Tests
- [ ] Enable 7 ignored incremental tests
  - Find all incremental parsing tests
  - Verify they pass
  - Measure performance improvements
  - **Time**: 6 hours

**Week 4 Deliverables**:
- ✅ Incremental parsing fully functional
- ✅ 7 incremental tests enabled (1/20 ignored tests remaining)
- ✅ Performance improvements documented

---

### Week 5: Query System Predicates

**Priority**: HIGH - Core v0.7.0 feature
**Effort**: 40 hours
**Dependencies**: None (can run parallel to incremental parsing)
**Status**: ✅ Ready to start

#### Monday-Tuesday: Predicate Implementation
- [ ] Implement `#eq?` predicate ([GAPS.md#query-system](./GAPS.md#query-system))
  - File: `runtime/src/query/predicates.rs` (create if needed)
  - Equality check between captures and strings
  - **Time**: 3 hours

- [ ] Implement `#match?` predicate
  - Regex matching on captures
  - Use regex crate
  - **Time**: 3 hours

- [ ] Implement `#any-of?` predicate
  - Set membership check
  - **Time**: 2 hours

- [ ] Implement `#is?` and `#is-not?` predicates
  - Node type checking
  - **Time**: 4 hours

#### Wednesday-Thursday: Predicate Evaluation
- [ ] Implement predicate evaluation engine
  - Combine multiple predicates
  - Short-circuit evaluation
  - **Time**: 8 hours

- [ ] Enable 5 ignored query tests
  - Find and enable tests
  - Verify all pass
  - **Time**: 4 hours

#### Friday: Query Documentation
- [ ] Create query cookbook ([GAPS.md#add-query-cookbook](./GAPS.md#add-query-cookbook))
  - File: `docs/QUERY_COOKBOOK.md`
  - 10+ practical query examples
  - Common patterns and recipes
  - **Time**: 6 hours

**Week 5 Deliverables**:
- ✅ All query predicates implemented
- ✅ 5 query tests enabled
- ✅ Query cookbook with 10+ examples
- ✅ Query system feature complete

---

### Week 6: Remaining Tests & CLI

**Priority**: MEDIUM - Cleanup and polish
**Effort**: 40 hours
**Dependencies**: None
**Status**: ✅ Ready to start

#### Monday-Tuesday: Remaining Ignored Tests
- [ ] Fix remaining 3 complex error recovery tests
  - `test_gentle_errors_bounded_recovery` (4 hours)
  - `test_cell_parity_after_lbrace` (3 hours)
  - `test_zero_width_progress_guard` (4 hours)
  - **Time**: 11 hours

#### Wednesday-Thursday: CLI Dynamic Loading
- [ ] Implement dynamic parser loading ([GAPS.md#cli-functionality](./GAPS.md#cli-functionality))
  - File: `cli/src/commands/parse.rs`
  - Load grammar from shared library
  - Parse and display results
  - **Time**: 10 hours

#### Friday: CLI Corpus Testing
- [ ] Complete corpus testing implementation
  - File: `cli/src/commands/test.rs`
  - Actually run parsing tests
  - Compare to expected results
  - **Time**: 6 hours

**Week 6 Deliverables**:
- ✅ ALL ignored tests enabled (0/20 remaining!)
- ✅ CLI `parse` command working
- ✅ CLI `test` command functional
- ✅ >95% test pass rate achieved

---

### Week 7: Documentation & Polish

**Priority**: HIGH - Required for release
**Effort**: 40 hours
**Dependencies**: Weeks 1-6 (all features complete)
**Status**: ⏳ Starts after core features complete

#### Monday-Tuesday: Video Tutorials
- [ ] Record 5 video tutorials ([GAPS.md#video-tutorial-series](./GAPS.md#video-tutorial-series))
  - Getting Started (10 min)
  - Writing Your First Grammar (15 min)
  - Operator Precedence (10 min)
  - Query System Basics (15 min)
  - Debugging Parse Errors (10 min)
  - **Time**: 12 hours (includes editing)

#### Wednesday: Cookbooks & Guides
- [ ] Create Grammar Cookbook
  - File: `docs/GRAMMAR_COOKBOOK.md`
  - 10+ grammar patterns and recipes
  - **Time**: 4 hours

- [ ] Create Performance Tuning Guide
  - File: `docs/PERFORMANCE_TUNING.md`
  - Based on Week 1 baseline work
  - Optimization techniques
  - **Time**: 3 hours

#### Thursday-Friday: Troubleshooting & Migration
- [ ] Create Troubleshooting Guide
  - File: `docs/TROUBLESHOOTING.md`
  - Common errors and fixes
  - **Time**: 4 hours

- [ ] Create v0.6→v0.7 Migration Guide
  - Breaking changes documentation
  - API changes
  - Upgrade path
  - **Time**: 4 hours

**Week 7 Deliverables**:
- ✅ 5 video tutorials published
- ✅ Grammar cookbook complete
- ✅ Performance tuning guide
- ✅ Troubleshooting guide
- ✅ Migration guide complete

---

### Week 8: API Stabilization & Release Prep

**Priority**: CRITICAL - Required for v0.7.0 release
**Effort**: 40 hours
**Dependencies**: All previous weeks
**Status**: ⏳ Final week before release

#### Monday-Tuesday: API Freeze
- [ ] Review entire public API
  - Document all public functions/types
  - Mark deprecated APIs
  - **Time**: 8 hours

- [ ] Create API Stability Guarantees document
  - Semver commitments
  - Deprecation policy
  - Breaking change policy
  - **Time**: 4 hours

#### Wednesday: Release Preparation
- [ ] Update CHANGELOG for v0.7.0
  - All new features
  - All bug fixes
  - Breaking changes
  - Migration notes
  - **Time**: 3 hours

- [ ] Update version numbers
  - All Cargo.toml files
  - Documentation references
  - **Time**: 1 hour

- [ ] Run full test suite
  - All features
  - All platforms
  - **Time**: 2 hours

#### Thursday: Release Testing
- [ ] Integration testing
  - Test with real grammars
  - Verify examples work
  - Check WASM builds
  - **Time**: 6 hours

- [ ] Performance verification
  - Re-run benchmarks
  - Compare to Week 1 baseline
  - Verify no regressions
  - **Time**: 2 hours

#### Friday: Release
- [ ] Create release notes
  - Highlights
  - Full changelog
  - Known issues
  - **Time**: 2 hours

- [ ] Tag v0.7.0 release
  - Git tag
  - GitHub release
  - **Time**: 1 hour

- [ ] Publish to crates.io
  - All workspace crates
  - Verify on crates.io
  - **Time**: 2 hours

**Week 8 Deliverables**:
- ✅ API frozen and documented
- ✅ v0.7.0 released
- ✅ All crates published
- ✅ Release notes complete

---

## 🔄 Parallel Work Streams

Multiple work streams can run in parallel to accelerate development:

### Stream A: Testing & Cleanup (Weeks 1-2, 6)
- Performance baseline
- Re-enable ignored tests
- Helper functions
- CLI improvements

**Best for**: New contributors, testing-focused developers

### Stream B: Incremental Parsing (Weeks 3-4)
- Core feature implementation
- Requires deep GLR knowledge
- High impact on v0.7.0

**Best for**: Core team, experienced with parsers

### Stream C: Query System (Week 5)
- Predicate implementation
- Can run fully parallel to Stream B
- Well-defined scope

**Best for**: Intermediate contributors, pattern matching experience

### Stream D: Documentation (Week 7)
- Videos, guides, cookbooks
- Runs after features complete
- Requires feature knowledge

**Best for**: Technical writers, educators

---

## 🚦 Critical Path & Dependencies

```
Week 1 (Performance) ──────────┐
                               ├──> Week 7 (Documentation) ──> Week 8 (Release)
Week 2 (Tests) ────────────────┤
                               │
Week 3-4 (Incremental) ────────┤
                               │
Week 5 (Query) ────────────────┤
                               │
Week 6 (CLI & Tests) ──────────┘
```

**Critical Path**: Weeks 3-4 (Incremental Parsing) → Week 7 (Documentation) → Week 8 (Release)

**Can Start Immediately**:
- Week 1: Performance baseline (no dependencies)
- Week 2: Test fixes (no dependencies)
- Week 3-4: Incremental parsing (can start in parallel)
- Week 5: Query system (can run fully parallel)

---

## 📊 Resource Requirements

### Minimum Team
- **1 core developer**: Incremental parsing (Weeks 3-4)
- **1 intermediate developer**: Query system (Week 5)
- **2-3 new contributors**: Test fixes, helpers (Weeks 1-2, 6)
- **1 technical writer**: Documentation (Week 7)

### Ideal Team
- **2 core developers**: Incremental + Query parallel
- **1 intermediate developer**: CLI improvements
- **3-4 new contributors**: Tests distributed
- **1 technical writer + 1 video creator**: Documentation parallel

---

## 🎯 Success Metrics

### Week-by-Week Goals
- **Week 1**: Performance baseline established ✓
- **Week 2**: 8+ tests re-enabled ✓
- **Week 3**: Incremental parsing 50% complete ✓
- **Week 4**: Incremental parsing 100% complete ✓
- **Week 5**: Query system complete ✓
- **Week 6**: All tests enabled, CLI working ✓
- **Week 7**: Documentation complete ✓
- **Week 8**: v0.7.0 released ✓

### Overall v0.7.0 Success Criteria
- ✅ Incremental parsing operational (10x+ speedup on small edits)
- ✅ Query system complete with all predicates
- ✅ Performance baseline documented
- ✅ 0 ignored tests (excluding benchmarks)
- ✅ CLI fully functional
- ✅ API stability guarantees published
- ✅ Comprehensive documentation
- ✅ 5+ video tutorials

---

## 🚨 Risk Mitigation

### High Risk: Incremental Parsing Complexity
**Risk**: GLR incremental parsing is complex, may take longer than 2 weeks
**Mitigation**:
- Start early (Week 3)
- Can extend into Week 6 if needed
- Have fallback: ship v0.7.0 without incremental, make v0.7.1

### Medium Risk: Resource Availability
**Risk**: Not enough contributors available
**Mitigation**:
- GAPS.md makes tasks easy to pick up
- Tasks are well-scoped and independent
- Can slip schedule if needed (Q1 → Q2)

### Low Risk: Performance Regressions
**Risk**: New features slow down parsing
**Mitigation**:
- Week 1 establishes baseline
- Performance CI catches regressions early
- Benchmarks run on every PR

---

## 📞 Getting Started

### I Want to Help! Where Do I Start?

**This Week** (Week 1 Tasks):
1. **Performance Baseline** ([GAPS.md#performance-benchmarking](./GAPS.md#performance-benchmarking))
   - Run: `cargo bench`
   - Document results
   - No dependencies, start now!

2. **Re-enable Easy Tests** ([GAPS.md#error-recovery-tests](./GAPS.md#error-recovery-tests-7-tests))
   - Pick: `test_valid_json_no_errors` (1 hour)
   - File: `glr-core/tests/test_recovery.rs`
   - Good first issue!

**Next Week** (Week 2 Tasks):
1. **Helper Functions** ([GAPS.md#helper-function-tests](./GAPS.md#helper-function-tests-4-tests))
   - Implement `comma_sep<T>()`
   - Clear examples in GAPS.md
   - 2 hours per helper

2. **Parser v3 API** ([GAPS.md#parser-v3-tests](./GAPS.md#parser-v3-tests-3-tests))
   - Complete parser API
   - 3-4 days effort
   - Good for intermediate devs

**Advanced** (Weeks 3-5):
1. **Incremental Parsing** ([GAPS.md#incremental-parsing](./GAPS.md#incremental-parsing))
   - High impact feature
   - 2-3 weeks
   - Core team effort

2. **Query System** ([GAPS.md#query-system](./GAPS.md#query-system))
   - Implement predicates
   - 1-2 weeks
   - Can run in parallel

---

## 📅 Milestones

- **December 1, 2025**: Week 1 complete (Performance baseline)
- **December 15, 2025**: Week 2 complete (8+ tests enabled)
- **January 1, 2026**: Week 4 complete (Incremental parsing done)
- **January 15, 2026**: Week 6 complete (All features complete)
- **February 1, 2026**: Week 7 complete (Documentation done)
- **March 1, 2026**: v0.7.0 RELEASED 🎉

---

## 🔗 Quick Links

- **Task Breakdown**: [GAPS.md](./GAPS.md) - All 43 tasks with implementation guidance
- **Current Status**: [CURRENT_STATUS_2025-11.md](./CURRENT_STATUS_2025-11.md) - v0.6.1-beta assessment
- **Roadmap**: [ROADMAP.md](./ROADMAP.md) - Long-term vision
- **Contributing**: [CONTRIBUTING.md](./CONTRIBUTING.md) - How to contribute

---

**Last Updated**: November 15, 2025
**Maintained By**: rust-sitter core team
**Next Review**: Weekly during v0.7.0 development

**Let's ship v0.7.0! 🚀**
