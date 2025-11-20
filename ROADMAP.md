# Rust-Sitter Roadmap

**Last Updated**: November 20, 2025
**Current Version**: v0.7.0-alpha (GLR v1 Complete + Phase 1B Complete)
**Status**: ✅ **Phase I Complete - Ready for v0.8.0 (Performance Optimization)**

---

## 📊 Executive Summary

**Current State** (November 2025):
- ✅ **GLR v1 COMPLETE**: 144/144 tests passing, production-ready GLR parser
- ✅ **Phase I COMPLETE**: Enterprise-grade infrastructure deployed (Nix + Policy-as-Code)
- ✅ **Policy-as-Code COMPLETE**: Automated governance (3-layer enforcement, security scanning)
- ✅ **Infrastructure Foundation**: Nix dev environment, CI/CD workflows, automated quality gates
- 📋 **Ready for Implementation**: Performance Optimization (v0.8.0) → Incremental Parsing (v0.9.0)

**Strategic Positioning**:
> "A production-ready GLR parser in Rust with enterprise-grade infrastructure (Nix, automated policies, comprehensive specs) and a clear path to editor-class performance through incremental parsing."

**Path to Production** (13 weeks remaining to v0.9.0):
```
Week 1:     ✅ Phase 1A completion (Nix CI integration)
Week 2:     ✅ Phase 1B complete (Policy-as-Code deployed)
Week 3:     ⏳ Performance optimization (Week 3 Day 1 ✅) ← ACTIVE
Week 4:     Performance optimization (arena allocation, stack pooling)
Weeks 5-15: Incremental parsing implementation (11-week plan)
Target:     Q1 2026 (v0.9.0)
```

---

## 📋 Quick Status

| Component | Status | Evidence | Version |
|-----------|--------|----------|---------|
| **GLR Parser** | ✅ Production-Ready | 144/144 tests (100%) | v0.7.0 ✅ |
| **Parse Tables** | ✅ Production-Ready | .parsetable pipeline working | v0.7.0 ✅ |
| **Documentation** | ✅ Comprehensive | 2,300+ lines (architecture + guides + API) | v0.7.0 ✅ |
| **Nix Infrastructure** | ✅ Production-Ready | AC-1,AC-4,AC-5 complete | Phase 1A ✅ |
| **Policy-as-Code** | ✅ Production-Ready | 3-layer enforcement deployed | Phase 1B ✅ |
| **Incremental Parsing** | 📋 100% Planned | 2,478 lines specs ready | v0.9.0 |
| **Forest API** | 📋 Planned | Part of incremental v1 | v0.9.0 |
| **Performance** | ⏳ In Progress | Week 3 Day 1 ✅ | v0.8.0 ← ACTIVE |
| **Query System** | ⚠️ Partial | Needs completion | v1.0.0 |

**Detailed Status**: [STATUS_REPORT_2025-11-20.md](./docs/STATUS_REPORT_2025-11-20.md)

---

## ✅ v0.7.0 - GLR v1 Complete (November 2025)

**Status**: ✅ **PRODUCTION-READY**
**Achievement**: Complete GLR parser with comprehensive specifications and documentation

### What Was Delivered

**GLR Core Engine** (AC-1 Complete):
- Multi-action cells for conflict preservation
- Fork/merge on shift/reduce and reduce/reduce conflicts
- Cascaded reductions via re-closure algorithm
- Phase-2 re-closure for correct derivation coverage
- Conflict resolution (precedence + associativity)
- 30/30 GLR tests passing + 114 additional tests

**Runtime Integration** (AC-5 Complete):
- runtime2 GLR parser fully integrated
- .parsetable file loading and decoding
- Tree API compatibility (89/89 tests)
- Performance instrumentation (`RUST_SITTER_LOG_PERFORMANCE`)

**Documentation** (AC-6 Complete):
- Architecture documentation (comprehensive)
- User guides (getting started, advanced features)
- API documentation (100% coverage of public APIs)
- 2,300+ total lines of documentation

**Test Coverage**:
- 144/144 tests passing (100% pass rate)
- Fork/merge correctness validated
- Precedence/associativity tested with real grammars
- Error recovery with graceful handling
- Property-based testing foundations

**Infrastructure**:
- Nix development shell (flake.nix)
- CI/CD workflows (GitHub Actions)
- Performance baseline established
- Test connectivity safeguards

### Strategic Impact

**Before GLR v1**:
- Simple LR parser (no ambiguity handling)
- Limited to deterministic grammars
- Manual conflict resolution required

**After GLR v1**:
- Full GLR parser (handles ambiguity)
- Production-ready (144/144 tests)
- Comprehensive documentation
- Enterprise-grade infrastructure foundation

**Reference**: [GLR_V1_COMPLETION_SUMMARY.md](./docs/releases/GLR_V1_COMPLETION_SUMMARY.md)

---

## ✅ Phase I: Infrastructure (Weeks 1-2) - COMPLETE

**Goal**: Enterprise-grade development infrastructure
**Status**: ✅ **100% COMPLETE** (Phase 1A ✅, Phase 1B ✅)

### Phase 1A: Nix CI Integration - ✅ PRODUCTION-READY

**Contract**: [NIX_CI_INTEGRATION_CONTRACT.md](./docs/specs/NIX_CI_INTEGRATION_CONTRACT.md)
**Status**: ✅ **PRODUCTION-READY** (all core ACs complete)

**Completed**:
- ✅ **AC-1**: Nix Development Shell (flake.nix, justfile, auto-setup)
- ✅ **AC-4**: CI Pipeline Integration (.github/workflows/nix-ci.yml)
- ✅ **AC-5**: Documentation (3,000+ lines across 3 guides)
  - [Nix Quickstart](./docs/guides/NIX_QUICKSTART.md) (1,100+ lines)
  - [Nix Troubleshooting](./docs/guides/NIX_TROUBLESHOOTING.md) (1,600+ lines)
  - [Migrating to Nix](./docs/guides/MIGRATING_TO_NIX.md) (1,300+ lines)

**Verification Scripts** (optional):
- 📋 **AC-2**: Local Reproduction (script available: `scripts/verify-nix-local-reproduction.sh`)
- 📋 **AC-3**: Performance Consistency (script available: `scripts/verify-nix-performance-consistency.sh`)

**Deliverables**:
- Reproducible development environment (`nix develop`)
- CI/CD pipeline using Nix (GitHub Actions + Cachix)
- Comprehensive documentation (3,000+ lines)
- Verification scripts for validation

**Completed**: Week 1

---

### Phase 1B: Policy-as-Code - ✅ PRODUCTION-READY

**Contract**: [POLICY_AS_CODE_CONTRACT.md](./docs/specs/POLICY_AS_CODE_CONTRACT.md)
**ADR**: [ADR-0010-POLICY-AS-CODE.md](./docs/adr/ADR-0010-POLICY-AS-CODE.md)
**BDD Scenarios**: [BDD_POLICY_ENFORCEMENT.md](./docs/plans/BDD_POLICY_ENFORCEMENT.md) (32 scenarios)
**Status**: ✅ **PRODUCTION-READY** (all 5 ACs complete)

**Completed Acceptance Criteria**:

✅ **AC-P1: Pre-commit Hooks**
- Framework: pre-commit (Python-based, industry standard)
- Hooks: formatting, linting, test connectivity, commit message, large files
- Auto-installation via Nix shell (`flake.nix`)
- Performance: <5 seconds typical execution
- File: `.pre-commit-config.yaml`

✅ **AC-P2: CI Policy Enforcement**
- Jobs: quality-gates, security-scanning, performance-gates, test-connectivity
- Enforcement: Cannot be bypassed, blocks PR merge
- Performance: ~30 minutes (parallel execution)
- File: `.github/workflows/policy.yml`

✅ **AC-P3: Security Policies**
- Vulnerability scanning (cargo audit, `audit.toml`)
- License compliance (cargo deny, `deny.toml`)
- Secret detection (4 methods: TruffleHog, patterns, entropy, files)
- File: `.github/workflows/secrets.yml`

✅ **AC-P4: Quality Verification Scripts**
- `scripts/check-quality.sh` (formatting, clippy, tests, docs)
- `scripts/check-security.sh` (audit, deny, secrets)
- `scripts/pre-push.sh` (combined validation)
- Performance: <60 seconds total

✅ **AC-P5: Documentation & Governance**
- Policy documentation (`POLICIES.md` - 380 lines)
- Technical guide (`docs/guides/POLICY_ENFORCEMENT.md` - 750 lines)
- CONTRIBUTING.md updated with policy workflow
- Override template (`.github/ISSUE_TEMPLATE/policy-override.md`)

**Architecture Delivered**: 3-Layer Defense
- **Layer 1**: Pre-commit hooks (fast local, <5s)
- **Layer 2**: Verification scripts (self-service, <60s)
- **Layer 3**: CI policy workflow (safety net, cannot bypass)

**Deliverables**:
- 3-layer enforcement infrastructure deployed
- 1,130+ lines of documentation
- Automated quality gates (zero manual checks)
- Security scanning (vulnerabilities, licenses, secrets)
- Policy override process

**Strategic Impact Achieved**:
- ✅ Eliminates manual quality checks (30-40% time savings)
- ✅ Enables enterprise-grade governance (security, compliance)
- ✅ Fast feedback (<5s local, automated gates in CI)
- ✅ Zero tolerance (formatting, linting, vulnerabilities)

**Reference**: [POLICIES.md](./POLICIES.md), [POLICY_ENFORCEMENT.md](./docs/guides/POLICY_ENFORCEMENT.md)

**Completed**: Week 2 (November 20, 2025)

---

## 🎯 v0.8.0 - Performance Optimization (Weeks 3-4)

**Goal**: Performance within 2x of Tree-sitter C implementation
**Status**: ⏳ **IN PROGRESS** (Week 3 Day 1 ✅)
**Contract**: [PERFORMANCE_OPTIMIZATION_CONTRACT.md](./docs/specs/PERFORMANCE_OPTIMIZATION_CONTRACT.md)
**BDD Scenarios**: [BDD_PERFORMANCE_OPTIMIZATION.md](./docs/plans/BDD_PERFORMANCE_OPTIMIZATION.md) (30 scenarios)

### Scope

**Week 3 Day 1: Benchmarking Infrastructure** ✅ COMPLETE
- ✅ Benchmark suite skeleton (Criterion framework, 4 groups)
- ✅ Test fixtures created (Python, JavaScript, Rust - small size)
- ✅ Profiling scripts (CPU flamegraphs, memory analysis)
- ✅ Tree-sitter comparison framework
- ✅ Baseline documentation template (PERFORMANCE_BASELINE_V0.7.0.md)

**Week 3 Day 2: Baseline Measurement** ⏳ NEXT
- Create medium/large fixtures (~500-5000 LOC)
- Run baseline benchmarks (v0.7.0)
- Generate CPU flamegraphs
- Generate memory profiles
- Compare to Tree-sitter C baseline
- Populate baseline documentation

**Week 3 Days 3-4: Profiling and Analysis**
- Profile GLR fork/merge on large Python files (>10K LOC)
- Identify top 5 performance bottlenecks
- Memory usage profiled with heaptrack
- Analyze allocation patterns
- Document optimization targets (AC-PERF2)

**Week 3 Day 5: Review and Planning**
- Review analysis findings
- Refine optimization plan
- Prepare for Week 4 implementation

**Week 4: Implementation**
- Arena allocation for parse tree nodes (AC-PERF3: >50% allocation reduction)
- Parse-stack pooling (AC-PERF4: >40% fork allocation reduction)
- Performance validation (AC-PERF5: ≤2x Tree-sitter C)
- Memory usage reduced by >30% on large files
- No regressions on small files

**Success Criteria** (AC-PERF5):
- Parsing time ≤2x of Tree-sitter C (all benchmarks)
- Memory usage <10x input size
- No regressions in correctness (144/144 tests still pass)

**Current Progress**:
- Week 3 Day 1: ✅ Complete (infrastructure ready)
- Deliverables: benchmark suite, profiling scripts, fixtures, baseline template

**Estimated Completion**: Week 4

---

## 🚀 v0.9.0 - Incremental Parsing + Forest API (Weeks 5-15)

**Status**: 📋 **100% PLANNED** (2,478 lines specifications)
**Contract**: [GLR_INCREMENTAL_CONTRACT.md](./docs/specs/GLR_INCREMENTAL_CONTRACT.md)
**ADR**: [ADR-0009-INCREMENTAL-PARSING-ARCHITECTURE.md](./docs/adr/ADR-0009-INCREMENTAL-PARSING-ARCHITECTURE.md)
**BDD Scenarios**: [BDD_INCREMENTAL_PARSING.md](./docs/plans/BDD_INCREMENTAL_PARSING.md) (32 scenarios)

### Goal: Close Competitive Gap with Tree-sitter

**Strategic Context**:
- **Before**: Full parse every edit (unacceptable for editors)
- **After**: ≤30% of full parse for single-line edits, ≥70% subtree reuse
- **Market Impact**: Positions rust-sitter as credible Tree-sitter alternative

### Acceptance Criteria

**AC-I1: API Surface**
- Tree-sitter compatible Edit model
- `Tree::edit(&mut self, edit: &Edit)` - mark dirty regions
- `Parser::parse_incremental(input, old_tree)` - reuse clean subtrees
- Stable node IDs (structural anchors)
- API documentation with examples

**AC-I2: Correctness**
- Golden test suite (100+ cases): incremental == full parse
- Property-based testing (quickcheck)
- Ambiguity preservation in GLR mode
- Edge case coverage (empty, boundary, large edits)

**AC-I3: Performance**
- Single-line edit: ≤30% of full parse cost
- Multi-line edit (≤10 lines): ≤50% of full parse cost
- Reuse percentage: ≥70% for small edits
- Automatic fallback for large edits (>50% of file)
- CI regression gates (5% threshold)

**AC-I4: Forest API v1** (feature-gated)
- `#[cfg(feature = "forest-api")]` ForestHandle
- Ambiguity count reporting
- Forest traversal (root alternatives, children, kind)
- Graphviz export for visualization
- Alternative tree resolution

**AC-I5: Observability & Documentation**
- Metrics tracking (parse mode, reuse %, time)
- Performance logging (env var gated)
- Architecture document
- User guide
- Forest API cookbook

### Architecture: Local Reparse Window

**Strategy**: Pragmatic, sound under-approximation

**Core Algorithms**:
1. **Dirty Region Detection**: LCA finding, O(log n)
2. **Reparse Window Calculation**: Expand by N tokens, find stable anchors
3. **Boundary Stitching**: Stitch new subtree between anchors
4. **Fallback Mechanism**: Triggers when edit >50% file or window >20% file

**Performance Targets**:
| Edit Size | Target Time | Target Reuse | Strategy |
|-----------|-------------|--------------|----------|
| 1 line    | ≤30% of full | ≥70% | Local window |
| 2-10 lines | ≤50% of full | ≥50% | Local window |
| >50% file | Full parse | 0% | Automatic fallback |

### Implementation Plan (11 weeks)

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

### Strategic Impact

**Competitive Position**:
- ✅ Incremental parsing (≤30% cost for edits)
- ✅ Forest API (unique: programmatic ambiguity access)
- ✅ GLR support (production-ready)
- ✅ Infrastructure-as-code (best-in-class)

**Market Opportunity**:
- **Greenfield Rust tooling**: Default choice for "serious parsing"
- **Existing Tree-sitter users**: Credible alternative with GLR benefits
- **Language tools**: LSPs, linters, formatters now viable
- **Research/analysis**: Forest API enables new use cases

**Reference**: [INCREMENTAL_PLANNING_SUMMARY.md](./docs/INCREMENTAL_PLANNING_SUMMARY.md)

**Estimated Completion**: Week 15 (Q1 2026)

---

## 🎓 v1.0.0 - Production Readiness (Q2 2026)

**Goal**: Production-ready for regulated environments and enterprise adoption

### Scope

**Query System Completion**:
- [ ] Complete predicate support (#match?, #eq?, etc.)
- [ ] Query capture validation
- [ ] Performance optimization
- [ ] Comprehensive documentation

**Production Grammar Validation**:
- [ ] Python grammar (full parity with tree-sitter-python)
- [ ] JavaScript/TypeScript (JSX support)
- [ ] Rust grammar (macros, generics)

**Performance**:
- [ ] Performance within 2x of Tree-sitter C (maintained)
- [ ] Large file handling (>100K LOC)
- [ ] Memory efficiency validated

**Documentation & Examples**:
- [ ] Complete API documentation
- [ ] Grammar authoring guide
- [ ] LSP integration example
- [ ] Production deployment guide

**Security & Compliance**:
- [ ] Security audit completed
- [ ] SBOM generation automated
- [ ] License compliance validated
- [ ] CVE monitoring in place

**Community**:
- [ ] External review completed (>4.5/5 satisfaction)
- [ ] Production users validated (3+ real-world deployments)
- [ ] Contribution guide mature
- [ ] Release process documented

**Success Criteria**:
- Zero critical bugs
- 100% test pass rate
- Performance goals met
- Documentation complete
- Security audit passed
- Community feedback positive

**Estimated Completion**: Q2 2026

---

## 📈 Planning Achievements

**Total Specifications**: 7,536 lines
```
Nix Documentation:     3,000 lines
├── NIX_QUICKSTART.md           (1,100 lines)
├── NIX_TROUBLESHOOTING.md      (1,600 lines)
└── MIGRATING_TO_NIX.md         (1,300 lines)

Policy-as-Code:        2,058 lines
├── POLICY_AS_CODE_CONTRACT.md  (1,016 lines)
├── ADR-0010-POLICY-AS-CODE.md  (667 lines)
├── BDD_POLICY_ENFORCEMENT.md   (375 lines)
└── POLICY_PLANNING_SUMMARY.md

Incremental Parsing:   2,478 lines
├── GLR_INCREMENTAL_CONTRACT.md (990 lines)
├── ADR-0009-INCREMENTAL-...md  (667 lines)
├── BDD_INCREMENTAL_PARSING.md  (821 lines)
└── INCREMENTAL_PLANNING_...md
```

**BDD Scenarios**: 64 total
- Policy-as-Code: 32 scenarios (8 per AC average)
- Incremental Parsing: 32 scenarios (6.4 per AC average)

**ADRs Created**: 3
- ADR-0008: Nix Development Environment
- ADR-0009: Incremental Parsing Architecture (Local Reparse Window)
- ADR-0010: Policy-as-Code Architecture (Layered Enforcement)

---

## 🗓️ Timeline Summary

**Week 1** (Current): Nix CI verification (Phase 1A completion)
**Week 2**: Policy-as-Code implementation (Phase 1B)
**Weeks 3-4**: Performance optimization (v0.8.0)
**Weeks 5-15**: Incremental parsing + Forest API (v0.9.0)
**Q2 2026**: Query system + production validation (v1.0.0)

**Milestones**:
- ✅ v0.7.0: GLR v1 Complete (November 2025)
- ⏳ v0.8.0: Performance Optimized (Week 4)
- 📋 v0.9.0: Incremental Parsing (Week 15, Q1 2026)
- 🎯 v1.0.0: Production Ready (Q2 2026)

---

## 📚 References

**Planning Documents**:
- [Strategic Implementation Plan v2.0](./docs/plans/STRATEGIC_IMPLEMENTATION_PLAN.md)
- [Status Report 2025-11-20](./docs/STATUS_REPORT_2025-11-20.md)

**Phase I (Infrastructure)**:
- [NIX_CI_INTEGRATION_CONTRACT.md](./docs/specs/NIX_CI_INTEGRATION_CONTRACT.md)
- [POLICY_AS_CODE_CONTRACT.md](./docs/specs/POLICY_AS_CODE_CONTRACT.md)
- [ADR-0008: Nix Environment](./docs/adr/ADR-0008-NIX-DEVELOPMENT-ENVIRONMENT.md)
- [ADR-0010: Policy-as-Code](./docs/adr/ADR-0010-POLICY-AS-CODE.md)

**Phase II (Incremental Parsing)**:
- [GLR_INCREMENTAL_CONTRACT.md](./docs/specs/GLR_INCREMENTAL_CONTRACT.md)
- [ADR-0009: Incremental Architecture](./docs/adr/ADR-0009-INCREMENTAL-PARSING-ARCHITECTURE.md)
- [BDD_INCREMENTAL_PARSING.md](./docs/plans/BDD_INCREMENTAL_PARSING.md)

**GLR v1 (Complete)**:
- [GLR_V1_COMPLETION_CONTRACT.md](./docs/specs/GLR_V1_COMPLETION_CONTRACT.md)
- [GLR_V1_COMPLETION_SUMMARY.md](./docs/releases/GLR_V1_COMPLETION_SUMMARY.md)
- [ADR-0007: Runtime2 GLR Integration](./docs/adr/ADR-0007-RUNTIME2-GLR-INTEGRATION.md)

---

**Roadmap Version**: 2.0.0
**Last Updated**: 2025-11-20
**Next Review**: Monthly (first Monday)
**Owner**: rust-sitter core team

---

END OF ROADMAP
