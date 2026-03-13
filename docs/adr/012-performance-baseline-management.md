# ADR-012: Performance Baseline Management

## Status

Accepted

## Context

Parser performance is critical for Adze's use cases:
- **Editor integration**: Real-time syntax highlighting requires sub-millisecond parses
- **CI/CD pipelines**: Large codebases need fast analysis
- **WASM deployment**: Resource-constrained environments

As features are added, performance can regress unexpectedly. The project needed:
1. **Quantified baselines**: Objective performance measurements
2. **Regression detection**: CI fails on performance degradation
3. **Historical tracking**: Performance trends over time
4. **Developer workflow**: Simple commands for baseline management

### Performance Characteristics (v0.6.1-beta)

| Metric | Value | Notes |
|--------|-------|-------|
| Python parsing (1000 lines) | 62.4 µs | ~16,000 lines/sec |
| GLR fork operation | 73 ns | Sub-microsecond |
| Stack pooling speedup | 28% | Fork optimization |
| Hot path operations | 3-54 ns | Extremely efficient |

### Alternatives Considered

1. **Manual benchmarking**: Run `cargo bench` and compare manually
   - Pros: Simple, no tooling
   - Cons: Error-prone, no CI integration, easy to forget

2. **Third-party service**: Use continuous benchmarking service
   - Pros: Automated, historical graphs
   - Cons: External dependency, cost, setup complexity

3. **Custom xtask commands**: Built-in baseline management
   - Pros: Integrated, version-controlled baselines, CI-ready
   - Cons: Development effort, maintenance

## Decision

We implemented **performance baseline management** through xtask commands with CI gating.

### Command Interface

```bash
# Save baseline for version
cargo xtask bench --save-baseline v0.8.0

# Compare current against baseline
cargo xtask compare-baseline v0.8.0 --threshold 5
```

### Baseline File Format

Baselines stored in `baselines/<version>.json`:

```json
{
  "version": "v0.8.0",
  "date": "2025-11-20T19:15:00Z",
  "platform": "Linux x86_64 (Rust 1.89.0)",
  "benchmarks": {
    "real_parsing/parse_arithmetic/python_small": {
      "mean_ns": 6268.862773980828,
      "stddev_ns": 442.79896892051346,
      "samples": 100
    },
    "real_parsing/parse_arithmetic/python_medium": {
      "mean_ns": 8168.7,
      "stddev_ns": 25.8,
      "samples": 100
    }
  }
}
```

### Criterion Integration

Baselines are extracted from Criterion's `estimates.json`:

```
target/criterion/
  real_parsing/
    parse_arithmetic/
      python_small/
        base/
          estimates.json  ← Parse this
          benchmark.json
```

### Comparison Algorithm

```
For each benchmark in current results:
  1. Find matching benchmark in baseline
  2. Calculate percent change: ((current - baseline) / baseline) * 100
  3. If |percent_change| > threshold AND current > baseline:
     - Mark as regression
     - Collect for report
  4. If current < baseline:
     - Mark as improvement
     - Note in report
```

### CI Integration

```yaml
# In CI workflow
- name: Check Performance
  run: |
    cargo xtask compare-baseline v0.8.0 --threshold 5
```

**Exit codes**:
- `0`: No regressions (or only improvements)
- `1`: Regressions detected above threshold

### BDD Scenarios

```gherkin
Feature: Regression Detection

  Scenario: Detect 6% regression (threshold 5%)
    Given baseline v0.8.0 with "parse_python_small" at 100ns
    And current benchmark shows "parse_python_small" at 106ns
    When I run "cargo xtask compare-baseline v0.8.0 --threshold 5"
    Then the command exits with code 1
    And the output contains "Performance regression detected"
    And the output shows "parse_python_small: 100ns → 106ns (+6.0%)"

  Scenario: Accept 3% improvement
    Given baseline v0.8.0 with "parse_python_small" at 100ns
    And current benchmark shows "parse_python_small" at 97ns
    When I run "cargo xtask compare-baseline v0.8.0 --threshold 5"
    Then the command exits with code 0
    And the output contains "Performance improvements detected"
```

### Performance Contract

From `V0.8.0_PERFORMANCE_CONTRACT.md` (AC-PERF5):
- Baseline capture MUST save Criterion results to JSON
- Comparison MUST detect regressions above threshold
- CI MUST gate on performance regression detection

## Consequences

### Positive

- **Objective measurements**: No subjective "feels slow" assessments
- **CI integration**: Regressions caught before merge
- **Historical tracking**: Version-controlled baselines
- **Simple workflow**: Two commands cover common cases
- **Threshold flexibility**: Different thresholds for different benchmarks
- **Improvement visibility**: Positive changes are highlighted too
- **BDD alignment**: Scenarios define expected behavior

### Negative

- **Platform variance**: Benchmarks vary across hardware
- **Noise sensitivity**: Short benchmarks have high variance
- **Baseline maintenance**: Need to update baselines for intentional changes
- **CI time**: Benchmarks add to CI duration
- **False positives**: Threshold tuning required

### Neutral

- **Criterion dependency**: Relies on Criterion's JSON format
- **Manual baseline updates**: Intentional improvements need new baseline
- **Platform-specific baselines**: May need per-platform baselines

## Related

- Related ADRs: [ADR-007](007-bdd-framework-for-parser-testing.md)
- Reference: [docs/archive/specs/BASELINE_MANAGEMENT_SPEC.md](../archive/specs/BASELINE_MANAGEMENT_SPEC.md)
- Reference: [docs/archive/PERFORMANCE_BASELINE.md](../archive/PERFORMANCE_BASELINE.md)
- Reference: [docs/archive/contracts/V0.8.0_PERFORMANCE_CONTRACT.md](../archive/contracts/V0.8.0_PERFORMANCE_CONTRACT.md)
- Reference: [baselines/](../../baselines/) - Stored baseline files
