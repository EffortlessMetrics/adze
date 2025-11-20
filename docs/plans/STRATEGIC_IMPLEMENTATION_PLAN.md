# Strategic Implementation Plan: Production-Ready rust-sitter

**Version**: 2.0.0
**Date**: 2025-11-20
**Status**: ACTIVE - Planning Complete for Phases I & II
**Methodology**: Contract-first, BDD/TDD, Infrastructure-as-Code, Documentation-Driven Development

---

## Executive Summary

This plan establishes a systematic approach to complete rust-sitter's journey to production readiness, using modern engineering practices including:
- **Infrastructure-as-Code**: Nix flakes for reproducible dev environments
- **Policy-as-Code**: Automated governance and quality gates
- **Contract-First Development**: Explicit acceptance criteria before implementation
- **BDD/TDD**: Behavior-driven specifications with test-first development
- **Documentation-Driven**: Specs and ADRs before code changes
- **Single Source of Truth**: Consolidated, version-controlled documentation
- **CI-as-Code**: GitHub Actions with deterministic build environments

**Current State** (2025-11-20):
- ✅ **GLR v1 COMPLETE** (144/144 tests passing, 100% pass rate, production-ready)
- ✅ Runtime2 + .parsetable pipeline working (89/89 tests, 100%)
- ✅ Comprehensive documentation (2,300+ lines: architecture, guides, API docs)
- ✅ Performance baseline established with CI regression gates
- ✅ **Phase 1A: Nix CI Integration** - 80% complete (AC-1,AC-4,AC-5 ✅, AC-2,AC-3 pending verification)
- ✅ **Phase 1B: Policy-as-Code** - 100% PLANNED (2,058 lines specs, ready for implementation)
- ✅ **Phase II: Incremental Parsing** - 100% PLANNED (2,478 lines specs, ready for implementation)
- 📋 **Total Planning**: 7,536 lines of comprehensive specifications across 3 major phases

**Planning Achievements**:
- **Nix Documentation**: 3,000+ lines (quickstart, troubleshooting, migration guides)
- **Policy-as-Code**: 2,058 lines (contract, ADR, 32 BDD scenarios)
- **Incremental Parsing**: 2,478 lines (contract, ADR, 32 BDD scenarios)

**Target State** (Q1 2026):
- ✅ Nix-based dev shell = CI environment (reproducible builds)
- ✅ Policy-as-Code enforcement (automated quality gates)
- ✅ Incremental GLR parsing (editor-class performance)
- ✅ Forest API v1 (programmatic ambiguity access)
- ✅ Performance within 2x of Tree-sitter C implementation
- ✅ Production-ready for regulated environments

---

## I. Foundational Infrastructure (Weeks 1-2)

### Goal: Establish reproducible development environment

**User Story**:
```gherkin
As a contributor
I want to run exactly the same toolchain as CI
So that "works on my machine" becomes "works everywhere"
```

### Phase 1A: Nix Development Shell (Week 1) - 80% COMPLETE

**Status**: ✅ 80% COMPLETE (AC-1,AC-4,AC-5 done, AC-2,AC-3 pending verification)
**Contract**: [NIX_CI_INTEGRATION_CONTRACT.md](../specs/NIX_CI_INTEGRATION_CONTRACT.md)
**ADR**: [ADR-0008-NIX-DEVELOPMENT-ENVIRONMENT.md](../adr/ADR-0008-NIX-DEVELOPMENT-ENVIRONMENT.md)
**Documentation**: 3,000+ lines across 3 comprehensive guides

**Acceptance Criteria**:
1. ✅ **AC-1: Nix Development Shell** - COMPLETE
   - Single `flake.nix` defines all dependencies
   - `nix develop` provides complete dev environment
   - All Rust toolchain components pinned (rust-toolchain.toml)
   - All system dependencies included
   - Environment variables set correctly

2. ⏳ **AC-2: Local Reproduction Capability** - VERIFICATION PENDING
   - Verification script ready: `scripts/verify-nix-local-reproduction.sh`
   - 9-step verification process defined
   - Awaiting Nix availability for execution

3. ⏳ **AC-3: Performance Baseline Consistency** - VERIFICATION PENDING
   - Verification script ready: `scripts/verify-nix-performance-consistency.sh`
   - Statistical analysis (2% variance threshold)
   - Awaiting Nix availability for execution

4. ✅ **AC-4: CI Pipeline Integration** - COMPLETE
   - `.github/workflows/nix-ci.yml` created and working
   - Nix caching configured (Cachix)
   - All jobs migrated to Nix environment

5. ✅ **AC-5: Documentation** - COMPLETE
   - Nix Quickstart Guide (1,100+ lines)
   - Nix Troubleshooting Guide (1,600+ lines)
   - Migration Guide (1,300+ lines)
   - CLAUDE.md updated with Nix section

**Deliverables**:
- [x] `flake.nix` at repository root ✅
- [x] `justfile` with CI commands (`just ci-all`, `just ci-perf`) ✅
- [x] `.github/workflows/nix-ci.yml` ✅
- [x] `CLAUDE.md` updated with Nix instructions ✅
- [x] ADR-0008 documenting Nix adoption rationale ✅
- [x] `docs/guides/NIX_QUICKSTART.md` ✅
- [x] `docs/guides/NIX_TROUBLESHOOTING.md` ✅
- [x] `docs/guides/MIGRATING_TO_NIX.md` ✅
- [x] `scripts/verify-nix-local-reproduction.sh` ✅ (ready to run)
- [x] `scripts/verify-nix-performance-consistency.sh` ✅ (ready to run)

**BDD Scenario**:
```gherkin
Scenario: Local development matches CI
  Given a fresh clone of the repository
  When I run `nix develop`
  And I run `just ci-all`
  Then all tests pass
  And the results match CI exactly
```

**Implementation**:
```nix
# flake.nix
{
  description = "rust-sitter dev + CI environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        devShells.default = pkgs.mkShell {
          name = "rust-sitter-dev";

          buildInputs = [
            # Rust toolchain (respects rust-toolchain.toml)
            pkgs.rustup

            # Core Rust tools
            pkgs.cargo-nextest
            pkgs.cargo-insta
            pkgs.just

            # Build dependencies
            pkgs.clang
            pkgs.llvmPackages.bintools
            pkgs.pkg-config
            pkgs.cmake
            pkgs.gnumake

            # System libraries
            pkgs.openssl
            pkgs.zlib

            # Tree-sitter dependencies
            # pkgs.tree-sitter  # Uncomment when ts-bridge ready

            # Scripting
            pkgs.python3
            pkgs.nodejs

            # Version control
            pkgs.git
          ];

          RUST_BACKTRACE = "1";
          RUST_TEST_THREADS = "2";
          RAYON_NUM_THREADS = "4";

          shellHook = ''
            if [ -f rust-toolchain.toml ]; then
              rustup show >/dev/null 2>&1 || rustup toolchain install
            fi
            echo "🦀 rust-sitter dev environment ready!"
            echo "Run 'just ci-all' to run full CI suite locally"
          '';
        };
      });
}
```

```makefile
# justfile
ci-all: ci-fmt ci-clippy ci-test

ci-fmt:
    cargo fmt --all -- --check

ci-clippy:
    cargo clippy --workspace --all-targets -- -D warnings

ci-test:
    cargo test --workspace -- --test-threads=2

ci-perf:
    cargo bench -p rust-sitter-benchmarks --bench glr_hot -- --save-baseline local
    cargo bench -p rust-sitter-benchmarks --bench glr_performance -- --save-baseline local

ci-glr:
    cargo test -p rust-sitter-glr-core --test test_recovery --features test-helpers
    cargo test -p rust-sitter-runtime2 --features glr-core
```

### Phase 1B: Policy-as-Code (Week 2) - 100% PLANNED

**Status**: 📋 100% PLANNED (2,058 lines specs, ready for 5-day implementation)
**Contract**: [POLICY_AS_CODE_CONTRACT.md](../specs/POLICY_AS_CODE_CONTRACT.md)
**ADR**: [ADR-0010-POLICY-AS-CODE.md](../adr/ADR-0010-POLICY-AS-CODE.md)
**BDD Scenarios**: [BDD_POLICY_ENFORCEMENT.md](./BDD_POLICY_ENFORCEMENT.md) (32 scenarios)
**Summary**: [POLICY_PLANNING_SUMMARY.md](../POLICY_PLANNING_SUMMARY.md)

**Acceptance Criteria** (5 ACs, fully specified):

1. **AC-P1: Pre-commit Hooks**
   - Framework: pre-commit (Python-based, industry standard)
   - Hooks: formatting, linting, test connectivity, commit message, large files
   - Installation: Automated in Nix shell (shellHook)
   - Performance: <5 seconds typical execution
   - BDD scenarios: 8 scenarios

2. **AC-P2: CI Policy Enforcement**
   - Workflow: `.github/workflows/policy.yml` (complete specification)
   - Jobs: quality-gates, security-scanning, performance-gates, test-connectivity
   - Execution: Parallel jobs for fast feedback
   - Enforcement: Cannot be bypassed, blocks PR merge
   - BDD scenarios: 10 scenarios

3. **AC-P3: Security Policies**
   - Vulnerability scanning: cargo audit (RustSec database)
   - License compliance: cargo deny (approved licenses only)
   - Secret detection: TruffleHog (high accuracy)
   - SBOM generation: cargo-sbom (supply chain visibility)
   - BDD scenarios: 6 scenarios

4. **AC-P4: Quality Verification Scripts**
   - `check-quality.sh`: Comprehensive quality validation (<30s)
   - `check-security.sh`: Security scanning (<10s)
   - `pre-push.sh`: Pre-push validation (quality + security)
   - Clear pass/fail with actionable remediation
   - BDD scenarios: 5 scenarios

5. **AC-P5: Documentation & Governance**
   - `POLICIES.md`: Policy overview (what, why, how)
   - `docs/guides/POLICY_ENFORCEMENT.md`: Implementation guide
   - Override procedures: Exception request process
   - Policy evolution: Versioning and adaptation strategy
   - BDD scenarios: 3 scenarios

**Architecture Decision: Layered Enforcement**
- **Layer 1**: Pre-commit hooks (fast local, <5s)
- **Layer 2**: Verification scripts (self-service, <30s)
- **Layer 3**: CI policy workflow (safety net, cannot bypass)

**Implementation Plan** (5 days):
- **Day 1**: Pre-commit setup
- **Day 2**: Verification scripts
- **Day 3**: CI policy workflow
- **Day 4**: Security & performance gates
- **Day 5**: Documentation & testing

**Strategic Impact**:
- Eliminates manual quality checks (30-40% time savings)
- Enables enterprise-grade governance (security, compliance)
- Fast feedback (<5s local, automated gates in CI)
- Zero tolerance (formatting, linting, vulnerabilities, test failures)

**Deliverables** (all specified, ready to implement):
- [ ] `.pre-commit-config.yaml` (5+ hooks)
- [ ] `.github/workflows/policy.yml` (complete workflow)
- [ ] `scripts/check-quality.sh`
- [ ] `scripts/check-security.sh`
- [ ] `scripts/pre-push.sh`
- [ ] `audit.toml`, `deny.toml` (security config)
- [ ] `POLICIES.md` (policy documentation)
- [ ] `docs/guides/POLICY_ENFORCEMENT.md`
- [ ] `.github/ISSUE_TEMPLATE/policy-override.md`

**BDD Scenario** (example of 32 total):
```gherkin
Scenario: Pre-commit hooks catch formatting issues
  Given I have modified Rust files
  And the files are not formatted correctly
  When I run "git commit -m 'feat: new feature'"
  Then the commit is blocked
  And I see a clear error: "Code not formatted. Run: cargo fmt"
  When I run "cargo fmt"
  And I run "git commit -m 'feat: new feature'"
  Then the commit succeeds
```

---

## II. GLR v1 Completion (Weeks 3-4) ✅ **COMPLETE**

**Status**: ✅ **PRODUCTION-READY** (Completed 2025-11-20)
**Achievement**: 144/144 tests passing (100%), all 6 acceptance criteria met
**Documentation**: 2,300+ lines comprehensive (architecture + guides + API docs)
**Summary**: [GLR_V1_COMPLETION_SUMMARY.md](../releases/GLR_V1_COMPLETION_SUMMARY.md)

### Goal: Complete all GLR v1 acceptance criteria ✅ ACHIEVED

**Reference**: [GLR_V1_COMPLETION_CONTRACT.md](../specs/GLR_V1_COMPLETION_CONTRACT.md)

### Phase 2A: Whitespace-Aware Tokenization (Week 3)

**Current Blocker**: BDD scenario 7 (complex ambiguous input) requires whitespace handling

**Contract**: AC-1 (GLR Core Engine Correctness) - Full completion

**Acceptance Criteria**:
1. Tokenizer correctly handles whitespace tokens
2. Dangling-else grammar parses with proper whitespace
3. BDD scenario 7 passes with complex input
4. No regressions in existing tests

**Deliverables**:
- [ ] Whitespace token support in runtime2/tokenizer.rs
- [ ] Updated BDD tests with whitespace scenarios
- [ ] Test coverage for edge cases (multiple spaces, tabs, newlines)
- [ ] Documentation of whitespace handling strategy

**BDD Scenario**:
```gherkin
Scenario: Parse dangling-else with whitespace
  Given the dangling-else grammar
  When parsing "if a then if b then s1 else s2"
  Then tokenization includes whitespace tokens
  And the parser produces valid parse trees
  And all whitespace is preserved in the tree
```

### Phase 2B: Tree API Compatibility (Week 3-4)

**Contract**: AC-5 (Runtime Integration) - Full completion

**Acceptance Criteria**:
1. All Tree API methods work with GLR-produced trees
2. Node traversal (parent, child, sibling) functions correctly
3. Node properties (kind, start/end byte, text) accurate
4. AST extraction from GLR trees works
5. Compatibility tests pass for all API surfaces

**Deliverables**:
- [ ] Comprehensive Tree API test suite
- [ ] AST extraction validation tests
- [ ] Performance benchmarks for tree operations
- [ ] API compatibility matrix documentation

**BDD Scenario**:
```gherkin
Scenario: GLR tree API compatibility
  Given a parse tree from GLR engine
  When using Tree::root_node()
  And traversing with node.child(0)
  And extracting text with node.utf8_text()
  Then all operations succeed
  And results match LR-produced trees
```

### Phase 2C: Documentation Completion (Week 4)

**Contract**: AC-6 (Documentation Completeness) - Full completion

**Acceptance Criteria**:
1. Architecture document (GLR_ARCHITECTURE.md) complete
2. User guide (GLR_USER_GUIDE.md) complete
3. Grammar author guide (PRECEDENCE_ASSOCIATIVITY.md) complete
4. API documentation 100% coverage
5. External review completed

**Deliverables**:
- [ ] `docs/architecture/GLR_ARCHITECTURE.md`
- [ ] `docs/guides/GLR_USER_GUIDE.md`
- [ ] `docs/guides/PRECEDENCE_ASSOCIATIVITY.md`
- [ ] Inline rustdoc for all public APIs
- [ ] External review feedback incorporated

---

## III. Performance Optimization (Weeks 5-6)

### Goal: Achieve performance within 2x of Tree-sitter C

**Reference**: ROADMAP_2025.md - Week 1-2 Performance Sprint

### Phase 3A: Profiling and Bottleneck Analysis (Week 5)

**Contract**: Establish performance baselines and identify optimization targets

**Acceptance Criteria**:
1. Profile GLR fork/merge on large Python files (>10K LOC)
2. Identify top 5 performance bottlenecks
3. Memory usage profiled with heaptrack
4. Benchmark against tree-sitter-c reference
5. Optimization targets documented

**Deliverables**:
- [ ] Performance profiling report
- [ ] Flamegraphs for Python parsing
- [ ] Memory allocation analysis
- [ ] Comparison with Tree-sitter C benchmarks
- [ ] `docs/PERFORMANCE_OPTIMIZATION_PLAN.md`

**BDD Scenario**:
```gherkin
Scenario: Identify performance bottlenecks
  Given a 10K line Python file
  When profiling with cargo-flamegraph
  Then the top 5 hot paths are identified
  And memory allocations are tracked
  And optimization targets are prioritized
```

### Phase 3B: Arena Allocation and Pool Optimization (Week 6)

**Contract**: Reduce allocations in hot paths

**Acceptance Criteria**:
1. Shared parse-stack pool reduces allocations by >50%
2. Arena allocation for parse tree nodes implemented
3. Memory usage reduced by >30% on large files
4. Performance within 2x of Tree-sitter C
5. No performance regressions on small files

**Deliverables**:
- [ ] Parse-stack pooling implementation
- [ ] Arena allocator integration
- [ ] Performance benchmarks showing improvements
- [ ] Memory usage comparison charts
- [ ] Updated PERFORMANCE_BASELINE.md

**BDD Scenario**:
```gherkin
Scenario: Optimize memory allocations
  Given the arena allocator implementation
  When parsing a 10K line Python file
  Then allocations decrease by >50%
  And parsing time is within 2x of Tree-sitter C
  And memory usage is <10x input size
```

---

## IV. Incremental GLR Parsing (Weeks 5-15) - 100% PLANNED

**Status**: 📋 100% PLANNED (2,478 lines specs, ready for 11-week implementation)
**Contract**: [GLR_INCREMENTAL_CONTRACT.md](../specs/GLR_INCREMENTAL_CONTRACT.md)
**ADR**: [ADR-0009-INCREMENTAL-PARSING-ARCHITECTURE.md](../adr/ADR-0009-INCREMENTAL-PARSING-ARCHITECTURE.md)
**BDD Scenarios**: [BDD_INCREMENTAL_PARSING.md](./BDD_INCREMENTAL_PARSING.md) (32 scenarios)
**Summary**: [INCREMENTAL_PLANNING_SUMMARY.md](../INCREMENTAL_PLANNING_SUMMARY.md)

### Goal: Close competitive gap with Tree-sitter via editor-class performance

**Strategic Context**:
- **Before**: Full parse every edit (unacceptable for editors)
- **After**: ≤30% of full parse for single-line edits, ≥70% subtree reuse
- **Market Impact**: Positions rust-sitter as credible Tree-sitter alternative

**Acceptance Criteria** (5 ACs, fully specified):

1. **AC-I1: API Surface**
   - Edit model (Tree-sitter compatible)
   - `Tree::edit(&mut self, edit: &Edit)` - mark dirty regions
   - `Parser::parse_incremental(input, old_tree)` - reuse clean subtrees
   - Stable node IDs (structural anchors, not persistent)
   - API documentation with examples
   - BDD scenarios: 8 scenarios

2. **AC-I2: Correctness**
   - Golden test suite (100+ cases): incremental == full parse
   - Property-based testing (quickcheck)
   - Ambiguity preservation in GLR mode
   - Edge case coverage (empty, boundary, large edits)
   - Corpus testing (Python, JavaScript, Rust grammars)
   - BDD scenarios: 10 scenarios

3. **AC-I3: Performance**
   - Single-line edit: ≤30% of full parse cost
   - Multi-line edit (≤10 lines): ≤50% of full parse cost
   - Reuse percentage: ≥70% for small edits
   - Automatic fallback for large edits (>50% of file)
   - CI regression gates (5% threshold)
   - BDD scenarios: 6 scenarios

4. **AC-I4: Forest API v1** (feature-gated)
   - `#[cfg(feature = "forest-api")]` ForestHandle
   - Ambiguity count reporting
   - Forest traversal (root alternatives, children, kind)
   - Graphviz export for visualization
   - Alternative tree resolution
   - BDD scenarios: 5 scenarios

5. **AC-I5: Observability & Documentation**
   - Metrics tracking (parse mode, reuse %, time)
   - Performance logging (env var gated: `RUST_SITTER_LOG_PERFORMANCE`)
   - Architecture document (INCREMENTAL_GLR_ARCHITECTURE.md)
   - User guide (incremental parsing section)
   - Forest API cookbook
   - BDD scenarios: 3 scenarios

**Architecture Decision: Local Reparse Window**
- **Strategy**: Pragmatic, sound under-approximation
- **Core Algorithm**:
  1. Dirty region detection (LCA finding, O(log n))
  2. Reparse window calculation (expand by N tokens, find stable anchors)
  3. Boundary stitching (stitch new subtree between anchors)
  4. Fallback mechanism (triggers: edit >50% file, window >20% file)

**Data Model**:
```rust
pub struct Edit {
    pub start_byte: u32,
    pub old_end_byte: u32,
    pub new_end_byte: u32,
    pub start_position: Point,
    pub old_end_position: Point,
    pub new_end_position: Point,
}

pub struct NodeAnchor {
    symbol: SymbolId,
    byte_offset: u32,
    path: Vec<usize>, // Root-to-node path
}
```

**Forest API** (feature-gated):
```rust
#[cfg(feature = "forest-api")]
pub struct ForestHandle {
    forest: Arc<Forest>,
}

impl ForestHandle {
    pub fn ambiguity_count(&self) -> usize;
    pub fn root_alternatives(&self) -> impl Iterator<Item = ForestNodeId>;
    pub fn to_graphviz(&self) -> String;
    pub fn resolve_alternative(&self, id: ForestNodeId) -> Tree;
}
```

**Implementation Plan** (11 weeks):

**Phase I: Foundations** (Weeks 5-6)
- Edit model implementation
- Stable node IDs/anchors
- Dirty region detection
- BDD specs & golden tests

**Phase II: Engine** (Weeks 7-8)
- Reparse window strategy
- Boundary stitching
- Fallback logic
- Performance benchmarks

**Phase III: Forest API** (Weeks 9-10)
- ForestHandle wrapper
- Ambiguity introspection
- Graphviz export
- Alternative resolution

**Phase IV: Documentation** (Week 11)
- Architecture document
- User guide
- Forest API cookbook
- Release preparation

**Performance Targets**:
| Edit Size | Target Time | Target Reuse | Strategy |
|-----------|-------------|--------------|----------|
| 1 line    | ≤30% of full | ≥70% | Local window |
| 2-10 lines | ≤50% of full | ≥50% | Local window |
| >50% file | Full parse | 0% | Automatic fallback |

**Strategic Impact**:
- **Competitive Position**: Credible Tree-sitter alternative (editor-class performance)
- **Market Opportunity**: LSPs, linters, formatters now viable
- **Unique Feature**: Forest API (programmatic ambiguity access, no competitor has this)
- **Greenfield Rust Tooling**: Default choice for "serious parsing"

**BDD Scenario** (example of 32 total):
```gherkin
Scenario: Single-line edit performance
  Given a 1000-line Python file
  And a full parse taking T_full milliseconds (baseline)
  When I change one character on line 500
  And I parse incrementally
  Then parse time < 0.3 × T_full
  And reuse percentage > 70%
  And no full parse fallback triggered
```

---

## V. Production Grammar Validation (Weeks 9-12)

### Goal: Validate three production grammars

**Reference**: ROADMAP_2025.md - Q1 2025 Grammar Support

### Phase 5A: Python Grammar (Week 9-10)

**Contract**: Full parity with tree-sitter-python

**Acceptance Criteria**:
1. All tree-sitter-python corpus tests pass
2. External scanner integration working
3. Performance within 2x of C implementation
4. Large Python files (>10K LOC) parse successfully
5. AST extraction for real Python code works

**Deliverables**:
- [ ] Python grammar integration
- [ ] Corpus test suite passing
- [ ] Performance benchmarks
- [ ] Example Python projects parsed
- [ ] Documentation and examples

### Phase 5B: JavaScript/TypeScript (Week 11)

**Contract**: JSX support with ambiguity handling

**Acceptance Criteria**:
1. JSX parsing works correctly
2. Ambiguity in JSX elements handled
3. TypeScript type annotations supported
4. Corpus tests passing
5. Performance acceptable

**Deliverables**:
- [ ] JavaScript/TypeScript grammar
- [ ] JSX support implementation
- [ ] Test suite
- [ ] Performance benchmarks
- [ ] Documentation

### Phase 5C: C++ Templates (Week 12)

**Contract**: Template disambiguation showcase

**Acceptance Criteria**:
1. C++ template parsing works
2. Template ambiguities handled correctly
3. Modern C++ features supported
4. Performance acceptable
5. Example C++ projects parse

**Deliverables**:
- [ ] C++ grammar with templates
- [ ] Template disambiguation logic
- [ ] Test suite with real C++ code
- [ ] Performance benchmarks
- [ ] Documentation and cookbook

---

## VI. Continuous Improvement Framework

### Metrics and Monitoring

**Key Performance Indicators (KPIs)**:
```yaml
Technical:
  - test_coverage: ">80%"
  - test_pass_rate: "100%"
  - performance_regression_threshold: "5%"
  - documentation_coverage: "100% of public APIs"

Quality:
  - clippy_warnings: "0"
  - security_vulnerabilities: "0 critical, 0 high"
  - code_review_approval: "required"

Community:
  - issue_response_time: "<48h"
  - pr_review_time: "<72h"
  - documentation_clarity_score: ">4.5/5"
```

### Governance

**Decision Making**:
1. **Architecture Decisions**: ADRs required for major changes
2. **API Changes**: Contract review before implementation
3. **Performance Changes**: Benchmark comparison required
4. **Documentation**: Review required before merge

**Review Process**:
1. All PRs require passing CI
2. Code review required for all changes
3. Documentation review for user-facing changes
4. Performance review for optimization PRs

---

## VII. Risk Management

### High Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Performance goals not met | HIGH | MEDIUM | Early profiling, incremental optimization |
| Nix learning curve | MEDIUM | HIGH | Good documentation, gradual adoption |
| Breaking API changes | HIGH | LOW | Contract-first, versioning, migration guides |
| Community adoption slow | MEDIUM | MEDIUM | Clear docs, responsive support, examples |

### Mitigation Strategies

**Technical Risks**:
- Automated performance regression detection
- Comprehensive test coverage
- Clear API contracts and versioning

**Project Risks**:
- Regular stakeholder communication
- Community engagement and support
- Clear roadmap and progress tracking

---

## VIII. Success Criteria

### Phase Completion

Each phase is **DONE** when:
1. ✅ All acceptance criteria met
2. ✅ All tests passing (100%)
3. ✅ Documentation complete
4. ✅ Code reviewed and merged
5. ✅ Performance benchmarks within targets
6. ✅ ADR/contract updated

### Overall Success

Project is **production-ready** when:
1. ✅ All 8 phases complete
2. ✅ GLR v1 contract 100% satisfied
3. ✅ Three production grammars validated
4. ✅ Performance within 2x of Tree-sitter C
5. ✅ Incremental parsing operational
6. ✅ Documentation complete and reviewed
7. ✅ Community feedback positive
8. ✅ Zero critical bugs

---

## IX. Timeline Summary

**Current Status** (2025-11-20):
```
✅ GLR v1: COMPLETE (144/144 tests, production-ready)
✅ Phase 1A: Nix CI Integration - 80% COMPLETE (AC-1,AC-4,AC-5 done)
📋 Phase 1B: Policy-as-Code - 100% PLANNED (2,058 lines specs)
📋 Phase II: Incremental Parsing - 100% PLANNED (2,478 lines specs)
```

**Implementation Roadmap**:
```
Week 1:     Nix Infrastructure (Phase 1A completion + verification)
Week 2:     Policy-as-Code (Phase 1B implementation, 5 days)
Weeks 3-4:  Performance Optimization (profiling, arena allocation)
Weeks 5-6:  Incremental Foundations (edit model, dirty detection)
Weeks 7-8:  Incremental Engine (reparse window, boundary stitching)
Weeks 9-10: Forest API (ambiguity introspection, Graphviz)
Week 11:    Incremental Documentation (architecture, user guide)
Weeks 12-15: Production Grammar Validation (Python, JS/TS, C++)

Total: 15 weeks to production-ready v0.9.0
       (Q1 2026 target)
```

**Planning Achievements**:
```
Total Specifications: 7,536 lines
├── Nix Documentation:     3,000 lines (3 guides)
├── Policy-as-Code:        2,058 lines (contract + ADR + 32 BDD)
└── Incremental Parsing:   2,478 lines (contract + ADR + 32 BDD)

BDD Scenarios: 64 total (32 Policy + 32 Incremental)
ADRs Created:  3 (ADR-0008 Nix, ADR-0009 Incremental, ADR-0010 Policy)
```

---

## X. References

**Phase I: Infrastructure**

*Nix CI Integration (Phase 1A - 80% complete)*:
- [NIX_CI_INTEGRATION_CONTRACT.md](../specs/NIX_CI_INTEGRATION_CONTRACT.md) - Contract with 5 ACs
- [ADR-0008-NIX-DEVELOPMENT-ENVIRONMENT.md](../adr/ADR-0008-NIX-DEVELOPMENT-ENVIRONMENT.md) - Architecture decision
- [NIX_QUICKSTART.md](../guides/NIX_QUICKSTART.md) - 5-minute setup guide (1,100+ lines)
- [NIX_TROUBLESHOOTING.md](../guides/NIX_TROUBLESHOOTING.md) - Problem-solving guide (1,600+ lines)
- [MIGRATING_TO_NIX.md](../guides/MIGRATING_TO_NIX.md) - Migration strategies (1,300+ lines)
- [PHASE_1A_COMPLETION_SUMMARY.md](../PHASE_1A_COMPLETION_SUMMARY.md) - Progress summary

*Policy-as-Code (Phase 1B - 100% planned)*:
- [POLICY_AS_CODE_CONTRACT.md](../specs/POLICY_AS_CODE_CONTRACT.md) - Contract with 5 ACs (1,016 lines)
- [ADR-0010-POLICY-AS-CODE.md](../adr/ADR-0010-POLICY-AS-CODE.md) - Layered enforcement architecture (667 lines)
- [BDD_POLICY_ENFORCEMENT.md](./BDD_POLICY_ENFORCEMENT.md) - 32 BDD scenarios (375 lines)
- [POLICY_PLANNING_SUMMARY.md](../POLICY_PLANNING_SUMMARY.md) - Comprehensive planning summary

**Phase II: Incremental Parsing (100% planned)**:
- [GLR_INCREMENTAL_CONTRACT.md](../specs/GLR_INCREMENTAL_CONTRACT.md) - Contract with 5 ACs (990 lines)
- [ADR-0009-INCREMENTAL-PARSING-ARCHITECTURE.md](../adr/ADR-0009-INCREMENTAL-PARSING-ARCHITECTURE.md) - Local reparse window strategy (667 lines)
- [BDD_INCREMENTAL_PARSING.md](./BDD_INCREMENTAL_PARSING.md) - 32 BDD scenarios (821 lines)
- [INCREMENTAL_PLANNING_SUMMARY.md](../INCREMENTAL_PLANNING_SUMMARY.md) - Comprehensive planning summary

**GLR v1 (Complete)**:
- [GLR_V1_COMPLETION_CONTRACT.md](../specs/GLR_V1_COMPLETION_CONTRACT.md) - Full contract
- [GLR_V1_COMPLETION_SUMMARY.md](../releases/GLR_V1_COMPLETION_SUMMARY.md) - Achievement summary
- [BDD_GLR_CONFLICT_PRESERVATION.md](./BDD_GLR_CONFLICT_PRESERVATION.md) - BDD scenarios
- [ADR-0007-RUNTIME2-GLR-INTEGRATION.md](../adr/ADR-0007-RUNTIME2-GLR-INTEGRATION.md) - Architecture

**Specifications**:
- [PARSETABLE_FILE_FORMAT_SPEC.md](../specs/PARSETABLE_FILE_FORMAT_SPEC.md) - Parse table format

**Status Reports**:
- [STATUS_REPORT_2025-11-20.md](../STATUS_REPORT_2025-11-20.md) - Overall project status
- [PROJECT_STATUS.md](../../PROJECT_STATUS.md) - High-level status

---

**Plan Version**: 2.0.0
**Last Updated**: 2025-11-20
**Next Review**: Weekly (every Monday)
**Owner**: rust-sitter core team

**Changelog**:
- **v2.0.0** (2025-11-20):
  - Added comprehensive Phase 1B (Policy-as-Code) planning (2,058 lines)
  - Added comprehensive Phase II (Incremental Parsing) planning (2,478 lines)
  - Updated Phase 1A status to 80% complete
  - Revised timeline to 15 weeks (Q1 2026 target)
  - Total specifications: 7,536 lines across 3 major phases
- **v1.0.0** (2025-11-20): Initial strategic plan with Nix infrastructure

---

END OF STRATEGIC PLAN
