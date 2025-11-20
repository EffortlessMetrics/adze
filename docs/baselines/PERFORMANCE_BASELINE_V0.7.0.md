# Performance Baseline v0.7.0

**Status**: Template (Measurements pending Week 3 Day 2)
**Created**: 2025-11-20
**Version**: v0.7.0 (Pre-optimization baseline)
**Contract**: [PERFORMANCE_OPTIMIZATION_CONTRACT.md](../specs/PERFORMANCE_OPTIMIZATION_CONTRACT.md)
**BDD Scenarios**: [BDD_PERFORMANCE_OPTIMIZATION.md](../plans/BDD_PERFORMANCE_OPTIMIZATION.md)

## Executive Summary

This document establishes the **v0.7.0 performance baseline** for rust-sitter's GLR parser implementation, serving as the reference point for measuring improvements in v0.8.0 (Performance Optimization milestone).

**Purpose**: Document baseline performance metrics before optimization work to enable:
- Quantitative measurement of optimization impact (AC-PERF3, AC-PERF4)
- Validation of 2x performance goal vs Tree-sitter C (AC-PERF5)
- Historical performance tracking across versions

**Key Metrics** (to be measured):
- Parse time per language/size combination
- Memory usage (peak, allocations)
- CPU profiling (hot functions)
- Allocation patterns (TreeNode, parse-stack)
- Comparison to Tree-sitter C baseline

**Status**: Infrastructure complete, actual measurements pending Week 3 Day 2.

---

## Baseline Methodology

### Measurement Approach

**BDD Scenario Coverage**:
- **Scenario 1.1**: CPU profiling captures hot functions (>1% CPU time)
- **Scenario 1.2**: Memory profiling captures allocation patterns
- **Scenario 1.3**: Benchmarks cover small, medium, large files
- **Scenario 1.4**: Tree-sitter baseline established for comparison

**Benchmarking Framework**:
- **Tool**: Criterion.rs (statistical benchmarking)
- **Configuration**: 100 samples, 10 warmup runs, 10-second measurement time
- **Location**: `benches/glr-performance.rs`
- **Fixtures**: `benches/fixtures/{language}/{size}/sample.{ext}`

**Profiling Tools**:
- **CPU**: `cargo-flamegraph` (generates SVG flamegraphs)
- **Memory**: `heaptrack` (Linux) or `valgrind --tool=massif`
- **Scripts**: `scripts/profile-cpu.sh`, `scripts/profile-memory.sh`

**Tree-sitter Comparison**:
- **Tool**: `tree-sitter` CLI + `hyperfine` for timing
- **Script**: `scripts/compare-tree-sitter.sh`
- **Method**: Same fixtures, same environment, statistical comparison

### Environment Specification

**System Configuration** (to be recorded):
```yaml
os: Linux / macOS / Windows
arch: x86_64 / aarch64
cpu_model: TBD
cpu_cores: TBD
memory_total: TBD
rust_version: 1.89.0 (Rust 2024 Edition)
tree_sitter_version: TBD
```

**Environment Variables**:
```bash
RUST_BACKTRACE=1
RUST_TEST_THREADS=2
RAYON_NUM_THREADS=4
TOKIO_WORKER_THREADS=2
```

**Build Configuration**:
```bash
# Release build with optimizations
cargo build --release
cargo build --release --features glr-core
```

---

## Benchmark Results (v0.7.0 Baseline)

### Parsing Performance

**Status**: Measurements pending Week 3 Day 2

#### Python Grammar

| Size   | Lines | Bytes  | Parse Time (ms) | Throughput (MB/s) | Memory (MB) | Allocations |
|--------|-------|--------|-----------------|-------------------|-------------|-------------|
| Small  | ~50   | ~2 KB  | TBD             | TBD               | TBD         | TBD         |
| Medium | ~500  | ~20 KB | TBD             | TBD               | TBD         | TBD         |
| Large  | ~5000 | ~200KB | TBD             | TBD               | TBD         | TBD         |

#### JavaScript Grammar

| Size   | Lines | Bytes  | Parse Time (ms) | Throughput (MB/s) | Memory (MB) | Allocations |
|--------|-------|--------|-----------------|-------------------|-------------|-------------|
| Small  | ~100  | ~3 KB  | TBD             | TBD               | TBD         | TBD         |
| Medium | ~1000 | ~30 KB | TBD             | TBD               | TBD         | TBD         |
| Large  | ~10K  | ~300KB | TBD             | TBD               | TBD         | TBD         |

#### Rust Grammar

| Size   | Lines | Bytes  | Parse Time (ms) | Throughput (MB/s) | Memory (MB) | Allocations |
|--------|-------|--------|-----------------|-------------------|-------------|-------------|
| Small  | ~75   | ~2.5KB | TBD             | TBD               | TBD         | TBD         |
| Medium | ~750  | ~25 KB | TBD             | TBD               | TBD         | TBD         |
| Large  | ~7500 | ~250KB | TBD             | TBD               | TBD         | TBD         |

### Tree-sitter C Baseline Comparison

**Purpose**: Establish reference performance for validation against 2x goal (AC-PERF5)

| Language   | Size   | Tree-sitter (ms) | rust-sitter (ms) | Ratio | Notes |
|------------|--------|------------------|------------------|-------|-------|
| Python     | Small  | TBD              | TBD              | TBD   | TBD   |
| Python     | Medium | TBD              | TBD              | TBD   | TBD   |
| Python     | Large  | TBD              | TBD              | TBD   | TBD   |
| JavaScript | Small  | TBD              | TBD              | TBD   | TBD   |
| JavaScript | Medium | TBD              | TBD              | TBD   | TBD   |
| JavaScript | Large  | TBD              | TBD              | TBD   | TBD   |
| Rust       | Small  | TBD              | TBD              | TBD   | TBD   |
| Rust       | Medium | TBD              | TBD              | TBD   | TBD   |
| Rust       | Large  | TBD              | TBD              | TBD   | TBD   |

**BDD Scenario 5.2**: All ratios must be ≤2.0x to meet performance goal.

---

## CPU Profiling Analysis

### Hot Functions (>1% CPU Time)

**Status**: Profiling pending Week 3 Day 2

**BDD Scenario 1.1**: CPU profiling must identify functions consuming >1% CPU time.

#### Expected Hot Functions

Based on GLR parser architecture, we expect these functions to be hotspots:

1. **Parsing Loop** (`glr-core/src/lib.rs`):
   - State transitions
   - Action lookups
   - Token processing

2. **Fork/Merge Operations** (`glr-core/src/lib.rs`):
   - Stack forking on conflicts
   - Stack merging on convergence
   - Stack management

3. **TreeNode Allocation** (`runtime/src/tree.rs` or `runtime2/src/tree.rs`):
   - Node creation
   - Field assignments
   - Parent/child linking

4. **Parse-Stack Operations** (`glr-core/src/lib.rs`):
   - Stack cloning
   - Stack pushing/popping
   - Stack state comparison

5. **Token Lexing** (various):
   - UTF-8 validation
   - Pattern matching
   - External scanner calls

#### Flamegraph Locations

Flamegraphs will be generated at:
- `docs/analysis/flamegraph-python-small.svg`
- `docs/analysis/flamegraph-python-medium.svg`
- `docs/analysis/flamegraph-python-large.svg`
- (Similar for JavaScript and Rust)

**Analysis Method**:
```bash
# Generate flamegraph for Python small fixture (rust-native)
cargo xtask profile cpu --language python --fixture python/small/sample.py

# Review flamegraph to identify hot functions
# Document functions with >1% CPU time width in flamegraph
```

### Preliminary Optimization Candidates

**To be identified after profiling**:
1. Function 1: TBD (Expected: Fork operations)
2. Function 2: TBD (Expected: TreeNode allocation)
3. Function 3: TBD (Expected: Action table lookups)
4. Function 4: TBD (Expected: Stack cloning)
5. Function 5: TBD (Expected: Token processing)

---

## Memory Profiling Analysis

### Allocation Patterns

**Status**: Profiling pending Week 3 Day 2

**BDD Scenario 1.2**: Memory profiling must identify allocation hotspots.

#### Expected Allocation Patterns

Based on GLR parser architecture:

1. **TreeNode Allocations**:
   - Each parse tree node is heap-allocated
   - Expected: Thousands of allocations for medium/large files
   - Target for AC-PERF3 (arena allocation)

2. **Parse-Stack Allocations**:
   - Each fork creates new stack
   - Stacks contain Vec<StackEntry>
   - Expected: Proportional to ambiguity level
   - Target for AC-PERF4 (stack pooling)

3. **Token Buffers**:
   - Input tokenization and buffering
   - UTF-8 validation overhead

4. **Metadata Allocations**:
   - Symbol tables, field maps
   - Generally one-time allocations

#### Allocation Count Baseline

**Purpose**: Establish baseline for AC-PERF3 (≥50% reduction) and AC-PERF4 (≥40% reduction)

| Category          | Small | Medium | Large | Notes |
|-------------------|-------|--------|-------|-------|
| TreeNode allocs   | TBD   | TBD    | TBD   | Target: -50% (arena) |
| Parse-stack allocs| TBD   | TBD    | TBD   | Target: -40% (pooling) |
| Token allocs      | TBD   | TBD    | TBD   | Lower priority |
| Total allocations | TBD   | TBD    | TBD   | Overall impact |

#### Memory Usage Baseline

**BDD Scenario 5.1**: Memory usage must be <10x input size.

| Language   | Size   | Input Size | Peak Memory | Ratio (Peak/Input) | Goal Met? |
|------------|--------|------------|-------------|--------------------|-----------|
| Python     | Small  | ~2 KB      | TBD         | TBD                | TBD       |
| Python     | Medium | ~20 KB     | TBD         | TBD                | TBD       |
| Python     | Large  | ~200 KB    | TBD         | TBD                | TBD       |
| JavaScript | Small  | ~3 KB      | TBD         | TBD                | TBD       |
| JavaScript | Medium | ~30 KB     | TBD         | TBD                | TBD       |
| JavaScript | Large  | ~300 KB    | TBD         | TBD                | TBD       |
| Rust       | Small  | ~2.5 KB    | TBD         | TBD                | TBD       |
| Rust       | Medium | ~25 KB     | TBD         | TBD                | TBD       |
| Rust       | Large  | ~250 KB    | TBD         | TBD                | TBD       |

**Analysis Method**:
```bash
# Generate memory profile for Python medium fixture (rust-native)
cargo xtask profile memory --language python --fixture python/medium/sample.py

# Review heaptrack/valgrind output for:
# - Peak memory usage
# - Allocation hotspots (top functions by allocation count)
# - Object lifetimes (short-lived vs long-lived)
```

### Memory Profile Locations

Memory profiles will be generated at:
- `docs/analysis/memory-python-small.txt`
- `docs/analysis/memory-python-medium.txt`
- `docs/analysis/memory-python-large.txt`
- (Similar for JavaScript and Rust)

---

## Fork-Heavy Workload Analysis

### Ambiguous Grammar Stress Test

**Purpose**: Measure parser performance on grammars with high conflict density (BDD Scenario 4.8).

**Test Grammar**: Arithmetic grammar with deliberate ambiguity (no precedence/associativity)

#### Expected Results

| Expression                  | Parse Time (µs) | Fork Count | Stack Peak | Notes |
|-----------------------------|-----------------|------------|------------|-------|
| `1 + 2`                     | TBD             | TBD        | TBD        | Minimal ambiguity |
| `1 + 2 * 3`                 | TBD             | TBD        | TBD        | Operator precedence conflict |
| `1 + 2 * 3 - 4`             | TBD             | TBD        | TBD        | Multiple conflicts |
| `1 + 2 * 3 - 4 / 5`         | TBD             | TBD        | TBD        | Deep ambiguity |
| `(1 + 2) * (3 - 4) / 5 + 6` | TBD             | TBD        | TBD        | Nested ambiguity |

**Analysis**:
- Fork count per conflict
- Stack growth rate
- Performance degradation with ambiguity level
- Memory usage under high fork load

---

## Statistical Summary

### Performance Characteristics

**To be calculated after measurement**:

- **Mean parse time**: TBD ms
- **Median parse time**: TBD ms
- **95th percentile**: TBD ms
- **Standard deviation**: TBD ms
- **Coefficient of variation**: TBD %

**Throughput**:
- **Small files**: TBD MB/s
- **Medium files**: TBD MB/s
- **Large files**: TBD MB/s

**Scalability**:
- **Time complexity**: O(?) with input size
- **Memory complexity**: O(?) with input size

---

## Performance Bottleneck Identification

### Top 5 CPU Bottlenecks

**BDD Scenario 2.1**: Document top 5 functions by CPU time.

1. **Bottleneck 1**: TBD
   - Function: TBD
   - CPU Time: TBD %
   - Analysis: TBD
   - Optimization opportunity: TBD

2. **Bottleneck 2**: TBD
   - Function: TBD
   - CPU Time: TBD %
   - Analysis: TBD
   - Optimization opportunity: TBD

3. **Bottleneck 3**: TBD
   - Function: TBD
   - CPU Time: TBD %
   - Analysis: TBD
   - Optimization opportunity: TBD

4. **Bottleneck 4**: TBD
   - Function: TBD
   - CPU Time: TBD %
   - Analysis: TBD
   - Optimization opportunity: TBD

5. **Bottleneck 5**: TBD
   - Function: TBD
   - CPU Time: TBD %
   - Analysis: TBD
   - Optimization opportunity: TBD

### Top 5 Memory Hotspots

**BDD Scenario 2.2**: Document top 5 functions by allocation count.

1. **Hotspot 1**: TBD
   - Function: TBD
   - Allocation Count: TBD
   - Analysis: TBD
   - Optimization opportunity: TBD

2. **Hotspot 2**: TBD
   - Function: TBD
   - Allocation Count: TBD
   - Analysis: TBD
   - Optimization opportunity: TBD

3. **Hotspot 3**: TBD
   - Function: TBD
   - Allocation Count: TBD
   - Analysis: TBD
   - Optimization opportunity: TBD

4. **Hotspot 4**: TBD
   - Function: TBD
   - Allocation Count: TBD
   - Analysis: TBD
   - Optimization opportunity: TBD

5. **Hotspot 5**: TBD
   - Function: TBD
   - Allocation Count: TBD
   - Analysis: TBD
   - Optimization opportunity: TBD

---

## Comparison to Performance Goals

### AC-PERF5: 2x Performance Goal

**Contract Requirement**: rust-sitter parsing time ≤ 2x Tree-sitter C (all benchmarks)

**Current Status**: Baseline pending measurement

| Language   | Size   | Tree-sitter | rust-sitter | Ratio | Goal (≤2.0x) | Status |
|------------|--------|-------------|-------------|-------|--------------|--------|
| Python     | Small  | TBD         | TBD         | TBD   | ≤2.0x        | ⏳     |
| Python     | Medium | TBD         | TBD         | TBD   | ≤2.0x        | ⏳     |
| Python     | Large  | TBD         | TBD         | TBD   | ≤2.0x        | ⏳     |
| JavaScript | Small  | TBD         | TBD         | TBD   | ≤2.0x        | ⏳     |
| JavaScript | Medium | TBD         | TBD         | TBD   | ≤2.0x        | ⏳     |
| JavaScript | Large  | TBD         | TBD         | TBD   | ≤2.0x        | ⏳     |
| Rust       | Small  | TBD         | TBD         | TBD   | ≤2.0x        | ⏳     |
| Rust       | Medium | TBD         | TBD         | TBD   | ≤2.0x        | ⏳     |
| Rust       | Large  | TBD         | TBD         | TBD   | ≤2.0x        | ⏳     |

**Pass Criteria**: All ratios ≤2.0x

### Memory Usage Goal

**Contract Requirement**: Memory usage <10x input size

| Language   | Size   | Input Size | Peak Memory | Ratio | Goal (<10x) | Status |
|------------|--------|------------|-------------|-------|-------------|--------|
| Python     | Small  | ~2 KB      | TBD         | TBD   | <10x        | ⏳     |
| Python     | Medium | ~20 KB     | TBD         | TBD   | <10x        | ⏳     |
| Python     | Large  | ~200 KB    | TBD         | TBD   | <10x        | ⏳     |
| JavaScript | Small  | ~3 KB      | TBD         | TBD   | <10x        | ⏳     |
| JavaScript | Medium | ~30 KB     | TBD         | TBD   | <10x        | ⏳     |
| JavaScript | Large  | ~300 KB    | TBD         | TBD   | <10x        | ⏳     |
| Rust       | Small  | ~2.5 KB    | TBD         | TBD   | <10x        | ⏳     |
| Rust       | Medium | ~25 KB     | TBD         | TBD   | <10x        | ⏳     |
| Rust       | Large  | ~250 KB    | TBD         | TBD   | <10x        | ⏳     |

**Pass Criteria**: All ratios <10x

---

## Baseline Validation

### Data Quality Checklist

**BDD Scenario 1.5**: Baseline data quality validation

- [ ] All benchmarks run successfully (no errors)
- [ ] Sufficient statistical significance (n≥100 samples)
- [ ] Reproducible results (coefficient of variation <5%)
- [ ] Environment variables documented
- [ ] System configuration recorded
- [ ] Profiling data captured (CPU flamegraphs + memory profiles)
- [ ] Tree-sitter comparison complete
- [ ] Results committed to version control

### Reproducibility

**Instructions for reproducing baseline**:

```bash
# 1. Set up environment
export RUST_BACKTRACE=1
export RUST_TEST_THREADS=2
export RAYON_NUM_THREADS=4

# 2. Build in release mode
cargo build --release --features glr-core

# 3. Run benchmarks (rust-native)
cargo xtask bench

# 4. Run profiling (rust-native)
cargo xtask profile cpu --language python --fixture python/medium/sample.py
cargo xtask profile memory --language python --fixture python/medium/sample.py

# 5. Run Tree-sitter comparison (rust-native)
cargo xtask compare-baseline --format markdown

# 6. Review results
ls -la docs/analysis/
```

---

## Known Limitations

### v0.7.0 Baseline Caveats

1. **GLR Runtime Status**: runtime2 GLR integration recently completed (PR #14)
   - Parser API stable but may have performance tuning opportunities
   - Forest-to-tree conversion recently optimized
   - Incremental parsing feature-gated (may affect some measurements)

2. **Test Fixtures**: Using synthetic fixtures (not production codebases)
   - Small files may not represent real-world workloads
   - Need to add medium/large fixtures in Week 3 Day 2

3. **Platform Variations**: Baseline captured on single platform
   - Cross-platform performance may vary
   - Consider recording baselines on Linux/macOS/Windows

4. **Concurrency Caps**: Tests run with capped concurrency (RUST_TEST_THREADS=2)
   - Reflects realistic test environment
   - May not represent production multi-threaded usage

---

## Next Steps

### Week 3 Day 2 (Immediate)

1. **Create Medium/Large Fixtures**:
   - Python: ~500 LOC, ~5000 LOC
   - JavaScript: ~1000 LOC, ~10K LOC
   - Rust: ~750 LOC, ~7500 LOC

2. **Run Baseline Benchmarks**:
   - Execute `cargo bench --bench glr-performance`
   - Record results in this document

3. **Run CPU Profiling** (rust-native):
   - Generate flamegraphs: `cargo xtask profile cpu --language <lang> --fixture <path>`
   - Identify top 5 CPU bottlenecks
   - Document in this file

4. **Run Memory Profiling** (rust-native):
   - Generate profiles: `cargo xtask profile memory --language <lang> --fixture <path>`
   - Identify top 5 allocation hotspots
   - Document in this file

5. **Run Tree-sitter Comparison** (rust-native):
   - Benchmark comparison: `cargo xtask compare-baseline --format markdown`
   - Generate comparison report
   - Document ratios in this file

6. **Validate Baseline**:
   - Ensure all measurements complete
   - Check data quality (reproducibility, statistical significance)
   - Commit finalized baseline to version control

### Week 3 Days 3-4 (Analysis)

1. **Analyze Profiling Data**:
   - Deep dive into hot functions and allocation patterns
   - Map bottlenecks to codebase locations
   - Estimate optimization impact

2. **Create Optimization Plan** (AC-PERF2):
   - Prioritize optimizations by impact
   - Design arena allocation strategy (AC-PERF3)
   - Design parse-stack pooling strategy (AC-PERF4)
   - Document in `docs/analysis/PERFORMANCE_ANALYSIS_V0.7.0.md`

### Week 4 (Implementation)

1. **Implement Arena Allocation** (AC-PERF3)
2. **Implement Parse-Stack Pooling** (AC-PERF4)
3. **Re-run Benchmarks** (AC-PERF5)
4. **Validate Performance Goals** (≤2x Tree-sitter C)
5. **Generate Final Report**: `docs/releases/PERFORMANCE_REPORT_V0.8.0.md`

---

## References

### Contract Documents

- [PERFORMANCE_OPTIMIZATION_CONTRACT.md](../specs/PERFORMANCE_OPTIMIZATION_CONTRACT.md) - v0.8.0 contract with acceptance criteria
- [BDD_PERFORMANCE_OPTIMIZATION.md](../plans/BDD_PERFORMANCE_OPTIMIZATION.md) - 30 BDD scenarios for testing

### Implementation Documents

- [PERFORMANCE_PLANNING_SUMMARY.md](../PERFORMANCE_PLANNING_SUMMARY.md) - Comprehensive planning summary
- [STRATEGIC_IMPLEMENTATION_PLAN.md](../plans/STRATEGIC_IMPLEMENTATION_PLAN.md) - Phase I and v0.8.0 planning

### Infrastructure

- `benches/glr-performance.rs` - Criterion benchmark suite
- `scripts/compare-tree-sitter.sh` - Tree-sitter comparison framework
- `scripts/profile-cpu.sh` - CPU profiling script
- `scripts/profile-memory.sh` - Memory profiling script
- `benches/fixtures/` - Test fixtures for benchmarking

### Related PRs

- PR #14: runtime2 GLR integration (merged)
- PR #67: External lexer utilities (merged)

---

## Document History

| Date       | Version | Changes                           | Author |
|------------|---------|-----------------------------------|--------|
| 2025-11-20 | 0.1     | Initial template created          | Claude |
| TBD        | 0.2     | Baseline measurements added       | TBD    |
| TBD        | 0.3     | Profiling analysis completed      | TBD    |
| TBD        | 1.0     | Final baseline validated          | TBD    |

---

**Status**: Template ready for Week 3 Day 2 measurements
**Next Update**: After baseline benchmarking and profiling complete
**Owner**: Performance Optimization Team (v0.8.0)
