# Performance Optimization Contract (v0.8.0)

**Version**: 1.0.0
**Status**: READY FOR IMPLEMENTATION
**Target**: v0.8.0
**Duration**: Weeks 3-4 (2 weeks)
**Methodology**: Contract-first, BDD/TDD, Performance-Driven Development
**Created**: November 20, 2025

---

## Executive Summary

This contract defines the acceptance criteria, deliverables, and success metrics for v0.8.0 (Performance Optimization). The goal is to achieve performance within 2x of Tree-sitter C implementation through profiling, analysis, and memory optimization.

**Strategic Goal**: Close the performance gap with Tree-sitter to position rust-sitter as a viable production alternative for editor-class parsing.

**Approach**: Data-driven optimization
1. **Week 3**: Profile, measure, analyze, identify bottlenecks
2. **Week 4**: Implement optimizations (arena allocation), validate improvements

---

## Acceptance Criteria

### AC-PERF1: Profiling Infrastructure

**Goal**: Establish comprehensive profiling and benchmarking infrastructure

**Requirements**:

1. **CPU Profiling**:
   - Flamegraph generation for GLR fork/merge operations
   - Identify top 5 performance hotspots (function-level)
   - Profiling on large Python files (>10K LOC)
   - Tool: `cargo flamegraph` or `perf`

2. **Memory Profiling**:
   - Heap profiling with heaptrack or valgrind massif
   - Allocation tracking (count, size, lifetime)
   - Identify memory hotspots (allocators, data structures)
   - Peak memory usage measurement

3. **Benchmark Suite**:
   - Comprehensive benchmarks for GLR parsing
   - Test cases: small (<100 LOC), medium (1K-10K LOC), large (>10K LOC)
   - Language coverage: Python, JavaScript, Rust
   - Baseline measurements documented

4. **Comparison Framework**:
   - Tree-sitter C baseline benchmarks
   - Apples-to-apples comparison (same input, same grammar)
   - Ratio calculation (rust-sitter time / tree-sitter time)
   - Memory comparison (rust-sitter peak / tree-sitter peak)

**Deliverables**:
- [ ] Profiling scripts (`scripts/profile-cpu.sh`, `scripts/profile-memory.sh`)
- [ ] Benchmark suite (`benches/glr-performance.rs`)
- [ ] Baseline measurements document (`docs/baselines/PERFORMANCE_BASELINE_V0.7.0.md`)
- [ ] Tree-sitter comparison script (`scripts/compare-tree-sitter.sh`)

**Success Criteria**:
- Can profile any grammar/input combination
- Top 5 bottlenecks identified with percentage breakdown
- Baseline performance documented (v0.7.0)
- Tree-sitter comparison automated

**BDD Scenarios**: 6 scenarios (profiling, benchmarking, comparison)

---

### AC-PERF2: Performance Analysis & Optimization Plan

**Goal**: Analyze profiling data and create evidence-based optimization plan

**Requirements**:

1. **Bottleneck Analysis**:
   - Document top 5 performance bottlenecks
   - Percentage of total time per bottleneck
   - Root cause analysis (why is it slow?)
   - Optimization opportunities identified

2. **Memory Analysis**:
   - Identify allocation hotspots (>1% of total allocations)
   - Analyze allocation patterns (short-lived vs. long-lived)
   - Identify unnecessary copies/clones
   - Calculate potential savings (arena allocation, object pooling)

3. **Optimization Plan**:
   - Prioritized list of optimizations (highest impact first)
   - Expected performance gain per optimization
   - Implementation complexity assessment
   - Risk assessment (correctness impact)

4. **Acceptance Criteria per Optimization**:
   - Clear success metric (e.g., "reduce allocations by 50%")
   - No correctness regressions (all tests pass)
   - Benchmark improvement documented

**Deliverables**:
- [ ] Performance analysis document (`docs/analysis/PERFORMANCE_ANALYSIS_V0.7.0.md`)
- [ ] Optimization plan (`docs/plans/PERFORMANCE_OPTIMIZATION_PLAN.md`)
- [ ] Risk assessment per optimization

**Success Criteria**:
- Top 5 bottlenecks documented with root causes
- Optimization plan prioritized by impact
- Expected performance gains estimated
- Risk mitigation strategies defined

**BDD Scenarios**: 4 scenarios (analysis, planning, prioritization)

---

### AC-PERF3: Arena Allocation for Parse Trees

**Goal**: Implement arena allocation for parse tree nodes to reduce allocations

**Requirements**:

1. **Arena Allocator**:
   - Custom arena allocator for parse tree nodes
   - Bump allocation strategy (fast, contiguous memory)
   - Per-parse arena (lifetime matches parse duration)
   - Zero-copy node creation

2. **Integration**:
   - Replace Box<Node> with arena-allocated nodes
   - Update Tree API to use arena references
   - Ensure correct lifetimes (arena outlives tree)
   - No API breaking changes (internal optimization)

3. **Memory Reduction**:
   - Measure allocation count before/after
   - Target: >50% reduction in allocations
   - Target: >30% reduction in peak memory usage (large files)
   - No memory leaks (valgrind clean)

4. **Performance Impact**:
   - Benchmark parsing time before/after
   - Target: ≥10% improvement in parsing time
   - No regressions on small files (<100 LOC)
   - Improvements visible on large files (>10K LOC)

**Deliverables**:
- [ ] Arena allocator implementation (`runtime2/src/arena.rs`)
- [ ] Tree API updates (use arena references)
- [ ] Allocation measurements (before/after)
- [ ] Benchmark comparison (v0.7.0 vs v0.8.0)

**Success Criteria**:
- ≥50% reduction in allocations
- ≥30% reduction in peak memory (large files)
- ≥10% improvement in parsing time
- All tests pass (144/144, no regressions)

**BDD Scenarios**: 8 scenarios (allocation, integration, performance, correctness)

---

### AC-PERF4: Parse-Stack Pooling

**Goal**: Implement object pooling for parse stacks to reduce GLR fork overhead

**Requirements**:

1. **Parse-Stack Pool**:
   - Reusable parse-stack pool (reduce allocations)
   - Thread-local or per-parser pool
   - Configurable pool size (default: 32 stacks)
   - Automatic return-to-pool on parse completion

2. **GLR Fork Optimization**:
   - Reuse stacks from pool during fork/merge
   - Measure fork/merge overhead before/after
   - Target: ≥40% reduction in fork allocations
   - No correctness impact (same parse results)

3. **Memory Management**:
   - Pool memory bounded (max size limit)
   - Automatic pool cleanup on parser drop
   - No memory leaks (valgrind clean)
   - Memory overhead <5% (pool vs. allocate-on-demand)

4. **Performance Impact**:
   - Benchmark GLR fork-heavy grammars (Python, ambiguous)
   - Target: ≥15% improvement on fork-heavy workloads
   - Minimal impact on deterministic grammars (<5% overhead)

**Deliverables**:
- [ ] Parse-stack pool implementation (`glr-core/src/stack_pool.rs`)
- [ ] GLR engine integration (use pool during fork/merge)
- [ ] Fork allocation measurements (before/after)
- [ ] Benchmark comparison (fork-heavy vs. deterministic)

**Success Criteria**:
- ≥40% reduction in fork allocations
- ≥15% improvement on fork-heavy workloads
- <5% overhead on deterministic grammars
- All tests pass (144/144, no regressions)

**BDD Scenarios**: 7 scenarios (pooling, fork optimization, memory management)

---

### AC-PERF5: Performance Validation & Documentation

**Goal**: Validate performance improvements and document optimization results

**Requirements**:

1. **Performance Validation**:
   - Benchmark suite run on v0.8.0
   - Comparison with v0.7.0 baseline
   - Comparison with Tree-sitter C implementation
   - Ratio calculation (rust-sitter / tree-sitter)

2. **Success Criteria Met**:
   - Parsing time within 2x of Tree-sitter C (all benchmarks)
   - Memory usage <10x input size (all benchmarks)
   - No regressions in correctness (144/144 tests pass)
   - Improvements documented per optimization

3. **Documentation**:
   - Performance report (`docs/reports/PERFORMANCE_REPORT_V0.8.0.md`)
   - Optimization summary (what was done, impact)
   - Benchmark results (before/after tables, charts)
   - Tree-sitter comparison (ratio tables)

4. **CI Integration**:
   - Performance regression gates updated (new baseline)
   - Benchmark suite runs in CI
   - Alerts on performance regressions (>5% slowdown)

**Deliverables**:
- [ ] Performance report (v0.8.0)
- [ ] Benchmark results (before/after comparison)
- [ ] Tree-sitter comparison (ratio tables)
- [ ] Updated performance gates in CI

**Success Criteria**:
- Parsing time ≤2x Tree-sitter C (met on all benchmarks)
- Memory usage <10x input size (met on all benchmarks)
- All tests pass (144/144, 100% pass rate)
- Performance improvements documented

**BDD Scenarios**: 5 scenarios (validation, comparison, documentation, CI)

---

## Implementation Plan (2 Weeks)

### Week 3: Profiling and Analysis

**Days 1-2: Profiling Infrastructure (AC-PERF1)**
- Implement profiling scripts (CPU, memory)
- Create benchmark suite (small, medium, large files)
- Establish Tree-sitter comparison framework
- Document baseline measurements (v0.7.0)

**Days 3-4: Performance Analysis (AC-PERF2)**
- Run profiling on representative workloads
- Analyze top 5 bottlenecks (CPU, memory)
- Create optimization plan (prioritized by impact)
- Document expected gains and risks

**Day 5: Review and Refinement**
- Review analysis with stakeholders
- Refine optimization plan based on feedback
- Prepare for implementation week

### Week 4: Implementation and Validation

**Days 1-3: Arena Allocation (AC-PERF3)**
- Implement arena allocator
- Integrate with Tree API
- Measure allocation reduction
- Validate correctness (all tests pass)

**Days 4-5: Parse-Stack Pooling (AC-PERF4)**
- Implement parse-stack pool
- Integrate with GLR engine
- Measure fork allocation reduction
- Validate correctness (all tests pass)

**Day 6: Validation & Documentation (AC-PERF5)**
- Run full benchmark suite (v0.8.0)
- Compare with v0.7.0 baseline
- Compare with Tree-sitter C
- Document results and update CI

---

## Success Metrics

### Primary Goals (MUST achieve)

1. **Parsing Performance**: ≤2x Tree-sitter C on all benchmarks
   - Small files (<100 LOC): ≤2x
   - Medium files (1K-10K LOC): ≤2x
   - Large files (>10K LOC): ≤2x

2. **Memory Usage**: <10x input size on all benchmarks
   - Python (10K LOC): <10x
   - JavaScript (5K LOC): <10x
   - Rust (3K LOC): <10x

3. **Correctness**: 100% test pass rate (144/144 tests)
   - No regressions introduced
   - All existing functionality preserved

### Secondary Goals (SHOULD achieve)

1. **Allocation Reduction**: ≥50% fewer allocations (arena allocation)
2. **Fork Optimization**: ≥40% fewer fork allocations (stack pooling)
3. **Parsing Speed**: ≥20% faster on large files (combined optimizations)
4. **Memory Peak**: ≥30% lower peak memory (large files)

### Stretch Goals (NICE to have)

1. **Sub-1.5x Performance**: Parsing time within 1.5x of Tree-sitter C
2. **Zero-Copy Parsing**: Eliminate unnecessary copies in hot paths
3. **SIMD Optimizations**: Vectorize token scanning (if applicable)

---

## Risk Assessment & Mitigation

### Risk 1: Correctness Regressions

**Risk**: Arena allocation or stack pooling introduces bugs
**Impact**: High (breaks parser correctness)
**Probability**: Medium
**Mitigation**:
- Comprehensive testing (144 existing tests + new property tests)
- Incremental implementation (arena first, then pooling)
- Rollback strategy (revert if tests fail)

### Risk 2: Performance Improvements Not Sufficient

**Risk**: Optimizations don't achieve 2x target
**Impact**: Medium (delays v0.9.0)
**Probability**: Low
**Mitigation**:
- Data-driven optimization (profile first)
- Multiple optimization strategies (arena + pooling)
- Fallback optimizations (zero-copy, SIMD) if needed

### Risk 3: API Breaking Changes

**Risk**: Arena allocation requires API changes
**Impact**: Medium (breaks user code)
**Probability**: Low
**Mitigation**:
- Keep internal (no public API changes)
- Use lifetimes carefully (arena outlives tree)
- Test with example grammars

### Risk 4: Memory Safety Issues

**Risk**: Arena lifetimes introduce unsoundness
**Impact**: High (unsafe code)
**Probability**: Low
**Mitigation**:
- Use Rust's lifetime system (borrow checker)
- Miri testing (undefined behavior detection)
- Valgrind validation (memory leaks, invalid access)

---

## BDD Scenario Summary

**Total Scenarios**: 30 scenarios across 5 ACs

**AC-PERF1: Profiling Infrastructure** (6 scenarios)
- Scenario: Profile GLR parsing on large file
- Scenario: Generate flamegraph for CPU hotspots
- Scenario: Profile memory allocations with heaptrack
- Scenario: Benchmark small, medium, large files
- Scenario: Compare with Tree-sitter C baseline
- Scenario: Document baseline measurements

**AC-PERF2: Performance Analysis** (4 scenarios)
- Scenario: Identify top 5 CPU bottlenecks
- Scenario: Analyze memory allocation patterns
- Scenario: Create prioritized optimization plan
- Scenario: Document expected performance gains

**AC-PERF3: Arena Allocation** (8 scenarios)
- Scenario: Allocate parse tree nodes in arena
- Scenario: Measure allocation reduction (>50%)
- Scenario: Measure memory reduction (>30% peak)
- Scenario: Measure parsing time improvement (>10%)
- Scenario: Validate correctness (all tests pass)
- Scenario: No memory leaks (valgrind clean)
- Scenario: Arena lifetime matches parse duration
- Scenario: No API breaking changes

**AC-PERF4: Parse-Stack Pooling** (7 scenarios)
- Scenario: Create parse-stack pool (size 32)
- Scenario: Reuse stacks during GLR fork/merge
- Scenario: Measure fork allocation reduction (>40%)
- Scenario: Measure performance improvement (>15% fork-heavy)
- Scenario: Minimal overhead on deterministic grammars (<5%)
- Scenario: Pool cleanup on parser drop
- Scenario: Validate correctness (all tests pass)

**AC-PERF5: Performance Validation** (5 scenarios)
- Scenario: Run benchmark suite (v0.8.0)
- Scenario: Compare with v0.7.0 baseline
- Scenario: Compare with Tree-sitter C (≤2x ratio)
- Scenario: Validate memory usage (<10x input size)
- Scenario: Document performance improvements

---

## Deliverables Summary

### Infrastructure (Week 3)

**Profiling Scripts**:
- `scripts/profile-cpu.sh` - CPU profiling with flamegraph
- `scripts/profile-memory.sh` - Memory profiling with heaptrack
- `scripts/compare-tree-sitter.sh` - Tree-sitter comparison

**Benchmarks**:
- `benches/glr-performance.rs` - Comprehensive GLR benchmark suite
- Test cases: small, medium, large files (Python, JavaScript, Rust)

**Documentation**:
- `docs/baselines/PERFORMANCE_BASELINE_V0.7.0.md` - v0.7.0 baseline
- `docs/analysis/PERFORMANCE_ANALYSIS_V0.7.0.md` - Bottleneck analysis
- `docs/plans/PERFORMANCE_OPTIMIZATION_PLAN.md` - Optimization plan

### Implementation (Week 4)

**Code Changes**:
- `runtime2/src/arena.rs` - Arena allocator implementation
- `glr-core/src/stack_pool.rs` - Parse-stack pool implementation
- Tree API updates (use arena references)
- GLR engine updates (use stack pool)

**Validation**:
- `docs/reports/PERFORMANCE_REPORT_V0.8.0.md` - Performance report
- Benchmark results (before/after comparison)
- Tree-sitter comparison (ratio tables)

**CI Integration**:
- Updated performance gates (new baseline)
- Benchmark suite in CI
- Performance regression alerts

---

## Definition of Done

**v0.8.0 is complete when**:

1. ✅ **All ACs met**: PERF1, PERF2, PERF3, PERF4, PERF5
2. ✅ **Performance goals achieved**:
   - Parsing time ≤2x Tree-sitter C (all benchmarks)
   - Memory usage <10x input size (all benchmarks)
3. ✅ **Correctness preserved**: 144/144 tests pass (100%)
4. ✅ **Documentation complete**:
   - Performance baseline (v0.7.0)
   - Performance analysis
   - Optimization plan
   - Performance report (v0.8.0)
   - Tree-sitter comparison
5. ✅ **CI integration**: Performance gates updated, regression detection active
6. ✅ **BDD scenarios**: 30 scenarios implemented and passing

---

## References

**Related Documents**:
- [ROADMAP.md](../../ROADMAP.md) - v0.8.0 scope
- [Strategic Implementation Plan](../plans/STRATEGIC_IMPLEMENTATION_PLAN.md) - Overall plan
- [Phase I Completion](../releases/PHASE_I_COMPLETION_SUMMARY.md) - Infrastructure foundation

**Performance Resources**:
- [Tree-sitter Performance](https://tree-sitter.github.io/tree-sitter/using-parsers#performance) - Tree-sitter C benchmarks
- [Rust Performance Book](https://nnethercote.github.io/perf-book/) - Rust optimization guide
- [Flamegraph](https://github.com/flamegraph-rs/flamegraph) - CPU profiling tool

**ADRs** (to be created if needed):
- ADR-0011: Arena Allocation Strategy (if architectural decision required)
- ADR-0012: Parse-Stack Pooling (if architectural decision required)

---

**Contract Version**: 1.0.0
**Status**: READY FOR IMPLEMENTATION
**Created**: November 20, 2025
**Target Completion**: Week 4 (end of November 2025)
**Maintained by**: rust-sitter core team
