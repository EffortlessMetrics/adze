# Performance Benchmarking Guide

This guide explains how to run, analyze, and extend rust-sitter's performance benchmarking infrastructure.

## Quick Start

```bash
# Generate valid arithmetic expression fixtures
cargo xtask generate-fixtures

# Run benchmarks and save as baseline
cargo xtask bench --save-baseline v0.8.0

# Compare against baseline (fail if regression > 5%)
cargo xtask compare-baseline v0.8.0 --threshold 5

# Profile CPU usage
cargo xtask profile cpu arithmetic large

# Profile memory usage
cargo xtask profile memory arithmetic medium
```

## Benchmark Infrastructure

### Current Benchmarks (v0.8.0)

rust-sitter uses Criterion for performance benchmarking with the following structure:

**Location**: `benchmarks/benches/glr_performance_real.rs`

**Benchmark Groups**:
- `arithmetic_parsing`: Real GLR parsing with arithmetic grammar
  - `small`: ~50 operations, ~46 µs
  - `medium`: ~200 operations, ~224 µs
  - `large`: ~1000 operations, ~1.18 ms

- `fixture_loading`: Compile-time fixture embedding verification (~1 ns)
- `validate_parse_result`: Parse result validation overhead (~350 ps)

### Fixture Generation

Fixtures are **generated**, not manually written, to ensure:
- ✅ Valid syntax for the target grammar
- ✅ Consistent structure for fair comparisons
- ✅ Scalable sizes without manual effort

**Fixture Structure**:
- **Small**: 50 operations, ~250 bytes
- **Medium**: 200 operations, ~1.5 KB
- **Large**: 1000 operations, ~6-7 KB

Generated expressions follow the pattern: `1 - 2 * 3 - 4 * 5 - 6 * 7 - ...`

This exercises:
- Left-associativity of subtraction
- Operator precedence (multiplication binds tighter)
- GLR conflict resolution
- Tree construction at scale

**Why Not Larger?**

The arithmetic grammar (and many LR parsers) have practical limits on expression depth/length.
Fixtures are sized to:
- Demonstrate scaling behavior
- Avoid hitting parser implementation limits
- Complete benchmarks in reasonable time (<5s per size)

### Commands

```bash
# Fixture Management
cargo xtask generate-fixtures              # Generate all fixtures
cargo xtask generate-fixtures --force      # Regenerate even if they exist
cargo xtask validate-fixtures              # Verify fixtures parse correctly
cargo xtask fixtures-info                  # Show fixture statistics

# Benchmarking
cargo bench -p rust-sitter-benchmarks      # Run all benchmarks
cargo bench -- arithmetic_parsing          # Run specific group
cargo bench --bench glr_performance_real   # Run specific benchmark file

# Baseline Management
cargo xtask save-baseline v0.8.0           # Save current results as baseline
cargo xtask compare-baseline v0.8.0        # Compare against baseline
cargo xtask compare-baseline v0.8.0 --threshold 10  # Custom threshold

# Profiling
cargo xtask profile cpu arithmetic large   # CPU profiling with flamegraph
cargo xtask profile memory python medium   # Memory profiling with heaptrack
cargo xtask profile cpu arithmetic small --json  # Output JSON metrics
```

## Baseline Management

Baselines are stored in `baselines/<version>.json` and contain:
- Benchmark names and IDs
- Mean, std dev, min, max times
- Iteration counts
- Criterion metadata

**Workflow**:

1. **Initial Baseline**: Establish reference performance
   ```bash
   cargo bench
   cargo xtask save-baseline v0.8.0
   ```

2. **Make Changes**: Implement optimizations, refactorings, etc.

3. **Measure Impact**: Compare against baseline
   ```bash
   cargo bench
   cargo xtask compare-baseline v0.8.0 --threshold 5
   ```

4. **Update Baseline**: If changes are intentional
   ```bash
   cargo xtask save-baseline v0.8.1
   ```

## Performance Expectations

### v0.8.0 Baseline (Arithmetic Grammar)

| Fixture | Operations | Time (µs) | Throughput |
|---------|-----------|-----------|------------|
| Small   | 50        | ~46       | ~1 op/µs   |
| Medium  | 200       | ~224      | ~0.9 op/µs |
| Large   | 1000      | ~1180     | ~0.85 op/µs|

**Scaling**: Approximately linear (O(n)) with slight overhead for larger expressions.

**Note**: These numbers measure the **arithmetic grammar** (simple binary operators).
More complex grammars (Python, JavaScript, Rust) will show different characteristics.

## Adding New Benchmarks

### 1. Create Fixtures

For language grammars, add fixture generation to `xtask/src/fixtures.rs`:

```rust
pub fn generate_python_fixtures(output_dir: &str) -> Result<()> {
    // Generate valid Python code at different scales
    // ...
}
```

### 2. Add Benchmark

In `benchmarks/benches/`:

```rust
use rust_sitter_python::grammar::parse;

const PYTHON_SMALL: &str = include_str!("../fixtures/python/small.py");
// ...

fn benchmark_python_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("python_parsing");

    for (label, source) in &[("small", PYTHON_SMALL), ...] {
        // Validate fixture parses
        assert!(parse(source).is_ok());

        group.bench_with_input(
            BenchmarkId::new("parse", label),
            source,
            |b, &source| {
                b.iter(|| black_box(parse(source).unwrap()));
            },
        );
    }

    group.finish();
}
```

### 3. Update Baseline Infrastructure

Add new benchmark to `xtask/src/baseline.rs` discovery if needed.

## Profiling

### CPU Profiling (Flamegraphs)

```bash
cargo xtask profile cpu arithmetic large
```

**Output**: `target/profile/flamegraph-cpu-arithmetic-large.svg`

**What to Look For**:
- Hot functions (wide bars)
- Unexpected call stacks
- Regex/lexer overhead
- Tree construction costs

### Memory Profiling

```bash
cargo xtask profile memory python medium
```

**Output**: `target/profile/massif-memory-python-medium.txt`

**What to Look For**:
- Peak memory usage
- Allocation patterns
- Memory leaks (growing without bound)
- Arena allocation opportunities

## CI Integration

Performance gates ensure regressions don't slip through:

**.github/workflows/performance.yml**:
```yaml
- name: Run performance benchmarks
  run: |
    cargo bench
    cargo xtask compare-baseline v0.8.0 --threshold 5
```

**Threshold Guidelines**:
- **5%**: Strict gate for critical paths
- **10%**: Reasonable for non-critical features
- **20%**: Experimental features

## Troubleshooting

### Benchmark Times Don't Scale

**Problem**: Large fixtures take same time as small fixtures.

**Cause**: Likely measuring error recovery, not actual parsing.

**Solution**:
1. Verify fixtures are **valid** for the grammar
2. Add validation assertions in benchmark setup
3. Check fixture generation logic

### Parser Errors on Generated Fixtures

**Problem**: `cargo xtask validate-fixtures` fails.

**Cause**: Generated syntax doesn't match grammar expectations.

**Solution**:
1. Review grammar rules (comments, whitespace, sequence vs single expressions)
2. Simplify generated fixtures
3. Test manually: `cargo run -p <grammar> --example parse_test`

### Inconsistent Results

**Problem**: Benchmark times vary wildly between runs.

**Cause**: System load, thermal throttling, or insufficient iterations.

**Solution**:
1. Close background applications
2. Let system cool down
3. Increase Criterion sample count
4. Run on dedicated CI infrastructure

## Best Practices

### DO:
- ✅ Generate fixtures programmatically
- ✅ Validate fixtures before benchmarking
- ✅ Use realistic input sizes (not artificial extremes)
- ✅ Document expected scaling behavior
- ✅ Compare against baselines regularly
- ✅ Profile before optimizing

### DON'T:
- ❌ Manually write large fixtures (maintenance burden)
- ❌ Benchmark error paths as if they were success paths
- ❌ Ignore regression warnings without investigation
- ❌ Optimize without measuring first
- ❌ Compare different grammars directly (apples to oranges)

## Future Work

### Planned Improvements:
1. **Language Grammar Benchmarks**: Python, JavaScript, Rust fixtures
2. **Incremental Parsing Benchmarks**: Measure edit performance
3. **Memory Benchmarks**: Track allocation counts and peak usage
4. **Comparative Benchmarks**: rust-sitter vs tree-sitter-c
5. **Stress Tests**: Pathological inputs, deeply nested structures

### Infrastructure TODOs:
- [ ] Nightly benchmark runs with artifact upload
- [ ] Performance dashboard (trend visualization)
- [ ] Automated regression bisection
- [ ] Per-PR performance reports

## References

- [Criterion Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Flamegraph Interpretation](http://www.brendangregg.com/flamegraphs.html)
- [Performance Contracts](../contracts/V0.8.0_PERFORMANCE_CONTRACT.md)
- [Baseline Format Spec](../specs/BASELINE_FORMAT_SPEC.md)
