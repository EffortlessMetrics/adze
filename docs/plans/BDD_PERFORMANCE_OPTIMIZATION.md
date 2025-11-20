# BDD Scenarios: Performance Optimization (v0.8.0)

**Version**: 1.0.0
**Contract**: [PERFORMANCE_OPTIMIZATION_CONTRACT.md](../specs/PERFORMANCE_OPTIMIZATION_CONTRACT.md)
**Target**: v0.8.0
**Total Scenarios**: 30
**Created**: November 20, 2025

---

## Overview

This document defines the behavior-driven development (BDD) scenarios for v0.8.0 (Performance Optimization). Each scenario follows the Given-When-Then format and maps to acceptance criteria in the contract.

**Scenario Distribution**:
- AC-PERF1 (Profiling Infrastructure): 6 scenarios
- AC-PERF2 (Performance Analysis): 4 scenarios
- AC-PERF3 (Arena Allocation): 8 scenarios
- AC-PERF4 (Parse-Stack Pooling): 7 scenarios
- AC-PERF5 (Performance Validation): 5 scenarios

---

## AC-PERF1: Profiling Infrastructure (6 Scenarios)

### Scenario 1.1: Profile GLR parsing on large file

```gherkin
Feature: CPU Profiling
  As a performance engineer
  I want to profile GLR parsing on large files
  So that I can identify performance bottlenecks

Scenario: Profile GLR parsing on large Python file
  Given a large Python file with 10,000 lines of code
  And the GLR parser configured for Python grammar
  When I run the CPU profiling script
  Then a flamegraph is generated showing function-level hotspots
  And the profiling output includes percentage breakdown
  And the top 5 functions by time are identified
  And the profiling overhead is less than 20% of parse time
```

### Scenario 1.2: Generate flamegraph for CPU hotspots

```gherkin
Feature: Flamegraph Generation
  As a performance engineer
  I want to generate flamegraphs for CPU profiling
  So that I can visualize performance hotspots

Scenario: Generate flamegraph from profiling data
  Given profiling data from a GLR parse
  And the cargo-flamegraph tool is installed
  When I run the flamegraph generation script
  Then an SVG flamegraph is created
  And the flamegraph shows function call hierarchy
  And hotspot functions are highlighted (>1% of total time)
  And the flamegraph is saved to docs/analysis/flamegraph-*.svg
```

### Scenario 1.3: Profile memory allocations with heaptrack

```gherkin
Feature: Memory Profiling
  As a performance engineer
  I want to profile memory allocations during parsing
  So that I can identify allocation hotspots

Scenario: Profile memory allocations with heaptrack
  Given a medium-sized JavaScript file (5,000 LOC)
  And the GLR parser configured for JavaScript grammar
  When I run the memory profiling script with heaptrack
  Then a heaptrack report is generated
  And the report shows allocation count and sizes
  And allocation hotspots are identified (>1% of total allocations)
  And peak memory usage is recorded
  And temporary vs. long-lived allocations are distinguished
```

### Scenario 1.4: Benchmark small, medium, and large files

```gherkin
Feature: Comprehensive Benchmarking
  As a performance engineer
  I want to benchmark parsing across file sizes
  So that I can understand performance scaling

Scenario: Run comprehensive benchmark suite
  Given test files of varying sizes:
    | Language   | Size Category | Lines of Code |
    | Python     | Small         | 50            |
    | Python     | Medium        | 5,000         |
    | Python     | Large         | 15,000        |
    | JavaScript | Small         | 100           |
    | JavaScript | Medium        | 3,000         |
    | Rust       | Small         | 75            |
  When I run the benchmark suite
  Then each file is parsed 100 times
  And mean, median, and standard deviation are calculated
  And results are saved to benches/results/baseline-v0.7.0.json
  And benchmark output includes parse time and memory usage
```

### Scenario 1.5: Compare with Tree-sitter C baseline

```gherkin
Feature: Tree-sitter Comparison
  As a performance engineer
  I want to compare rust-sitter with Tree-sitter C
  So that I can quantify the performance gap

Scenario: Benchmark rust-sitter vs. Tree-sitter C
  Given the same input file (Python, 5,000 LOC)
  And the equivalent grammar for both parsers
  When I run the comparison script
  Then Tree-sitter C parsing time is measured
  And rust-sitter parsing time is measured
  And the performance ratio is calculated (rust-sitter / tree-sitter)
  And the results show:
    - Tree-sitter: 45ms
    - rust-sitter: 120ms
    - Ratio: 2.67x slower
  And the comparison is documented in docs/baselines/tree-sitter-comparison.md
```

### Scenario 1.6: Document baseline measurements

```gherkin
Feature: Baseline Documentation
  As a performance engineer
  I want to document v0.7.0 performance baseline
  So that I can measure improvement in v0.8.0

Scenario: Create performance baseline document
  Given benchmark results for v0.7.0
  And profiling data (CPU, memory)
  And Tree-sitter comparison ratios
  When I generate the baseline documentation
  Then a document is created at docs/baselines/PERFORMANCE_BASELINE_V0.7.0.md
  And the document includes:
    - Parse time by file size (small, medium, large)
    - Memory usage by file size
    - Allocation counts
    - Tree-sitter comparison ratios
    - Top 5 CPU bottlenecks
    - Top 5 memory hotspots
  And all measurements include statistical data (mean, stddev)
```

---

## AC-PERF2: Performance Analysis (4 Scenarios)

### Scenario 2.1: Identify top 5 CPU bottlenecks

```gherkin
Feature: Bottleneck Identification
  As a performance engineer
  I want to identify the top CPU bottlenecks
  So that I can prioritize optimization efforts

Scenario: Analyze flamegraph to identify bottlenecks
  Given a flamegraph from profiling data
  And the total parse time is 100ms
  When I analyze the flamegraph
  Then the top 5 functions by time are identified:
    | Function                | Time (ms) | Percentage |
    | glr_fork                | 25        | 25%        |
    | parse_stack_clone       | 20        | 20%        |
    | allocate_tree_node      | 15        | 15%        |
    | token_lookahead         | 10        | 10%        |
    | reduce_action           | 8         | 8%         |
  And root causes are documented for each bottleneck
  And optimization opportunities are identified
```

### Scenario 2.2: Analyze memory allocation patterns

```gherkin
Feature: Memory Analysis
  As a performance engineer
  I want to analyze memory allocation patterns
  So that I can identify optimization opportunities

Scenario: Analyze heaptrack data for allocation patterns
  Given heaptrack profiling data from a medium file parse
  And total allocations: 50,000
  And peak memory: 10 MB
  When I analyze the allocation patterns
  Then allocation hotspots are identified (>1% of total):
    | Allocator              | Count  | Size (MB) | Percentage |
    | parse_tree_node_alloc  | 15,000 | 4.5       | 45%        |
    | parse_stack_alloc      | 10,000 | 2.0       | 20%        |
    | token_buffer_alloc     | 5,000  | 1.5       | 15%        |
  And short-lived vs. long-lived allocations are distinguished
  And potential for arena allocation is calculated (>50% reduction)
  And potential for object pooling is identified (parse stacks)
```

### Scenario 2.3: Create prioritized optimization plan

```gherkin
Feature: Optimization Planning
  As a performance engineer
  I want to create a prioritized optimization plan
  So that I can focus on high-impact optimizations

Scenario: Prioritize optimizations by impact
  Given bottleneck analysis (CPU, memory)
  And allocation pattern analysis
  When I create the optimization plan
  Then optimizations are prioritized by expected impact:
    | Optimization            | Expected Gain | Complexity | Risk | Priority |
    | Arena allocation        | 30% memory    | Medium     | Low  | High     |
    | Parse-stack pooling     | 20% CPU       | Medium     | Low  | High     |
    | Zero-copy token buffer  | 10% memory    | High       | Med  | Medium   |
    | SIMD token scanning     | 5% CPU        | High       | Low  | Low      |
  And each optimization has clear acceptance criteria
  And risk mitigation strategies are defined
  And the plan is documented in docs/plans/PERFORMANCE_OPTIMIZATION_PLAN.md
```

### Scenario 2.4: Document expected performance gains

```gherkin
Feature: Performance Gain Estimation
  As a performance engineer
  I want to document expected performance gains
  So that I can validate optimization results

Scenario: Estimate performance gains for optimizations
  Given the optimization plan
  And baseline measurements (v0.7.0)
  When I estimate performance gains
  Then expected improvements are documented:
    - Arena allocation: 30% memory reduction, 10% parsing speed improvement
    - Parse-stack pooling: 40% fork allocation reduction, 15% parsing speed improvement (fork-heavy)
    - Combined optimizations: 30% total memory reduction, 20% total parsing speed improvement
  And Tree-sitter comparison ratio improvement:
    - Current: 2.67x slower
    - Expected after optimizations: 2.0x slower (goal: ≤2x)
  And estimates are conservative (worst case)
```

---

## AC-PERF3: Arena Allocation (8 Scenarios)

### Scenario 3.1: Allocate parse tree nodes in arena

```gherkin
Feature: Arena Allocation
  As a performance engineer
  I want to allocate parse tree nodes in an arena
  So that I can reduce allocation overhead

Scenario: Create arena allocator for parse trees
  Given a parse operation for a medium-sized file
  When I create an arena allocator
  Then the arena is initialized with a capacity (e.g., 1MB)
  And parse tree nodes are allocated from the arena (bump allocation)
  And node allocation is contiguous in memory
  And the arena lifetime matches the parse duration
  And the arena is dropped when the parse completes
```

### Scenario 3.2: Measure allocation reduction (>50%)

```gherkin
Feature: Allocation Reduction
  As a performance engineer
  I want to measure allocation reduction from arena allocation
  So that I can validate optimization effectiveness

Scenario: Compare allocations before and after arena allocation
  Given a medium-sized Python file (5,000 LOC)
  And baseline allocations (v0.7.0): 15,000 tree node allocations
  When I parse with arena allocation (v0.8.0)
  Then total tree node allocations are measured
  And allocation reduction is calculated:
    - Before: 15,000 allocations
    - After: 1 arena allocation + 0 tree node allocations
    - Reduction: 99.99% (>50% target met)
  And the reduction is validated with heaptrack
```

### Scenario 3.3: Measure memory reduction (>30% peak)

```gherkin
Feature: Memory Reduction
  As a performance engineer
  I want to measure peak memory reduction from arena allocation
  So that I can validate memory optimization

Scenario: Compare peak memory before and after arena allocation
  Given a large Python file (15,000 LOC)
  And baseline peak memory (v0.7.0): 15 MB
  When I parse with arena allocation (v0.8.0)
  Then peak memory usage is measured with heaptrack
  And memory reduction is calculated:
    - Before: 15 MB
    - After: 10 MB
    - Reduction: 33% (>30% target met)
  And memory usage scales linearly with file size
```

### Scenario 3.4: Measure parsing time improvement (>10%)

```gherkin
Feature: Parsing Speed Improvement
  As a performance engineer
  I want to measure parsing time improvement from arena allocation
  So that I can validate performance optimization

Scenario: Compare parsing time before and after arena allocation
  Given a large Python file (15,000 LOC)
  And baseline parsing time (v0.7.0): 180ms
  When I benchmark with arena allocation (v0.8.0)
  Then parsing time is measured across 100 iterations
  And time improvement is calculated:
    - Before: 180ms (mean)
    - After: 160ms (mean)
    - Improvement: 11% (>10% target met)
  And improvement is statistically significant (t-test, p < 0.05)
```

### Scenario 3.5: Validate correctness (all tests pass)

```gherkin
Feature: Correctness Validation
  As a performance engineer
  I want to ensure arena allocation doesn't introduce bugs
  So that parser correctness is preserved

Scenario: Run full test suite with arena allocation
  Given the arena allocation implementation
  And the existing test suite (144 tests)
  When I run all tests
  Then all 144 tests pass (100% pass rate)
  And no new test failures are introduced
  And existing passing tests remain passing
  And parse results match v0.7.0 exactly (golden test validation)
```

### Scenario 3.6: No memory leaks (valgrind clean)

```gherkin
Feature: Memory Safety
  As a performance engineer
  I want to validate arena allocation has no memory leaks
  So that memory safety is preserved

Scenario: Run valgrind to detect memory leaks
  Given the arena allocation implementation
  And a test parsing a medium file
  When I run the test under valgrind
  Then valgrind reports 0 memory leaks
  And all memory is properly freed
  And no invalid memory access is detected
  And valgrind output shows "All heap blocks were freed -- no leaks are possible"
```

### Scenario 3.7: Arena lifetime matches parse duration

```gherkin
Feature: Lifetime Correctness
  As a performance engineer
  I want to ensure arena lifetime is correct
  So that no use-after-free bugs are introduced

Scenario: Validate arena lifetime with Rust lifetimes
  Given the arena allocator with lifetime 'arena
  And parse tree nodes with references to arena
  When I attempt to use tree after arena is dropped
  Then the Rust compiler rejects the code (lifetime error)
  And the arena outlives all tree nodes (enforced by type system)
  And no runtime use-after-free is possible
  And Miri testing passes (undefined behavior detection)
```

### Scenario 3.8: No API breaking changes

```gherkin
Feature: API Compatibility
  As a performance engineer
  I want to ensure arena allocation doesn't break the API
  So that existing user code continues to work

Scenario: Validate API compatibility with example grammars
  Given example grammars (arithmetic, optional, repetition, word)
  And existing user code using Tree API
  When I compile user code with v0.8.0
  Then the code compiles without changes
  And parse results match v0.7.0 exactly
  And no API breaking changes are introduced
  And arena allocation is an internal optimization (transparent to users)
```

---

## AC-PERF4: Parse-Stack Pooling (7 Scenarios)

### Scenario 4.1: Create parse-stack pool (size 32)

```gherkin
Feature: Stack Pooling
  As a performance engineer
  I want to create a parse-stack pool
  So that I can reuse stacks during GLR fork/merge

Scenario: Initialize parse-stack pool
  Given a GLR parser instance
  When the parser is created
  Then a parse-stack pool is initialized
  And the pool has a default capacity of 32 stacks
  And the pool is empty initially (no stacks allocated)
  And the pool is thread-local or per-parser
  And the pool is dropped when the parser is dropped
```

### Scenario 4.2: Reuse stacks during GLR fork/merge

```gherkin
Feature: Stack Reuse
  As a performance engineer
  I want to reuse stacks from the pool during fork/merge
  So that I can reduce fork allocation overhead

Scenario: Reuse stacks during GLR fork operation
  Given a GLR parse with fork/merge conflicts
  And the parse-stack pool is initialized
  When a fork operation occurs (shift/reduce conflict)
  Then a stack is requested from the pool
  And if the pool has available stacks, a stack is reused
  And if the pool is empty, a new stack is allocated
  And the reused stack is reset to initial state
  And the fork operation completes successfully
  And parse results are identical to non-pooled implementation
```

### Scenario 4.3: Measure fork allocation reduction (>40%)

```gherkin
Feature: Fork Allocation Reduction
  As a performance engineer
  I want to measure fork allocation reduction from stack pooling
  So that I can validate optimization effectiveness

Scenario: Compare fork allocations before and after stack pooling
  Given a fork-heavy Python grammar (many ambiguities)
  And a medium-sized Python file (5,000 LOC) with 1,000 forks
  And baseline fork allocations (v0.7.0): 1,000 stack allocations
  When I parse with stack pooling (v0.8.0)
  Then fork allocations are measured
  And allocation reduction is calculated:
    - Before: 1,000 stack allocations
    - After: 32 pool allocations + 0 fork allocations (all reused)
    - Reduction: 97% (>40% target met)
  And the reduction is validated with heaptrack
```

### Scenario 4.4: Measure performance improvement (>15% fork-heavy)

```gherkin
Feature: Fork Performance Improvement
  As a performance engineer
  I want to measure performance improvement on fork-heavy workloads
  So that I can validate stack pooling effectiveness

Scenario: Compare parsing time on fork-heavy grammar
  Given a fork-heavy Python file (5,000 LOC, 1,000 forks)
  And baseline parsing time (v0.7.0): 200ms
  When I benchmark with stack pooling (v0.8.0)
  Then parsing time is measured across 100 iterations
  And time improvement is calculated:
    - Before: 200ms (mean)
    - After: 165ms (mean)
    - Improvement: 17.5% (>15% target met)
  And improvement is statistically significant (t-test, p < 0.05)
```

### Scenario 4.5: Minimal overhead on deterministic grammars (<5%)

```gherkin
Feature: Deterministic Grammar Performance
  As a performance engineer
  I want to ensure stack pooling has minimal overhead on deterministic grammars
  So that non-fork-heavy workloads are not penalized

Scenario: Measure overhead on deterministic grammar
  Given a deterministic JavaScript grammar (no forks)
  And a medium-sized JavaScript file (3,000 LOC)
  And baseline parsing time (v0.7.0): 80ms
  When I benchmark with stack pooling (v0.8.0)
  Then parsing time is measured across 100 iterations
  And overhead is calculated:
    - Before: 80ms (mean)
    - After: 82ms (mean)
    - Overhead: 2.5% (<5% target met)
  And the overhead is within acceptable bounds
```

### Scenario 4.6: Pool cleanup on parser drop

```gherkin
Feature: Pool Memory Management
  As a performance engineer
  I want to ensure the pool is properly cleaned up
  So that no memory leaks occur

Scenario: Validate pool cleanup when parser is dropped
  Given a GLR parser with a stack pool
  And the pool contains 10 reused stacks
  When the parser is dropped
  Then all pool stacks are freed
  And the pool memory is released
  And valgrind shows no memory leaks
  And no stacks are leaked after parser drop
```

### Scenario 4.7: Validate correctness (all tests pass)

```gherkin
Feature: Stack Pooling Correctness
  As a performance engineer
  I want to ensure stack pooling doesn't introduce bugs
  So that parser correctness is preserved

Scenario: Run full test suite with stack pooling
  Given the stack pooling implementation
  And the existing test suite (144 tests)
  When I run all tests
  Then all 144 tests pass (100% pass rate)
  And no new test failures are introduced
  And parse results match v0.7.0 exactly (golden test validation)
  And fork/merge behavior is identical to non-pooled implementation
```

---

## AC-PERF5: Performance Validation (5 Scenarios)

### Scenario 5.1: Run benchmark suite (v0.8.0)

```gherkin
Feature: v0.8.0 Benchmarking
  As a performance engineer
  I want to benchmark v0.8.0 performance
  So that I can measure optimization impact

Scenario: Run full benchmark suite on v0.8.0
  Given v0.8.0 with arena allocation and stack pooling
  And the comprehensive benchmark suite (small, medium, large files)
  When I run the benchmark suite
  Then all benchmarks complete successfully
  And results are saved to benches/results/v0.8.0.json
  And results include:
    - Parse time (mean, median, stddev)
    - Memory usage (peak, average)
    - Allocation counts
    - Fork counts (for fork-heavy grammars)
  And results are comparable to v0.7.0 baseline
```

### Scenario 5.2: Compare with v0.7.0 baseline

```gherkin
Feature: Version Comparison
  As a performance engineer
  I want to compare v0.8.0 with v0.7.0
  So that I can quantify optimization improvements

Scenario: Calculate improvement from v0.7.0 to v0.8.0
  Given v0.7.0 baseline results
  And v0.8.0 benchmark results
  When I compare the results
  Then improvements are calculated for each metric:
    | Metric              | v0.7.0  | v0.8.0  | Improvement |
    | Parse time (medium) | 100ms   | 80ms    | 20%         |
    | Peak memory (large) | 15MB    | 10MB    | 33%         |
    | Allocations (medium)| 15,000  | 50      | 99.7%       |
    | Fork allocs (Python)| 1,000   | 32      | 97%         |
  And all improvements meet or exceed targets
  And no regressions are observed
```

### Scenario 5.3: Compare with Tree-sitter C (≤2x ratio)

```gherkin
Feature: Tree-sitter Comparison
  As a performance engineer
  I want to compare v0.8.0 with Tree-sitter C
  So that I can validate the 2x performance goal

Scenario: Validate parsing time within 2x of Tree-sitter C
  Given Tree-sitter C baseline benchmarks
  And v0.8.0 benchmark results
  When I calculate the performance ratio
  Then the ratio is ≤2x for all benchmarks:
    | File Size | Tree-sitter | rust-sitter | Ratio | Goal Met? |
    | Small     | 5ms         | 9ms         | 1.8x  | ✅         |
    | Medium    | 45ms        | 85ms        | 1.9x  | ✅         |
    | Large     | 180ms       | 350ms       | 1.9x  | ✅         |
  And the 2x performance goal is achieved
  And results are documented in docs/reports/PERFORMANCE_REPORT_V0.8.0.md
```

### Scenario 5.4: Validate memory usage (<10x input size)

```gherkin
Feature: Memory Usage Validation
  As a performance engineer
  I want to validate memory usage is within 10x input size
  So that memory efficiency is acceptable

Scenario: Validate peak memory usage across file sizes
  Given v0.8.0 benchmark results
  And input file sizes
  When I calculate memory usage ratio (peak / input size)
  Then the ratio is <10x for all benchmarks:
    | File      | Input Size | Peak Memory | Ratio | Goal Met? |
    | Small     | 5 KB       | 40 KB       | 8x    | ✅         |
    | Medium    | 500 KB     | 4 MB        | 8x    | ✅         |
    | Large     | 1.5 MB     | 10 MB       | 6.7x  | ✅         |
  And the memory efficiency goal is achieved
  And memory usage scales sub-linearly with file size
```

### Scenario 5.5: Document performance improvements

```gherkin
Feature: Performance Documentation
  As a performance engineer
  I want to document v0.8.0 performance improvements
  So that optimization results are recorded

Scenario: Create comprehensive performance report
  Given v0.8.0 benchmark results
  And v0.7.0 baseline comparison
  And Tree-sitter C comparison
  When I generate the performance report
  Then a document is created at docs/reports/PERFORMANCE_REPORT_V0.8.0.md
  And the report includes:
    - Executive summary (key achievements)
    - Optimization summary (arena allocation, stack pooling)
    - Benchmark results (before/after tables)
    - Tree-sitter comparison (ratio tables, goal validation)
    - Memory usage analysis (allocation reduction, peak reduction)
    - Correctness validation (144/144 tests pass)
    - Lessons learned and future optimizations
  And the report is comprehensive and actionable
```

---

## Scenario Summary

**Total Scenarios**: 30

**By Acceptance Criterion**:
- AC-PERF1 (Profiling Infrastructure): 6 scenarios
- AC-PERF2 (Performance Analysis): 4 scenarios
- AC-PERF3 (Arena Allocation): 8 scenarios
- AC-PERF4 (Parse-Stack Pooling): 7 scenarios
- AC-PERF5 (Performance Validation): 5 scenarios

**By Category**:
- Infrastructure (profiling, benchmarking): 6 scenarios
- Analysis (bottlenecks, optimization planning): 4 scenarios
- Implementation (arena, pooling): 15 scenarios
- Validation (correctness, performance): 5 scenarios

**Coverage**:
- Functional correctness: 100% (all tests pass, no regressions)
- Performance goals: 100% (≤2x Tree-sitter, <10x memory)
- Memory safety: 100% (valgrind clean, lifetime correct)
- API compatibility: 100% (no breaking changes)

---

## Implementation Notes

**Testing Strategy**:
1. **Property-based testing**: Use proptest for allocation reduction validation
2. **Golden tests**: Parse results match v0.7.0 exactly
3. **Benchmark automation**: CI runs benchmark suite on every commit
4. **Regression detection**: Alert on >5% performance regression

**Tools Required**:
- `cargo-flamegraph` - CPU profiling
- `heaptrack` - Memory profiling
- `valgrind` - Memory leak detection
- `cargo-miri` - Undefined behavior detection
- `cargo-criterion` - Statistical benchmarking

**Validation Process**:
1. Implement optimization
2. Run full test suite (144 tests)
3. Run benchmark suite (compare with baseline)
4. Run valgrind (memory safety)
5. Run miri (undefined behavior)
6. Document results

---

## References

**Related Documents**:
- [Performance Optimization Contract](../specs/PERFORMANCE_OPTIMIZATION_CONTRACT.md)
- [ROADMAP.md](../../ROADMAP.md) - v0.8.0 scope
- [Strategic Implementation Plan](../plans/STRATEGIC_IMPLEMENTATION_PLAN.md)

**Previous BDD Scenarios**:
- [BDD_POLICY_ENFORCEMENT.md](./BDD_POLICY_ENFORCEMENT.md) - Policy-as-Code scenarios
- [BDD_INCREMENTAL_PARSING.md](./BDD_INCREMENTAL_PARSING.md) - Incremental parsing scenarios

---

**Document Version**: 1.0.0
**Status**: READY FOR IMPLEMENTATION
**Created**: November 20, 2025
**Maintained by**: rust-sitter core team
