# Strategic Implementation Plan: Production-Ready rust-sitter

**Version**: 1.0.0
**Date**: 2025-11-20
**Status**: ACTIVE
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
- ✅ GLR v1 core complete (93/93 tests passing, performance baseline established)
- ✅ Runtime2 + .parsetable pipeline working
- ✅ Comprehensive roadmaps and contracts in place
- ⚠️ Development environment not reproducible across machines
- ⚠️ GLR v1 documentation incomplete (3/7 deliverables)
- ⚠️ CI runs in GitHub-managed environment (not local-reproducible)

**Target State** (Q1 2025):
- ✅ Nix-based dev shell = CI environment
- ✅ GLR v1 complete with full documentation
- ✅ Performance within 2x of Tree-sitter C implementation
- ✅ Three production grammars validated (Python, JavaScript, C++)
- ✅ Incremental GLR operational

---

## I. Foundational Infrastructure (Weeks 1-2)

### Goal: Establish reproducible development environment

**User Story**:
```gherkin
As a contributor
I want to run exactly the same toolchain as CI
So that "works on my machine" becomes "works everywhere"
```

### Phase 1A: Nix Development Shell (Week 1)

**Contract**: [ADR-0008-NIX-DEVELOPMENT-ENVIRONMENT.md](../adr/ADR-0008-NIX-DEVELOPMENT-ENVIRONMENT.md)

**Acceptance Criteria**:
1. Single `flake.nix` defines all dependencies
2. `nix develop` provides identical environment to CI
3. All Rust toolchain components (rustc, cargo, clippy, rustfmt) pinned
4. All system dependencies (libtree-sitter-dev, libclang) included
5. Environment variables (RUST_TEST_THREADS, etc.) set correctly

**Deliverables**:
- [ ] `flake.nix` at repository root
- [ ] `justfile` with CI commands (`just ci-all`, `just ci-perf`)
- [ ] `.github/workflows/ci.yml` updated to use Nix
- [ ] `docs/dev-workflow.md` updated with Nix instructions
- [ ] ADR documenting Nix adoption rationale

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

### Phase 1B: Policy-as-Code (Week 2)

**Contract**: Automated enforcement of quality standards

**Acceptance Criteria**:
1. Pre-commit hooks enforce formatting and linting
2. CI blocks merges on test failures
3. Performance regression gates active (5% threshold)
4. Test connectivity guards prevent silent test disconnections
5. Security scanning for dependencies

**Deliverables**:
- [ ] `.github/workflows/policy.yml` - Policy enforcement
- [ ] `.pre-commit-config.yaml` - Local quality gates
- [ ] `scripts/check-quality.sh` - Quality verification script
- [ ] ADR documenting policy decisions

**BDD Scenario**:
```gherkin
Scenario: Quality gates prevent bad merges
  Given a pull request with failing tests
  When CI runs
  Then the merge is blocked
  And the PR shows clear failure reasons
```

---

## II. GLR v1 Completion (Weeks 3-4)

### Goal: Complete all GLR v1 acceptance criteria

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

## IV. Incremental GLR Parsing (Weeks 7-8)

### Goal: Enable incremental reparsing with GLR

**Reference**: ROADMAP_2025.md - Week 2 Incremental Parsing

### Phase 4A: Incremental Algorithm Design (Week 7)

**Contract**: Design GLR-aware incremental parsing

**Acceptance Criteria**:
1. Algorithm preserves ambiguity across edits
2. Fork tracking allows selective reparse
3. Design handles typical edit patterns (<100ms reparse)
4. Edge cases documented (edit within ambiguous region)
5. Algorithm validated with toy grammars

**Deliverables**:
- [ ] `docs/specs/INCREMENTAL_GLR_ALGORITHM.md`
- [ ] Design review and approval
- [ ] Prototype with arithmetic grammar
- [ ] Performance model and predictions
- [ ] ADR documenting design decisions

### Phase 4B: Implementation and Testing (Week 8)

**Contract**: Working incremental GLR

**Acceptance Criteria**:
1. `glr_incremental::reparse()` function works
2. Typical edits reparse in <100ms
3. Ambiguity preserved correctly
4. Unit tests pass for all edit patterns
5. Integration with Tree::edit() API

**Deliverables**:
- [ ] Incremental GLR implementation
- [ ] Comprehensive test suite
- [ ] Performance benchmarks
- [ ] Documentation and examples
- [ ] Updated API documentation

**BDD Scenario**:
```gherkin
Scenario: Incremental reparse preserves ambiguity
  Given a tree from an ambiguous grammar
  When editing a single character
  And calling tree.edit()
  And reparsing
  Then only affected subtrees are reparsed
  And ambiguity is preserved
  And reparse completes in <100ms
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

```
Week 1-2:   Nix Infrastructure + Policy-as-Code
Week 3-4:   GLR v1 Completion (Tokenization, Tree API, Docs)
Week 5-6:   Performance Optimization
Week 7-8:   Incremental GLR Parsing
Week 9-10:  Python Grammar Validation
Week 11:    JavaScript/TypeScript Grammar
Week 12:    C++ Grammar

Total: 12 weeks to production-ready v0.7.0
```

---

## X. References

**Contracts and Specs**:
- [GLR_V1_COMPLETION_CONTRACT.md](../specs/GLR_V1_COMPLETION_CONTRACT.md)
- [BDD_GLR_CONFLICT_PRESERVATION.md](./BDD_GLR_CONFLICT_PRESERVATION.md)
- [PARSETABLE_FILE_FORMAT_SPEC.md](../specs/PARSETABLE_FILE_FORMAT_SPEC.md)

**Roadmaps**:
- [ROADMAP_2025.md](../roadmaps/ROADMAP_2025.md)
- [CONCRETE_NEXT_STEPS.md](../roadmaps/CONCRETE_NEXT_STEPS.md)

**ADRs**:
- [ADR-0007-RUNTIME2-GLR-INTEGRATION.md](../adr/ADR-0007-RUNTIME2-GLR-INTEGRATION.md)

**Status**:
- [PROJECT_STATUS.md](../../PROJECT_STATUS.md)
- [GLR_V1_COMPLETION_CONTRACT.md](../specs/GLR_V1_COMPLETION_CONTRACT.md)

---

**Plan Version**: 1.0.0
**Last Updated**: 2025-11-20
**Next Review**: Weekly (every Monday)
**Owner**: rust-sitter core team

---

END OF STRATEGIC PLAN
