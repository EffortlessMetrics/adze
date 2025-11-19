# Performance Baseline - rust-sitter v0.6.1-beta

**Last Updated**: November 15, 2025
**Version**: v0.6.1-beta
**Purpose**: Establish baseline performance metrics for tracking improvements and regressions

---

## Executive Summary

This document establishes the performance baseline for rust-sitter v0.6.1-beta. All future performance work will be measured against these metrics.

**Status**: 🚧 **In Progress** - Week 1 of v0.7.0 implementation
**Next Steps**: Run all benchmarks and populate this document with actual numbers

---

## Benchmark Infrastructure

### Available Benchmarks

rust-sitter has comprehensive benchmark coverage across multiple crates:

#### Core Parser Benchmarks
**Location**: `glr-core/benches/`
- `automaton.rs` - LR(1) automaton construction performance
- `perf_snapshot.rs` - Snapshot of parser performance

#### Runtime Benchmarks
**Location**: `runtime/benches/`
- `glr_parser_bench.rs` - GLR parser performance
- `parser_bench.rs` - General parser benchmarks
- `parser_benchmark.rs` - Comprehensive parser metrics
- `pure_rust_bench.rs` - Pure-Rust backend performance
- `incremental_benchmark.rs` - Incremental parsing (when enabled)
- `incremental_parsing.rs` - Incremental parse performance
- `incremental_simple.rs` - Simple incremental tests
- `perf_benchmark.rs` - Performance profiling
- `simple_bench.rs` - Basic parsing benchmarks

#### Table Generation Benchmarks
**Location**: `tablegen/benches/`
- `compression.rs` - Parse table compression performance

#### High-Level Benchmarks
**Location**: `benchmarks/benches/`
- `glr_hot.rs` - Hot path profiling
- `glr_performance.rs` - GLR-specific performance
- `optimization_bench.rs` - Optimization effectiveness
- `parse_bench.rs` - General parsing performance
- `stack_optimization.rs` - Stack operation performance
- `incremental_bench.rs` - Incremental parsing benchmarks

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark crate
cargo bench -p rust-sitter-glr-core
cargo bench -p rust-sitter
cargo bench -p rust-sitter-tablegen

# Run with performance counters (if feature enabled)
cargo bench --features perf-counters

# Quick benchmarks for fast iteration
./scripts/bench-quick.sh

# Save baseline for comparison
cargo bench -- --save-baseline before
# Make changes...
cargo bench -- --save-baseline after
# Compare with critcmp
critcmp before after
```

---

## Performance Metrics (To Be Populated)

### Parse Speed

**Methodology**: Parse representative source files of various sizes, measure tokens/second

| Grammar | File Size | Tokens | Time (ms) | Tokens/sec | Notes |
|---------|-----------|--------|-----------|------------|-------|
| Arithmetic | TBD | TBD | TBD | TBD | Simple expression grammar |
| JSON | TBD | TBD | TBD | TBD | Standard JSON grammar |
| Python | TBD | TBD | TBD | TBD | Complex grammar with 273 symbols |
| Test-Mini | TBD | TBD | TBD | TBD | Minimal test grammar |
| Test-Vec-Wrapper | TBD | TBD | TBD | TBD | Vec repetition grammar |

**Target for v0.7.0**: Document actual numbers, establish baseline

---

### Memory Usage

**Methodology**: Measure peak memory during parsing

| Grammar | Input Size | Peak Memory (MB) | Nodes Created | Bytes/Node | Notes |
|---------|-----------|------------------|---------------|------------|-------|
| Arithmetic | TBD | TBD | TBD | TBD | |
| JSON | TBD | TBD | TBD | TBD | |
| Python | TBD | TBD | TBD | TBD | |

**Target for v0.7.0**: Establish baseline, identify optimization opportunities

---

### GLR Parser Specifics

**Fork/Merge Performance**

| Test Case | Forks | Merges | Parse Time (μs) | Overhead vs LR | Notes |
|-----------|-------|--------|-----------------|----------------|-------|
| Unambiguous | TBD | TBD | TBD | TBD | Should have minimal GLR overhead |
| Ambiguous (2-way) | TBD | TBD | TBD | TBD | Simple ambiguity |
| Ambiguous (N-way) | TBD | TBD | TBD | TBD | Complex ambiguity |

**Target for v0.7.0**: Quantify GLR overhead, optimize fork/merge operations

---

### Incremental Parsing

**When Implemented** (v0.7.0)

| Edit Type | File Size | Edited Region | Full Parse (ms) | Incremental Parse (ms) | Speedup | Subtrees Reused |
|-----------|-----------|---------------|-----------------|------------------------|---------|-----------------|
| Single char | TBD | TBD | TBD | TBD | TBD | TBD |
| Small edit | TBD | TBD | TBD | TBD | TBD | TBD |
| Large edit | TBD | TBD | TBD | TBD | TBD | TBD |

**Target for v0.7.0**: 10x+ speedup on small edits

---

### Table Compression

**Parse Table Sizes**

| Grammar | Uncompressed (KB) | Compressed (KB) | Compression Ratio | Compression Time (ms) | Notes |
|---------|-------------------|-----------------|-------------------|----------------------|-------|
| Arithmetic | TBD | TBD | TBD | TBD | Simple grammar |
| JSON | TBD | TBD | TBD | TBD | Medium complexity |
| Python | TBD | TBD | TBD | TBD | Complex grammar (273 symbols) |

**Target for v0.7.0**: Document current compression effectiveness

---

## Comparison to Tree-sitter-c

**Methodology**: Compare rust-sitter pure-Rust backend to official Tree-sitter C implementation

### Parse Speed Comparison

| Grammar | Tree-sitter-c (tokens/sec) | rust-sitter (tokens/sec) | Ratio | Notes |
|---------|---------------------------|--------------------------|-------|-------|
| JSON | TBD | TBD | TBD | |
| Python | TBD | TBD | TBD | |
| JavaScript | TBD | TBD | TBD | |

### Memory Usage Comparison

| Grammar | Tree-sitter-c (MB) | rust-sitter (MB) | Ratio | Notes |
|---------|-------------------|------------------|-------|-------|
| JSON | TBD | TBD | TBD | |
| Python | TBD | TBD | TBD | |
| JavaScript | TBD | TBD | TBD | |

**Target for v0.7.0**: Document actual comparison, identify performance gaps

---

## Performance Profiling

### Hot Paths (To Be Identified)

**Methodology**: Use `cargo flamegraph` and `perf` to identify hot paths

**Expected Hot Paths**:
- Token recognition (lexing)
- Action table lookup
- Stack operations (push/pop/peek)
- GOTO table lookup
- GLR fork/merge logic
- Tree node construction

**Profiling Commands**:
```bash
# Generate flamegraph
cargo flamegraph --bench parser_bench

# Use perf for CPU profiling
perf record cargo bench
perf report

# Memory profiling with heaptrack
heaptrack cargo bench
heaptrack_gui heaptrack.cargo.*.gz
```

**Target for v0.7.0**: Identify and document top 5 hot paths

---

## Optimization Opportunities

### Identified (To Be Populated)

**Once profiling is complete, list opportunities ranked by impact:**

1. **TBD** - Estimated impact: X%
2. **TBD** - Estimated impact: Y%
3. **TBD** - Estimated impact: Z%

---

## Historical Performance Data

### v0.6.1-beta Baseline

**Performance Characteristics** (qualitative, to be quantified):
- ✅ GLR parsing is algorithmically correct
- ✅ All core tests passing (13/13 macro, 6/6 integration)
- ⚠️ Performance not yet profiled
- ⚠️ No regression tests in CI
- ⚠️ Unknown comparison to tree-sitter-c

---

## CI Integration

### Current Status

**Performance CI**: ❌ Not yet implemented

**Planned for v0.7.0**:
- [ ] Performance regression workflow (`.github/workflows/performance.yml`)
- [ ] Baseline comparison on PRs
- [ ] Automatic regression detection (>5% slowdown triggers failure)
- [ ] Performance reports as PR comments
- [ ] Historical performance tracking

### Target CI Workflow

```yaml
name: Performance

on: [pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run benchmarks (baseline)
        run: cargo bench -- --save-baseline pr-base

      - name: Checkout PR
        run: git checkout ${{ github.event.pull_request.head.sha }}

      - name: Run benchmarks (PR)
        run: cargo bench -- --save-baseline pr-head

      - name: Compare performance
        run: critcmp pr-base pr-head > performance_report.txt

      - name: Check for regressions
        run: |
          # Fail if any benchmark regressed >5%
          ./scripts/check-performance-regression.sh performance_report.txt

      - name: Comment PR
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('performance_report.txt', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.name,
              body: '## Performance Report\n\n```\n' + report + '\n```'
            });
```

---

## Action Items

### Week 1 (Current)

- [ ] **Run all benchmarks** - Populate tables in this document
- [ ] **Profile with flamegraph** - Identify hot paths
- [ ] **Memory profiling** - Measure peak memory usage
- [ ] **Document comparison to tree-sitter-c** - If comparison grammars available
- [ ] **Create performance CI workflow** - Prevent future regressions

### Week 2-6 (During v0.7.0 development)

- [ ] **Incremental parsing benchmarks** - Once implemented
- [ ] **Query system benchmarks** - Once predicates complete
- [ ] **Re-run after major changes** - Track performance impact
- [ ] **Optimize identified hot paths** - Based on profiling data

### v0.7.0 Release

- [ ] **Complete performance baseline** - All metrics populated
- [ ] **Performance tuning guide** - Based on profiling findings
- [ ] **Regression tests in CI** - Automatic detection
- [ ] **Performance report in changelog** - Summary of improvements

---

## How to Contribute

### Establishing Baseline

**Want to help establish the baseline?** Here's how:

1. **Run benchmarks**: `cargo bench | tee benchmark_results.txt`
2. **Record results**: Add to tables in this document
3. **Run profiling**: Generate flamegraphs and identify hot paths
4. **Submit PR**: With populated performance data

### Performance Improvements

**Found a performance bottleneck?**

1. **Document current performance**: Add benchmark before changes
2. **Make optimization**: Implement improvement
3. **Measure improvement**: Run benchmark after changes
4. **Submit PR**: With before/after comparison

See [GAPS.md#performance-benchmarking](../GAPS.md#performance-benchmarking) for detailed tasks.

---

## Resources

- **Profiling Guide**: [docs/PERFORMANCE_TUNING.md](./PERFORMANCE_TUNING.md) (to be created in Week 7)
- **Benchmark Infrastructure**: [benchmarks/](../benchmarks/)
- **Implementation Plan**: [IMPLEMENTATION_PLAN.md](../IMPLEMENTATION_PLAN.md) - Week 1
- **Task List**: [GAPS.md](../GAPS.md) - Performance section

---

**Status**: 🚧 **Week 1 In Progress**
**Next Review**: After benchmarks are run and data is populated
**Maintained By**: rust-sitter core team
