# Baseline Management Specification

**Version**: 1.0.0
**Date**: 2025-11-20
**Status**: ACTIVE
**Related Contract**: [V0.8.0_PERFORMANCE_CONTRACT.md](../contracts/V0.8.0_PERFORMANCE_CONTRACT.md) - AC-PERF5

---

## Executive Summary

This specification defines how performance baselines are captured, stored, and compared to enable CI regression detection and performance tracking over time.

**Goal**: Enable `cargo xtask bench --save-baseline v0.8.0` to save Criterion results, and `cargo xtask compare-baseline v0.8.0 --threshold 5` to detect regressions.

---

## Requirements

### REQ-BL1: Baseline Capture (MUST)

**Requirement**: `cargo xtask bench --save-baseline <version>` MUST save Criterion results to a JSON file.

**Acceptance Criteria**:
- [ ] Discovers all benchmark results in `target/criterion/`
- [ ] Parses Criterion's `estimates.json` files
- [ ] Extracts mean, std dev, sample count for each benchmark
- [ ] Saves to `baselines/<version>.json` with metadata
- [ ] Validates saved file is valid JSON

**Data Schema**:
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

**BDD Scenario**:
```gherkin
Feature: Baseline Capture
  As a developer
  I want to save benchmark results as a baseline
  So that I can track performance over time

  Scenario: Save baseline for v0.8.0
    Given I have run "cargo bench --bench glr_performance_real"
    When I run "cargo xtask bench --save-baseline v0.8.0"
    Then a file "baselines/v0.8.0.json" is created
    And the file contains valid JSON
    And the JSON has a "version" field with value "v0.8.0"
    And the JSON has a "benchmarks" object
    And each benchmark has "mean_ns", "stddev_ns", and "samples" fields
```

---

### REQ-BL2: Criterion JSON Parsing (MUST)

**Requirement**: Parse Criterion's `estimates.json` files, not text output.

**Rationale**:
- Text output is human-readable but parsing-hostile
- JSON is stable, machine-readable, and future-proof
- Avoids regex fragility and version sensitivity

**Implementation**:
```rust
// Parse estimates.json from Criterion output
#[derive(Deserialize)]
struct CriterionEstimates {
    mean: Estimate,
    std_dev: Estimate,
}

#[derive(Deserialize)]
struct Estimate {
    point_estimate: f64,
    standard_error: f64,
    confidence_interval: ConfidenceInterval,
}

fn parse_criterion_estimates(path: &Path) -> Result<BenchmarkResult> {
    let json = std::fs::read_to_string(path)?;
    let estimates: CriterionEstimates = serde_json::from_str(&json)?;

    Ok(BenchmarkResult {
        mean_ns: estimates.mean.point_estimate,
        stddev_ns: estimates.std_dev.point_estimate,
        samples: 100, // From benchmark.json or default
    })
}
```

**Acceptance Criteria**:
- [ ] Parses `estimates.json` successfully
- [ ] Extracts mean and std dev in nanoseconds
- [ ] Handles missing files gracefully (warn, skip benchmark)
- [ ] Validates JSON structure before parsing

---

### REQ-BL3: Baseline Comparison (MUST)

**Requirement**: `cargo xtask compare-baseline <version> --threshold <percent>` MUST detect regressions.

**Algorithm**:
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

**Acceptance Criteria**:
- [ ] Loads baseline JSON successfully
- [ ] Runs current benchmarks (via `cargo bench`)
- [ ] Compares each benchmark against baseline
- [ ] Detects regressions above threshold
- [ ] Exits with code 1 if regressions found
- [ ] Exits with code 0 if no regressions
- [ ] Prints clear report showing changes

**BDD Scenario**:
```gherkin
Feature: Regression Detection
  As a CI system
  I want to detect performance regressions
  So that I can prevent merging slow code

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
    And the output shows "parse_python_small: 100ns → 97ns (-3.0%)"
```

---

### REQ-BL4: Criterion Directory Walking (MUST)

**Requirement**: Discover all benchmarks in `target/criterion/` automatically.

**Directory Structure**:
```
target/criterion/
  real_parsing/
    parse_arithmetic/
      python_small/
        base/
          estimates.json  ← Parse this
          benchmark.json
      python_medium/
        base/
          estimates.json  ← Parse this
  fixture_loading_python_small/
    base/
      estimates.json      ← Parse this
```

**Implementation**:
```rust
fn discover_benchmarks(criterion_dir: &Path) -> Result<HashMap<String, BenchmarkResult>> {
    let mut benchmarks = HashMap::new();

    // Walk directory tree
    for entry in walkdir::WalkDir::new(criterion_dir) {
        let entry = entry?;
        let path = entry.path();

        // Look for base/estimates.json files
        if path.ends_with("base/estimates.json") {
            let bench_name = extract_benchmark_name(path, criterion_dir)?;
            let result = parse_criterion_estimates(path)?;
            benchmarks.insert(bench_name, result);
        }
    }

    Ok(benchmarks)
}

fn extract_benchmark_name(estimates_path: &Path, criterion_dir: &Path) -> Result<String> {
    // Convert: target/criterion/real_parsing/parse_arithmetic/python_small/base/estimates.json
    // To: real_parsing/parse_arithmetic/python_small

    let relative = estimates_path.strip_prefix(criterion_dir)?;
    let components: Vec<_> = relative.components()
        .filter(|c| c.as_os_str() != "base" && c.as_os_str() != "estimates.json")
        .collect();

    Ok(components.iter()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}
```

**Acceptance Criteria**:
- [ ] Discovers all benchmarks recursively
- [ ] Handles grouped benchmarks (e.g., `real_parsing/parse_arithmetic/python_small`)
- [ ] Handles flat benchmarks (e.g., `fixture_loading_python_small`)
- [ ] Skips `new/` directories (only uses `base/`)
- [ ] Reports count of discovered benchmarks

---

### REQ-BL5: CI Integration (SHOULD)

**Requirement**: CI workflow SHOULD run baseline comparison on every PR.

**GitHub Actions Workflow**:
```yaml
- name: Run benchmarks
  run: cargo bench --bench glr_performance_real

- name: Compare against baseline
  run: cargo xtask compare-baseline v0.8.0 --threshold 5
  continue-on-error: false  # Fail PR if regressions detected
```

**Acceptance Criteria**:
- [ ] CI workflow file created/updated
- [ ] Runs on PRs touching performance-sensitive code
- [ ] Fails build on regression
- [ ] Uploads Criterion HTML reports as artifacts

---

## Design Decisions

### Decision 1: JSON Storage Format

**Options**:
1. Store raw Criterion output (large, complex)
2. Store minimal data (mean, std dev)
3. Store full statistics (percentiles, confidence intervals)

**Chosen**: **Option 2 - Minimal data**

**Rationale**:
- Mean and std dev sufficient for regression detection
- Smaller file size (easier to review in PRs)
- Can always re-run benchmarks for full statistics

**Trade-offs**:
- ✅ Simple, readable JSON
- ✅ Fast comparison
- ⚠️ Loses detailed percentile data (acceptable - not used for regression detection)

---

### Decision 2: Baseline File Naming

**Options**:
1. `baselines/<version>.json` (e.g., `v0.8.0.json`)
2. `baselines/<version>/<platform>.json`
3. `baselines/<date>-<version>.json`

**Chosen**: **Option 1 - Simple version-based naming**

**Rationale**:
- One baseline per version simplifies comparison
- Platform differences handled via baseline metadata
- Version is the primary dimension of interest

**Trade-offs**:
- ✅ Simple, predictable file names
- ✅ Easy to find baseline for any version
- ⚠️ One platform per baseline (acceptable - CI runs on consistent platform)

---

### Decision 3: Threshold Application

**Options**:
1. Single global threshold (e.g., 5%)
2. Per-benchmark thresholds
3. Statistical significance test

**Chosen**: **Option 1 - Single global threshold (configurable)**

**Rationale**:
- Simplicity: Easy to understand and communicate
- Consistency: Same standard for all benchmarks
- Flexibility: Can override via flag if needed

**Future Enhancement**: Per-benchmark thresholds via config file if needed

---

## Implementation Plan

### Phase 1: Criterion JSON Parsing

**Files to Modify**:
- `xtask/src/baseline.rs`

**Tasks**:
1. Add `serde_json` dependency to xtask
2. Define `CriterionEstimates` struct
3. Implement `parse_criterion_estimates(path)` function
4. Add unit tests for JSON parsing

**Success Criteria**:
- Parses `estimates.json` successfully
- Extracts mean_ns and stddev_ns
- Unit tests pass

---

### Phase 2: Benchmark Discovery

**Files to Modify**:
- `xtask/src/baseline.rs`

**Tasks**:
1. Add `walkdir` dependency to xtask
2. Implement `discover_benchmarks(criterion_dir)` function
3. Implement `extract_benchmark_name(path)` function
4. Add unit tests with mock directory structure

**Success Criteria**:
- Discovers all benchmarks in test fixture
- Handles grouped and flat benchmark names correctly
- Skips `new/` directories

---

### Phase 3: Baseline Save

**Files to Modify**:
- `xtask/src/baseline.rs`
- `xtask/src/bench.rs`

**Tasks**:
1. Implement `save_baseline(version)` function
2. Create `Baseline` struct with schema
3. Write JSON to `baselines/<version>.json`
4. Add validation tests

**Success Criteria**:
- `cargo xtask bench --save-baseline v0.8.0` creates valid JSON
- Baseline file contains all discovered benchmarks
- Schema validation passes

---

### Phase 4: Baseline Compare

**Files to Modify**:
- `xtask/src/baseline.rs`

**Tasks**:
1. Implement `compare_baseline(version, threshold)` function
2. Implement regression detection algorithm
3. Implement report formatting
4. Add comparison tests

**Success Criteria**:
- Detects regressions correctly
- Exits with appropriate code (0 or 1)
- Report is clear and actionable

---

### Phase 5: CI Integration

**Files to Create/Modify**:
- `.github/workflows/performance.yml`

**Tasks**:
1. Add performance workflow
2. Configure to run on PRs
3. Set up baseline comparison
4. Upload artifacts

**Success Criteria**:
- CI runs on PRs
- Fails on regression
- Results visible in PR

---

## Testing Strategy

### Unit Tests

**Test**: JSON parsing
```rust
#[test]
fn test_parse_criterion_estimates() {
    let json = r#"{
        "mean": {"point_estimate": 6268.862},
        "std_dev": {"point_estimate": 442.798}
    }"#;

    // Write to temp file, parse, validate
    let result = parse_criterion_estimates(&temp_file).unwrap();
    assert!((result.mean_ns - 6268.862).abs() < 0.001);
}
```

**Test**: Benchmark discovery
```rust
#[test]
fn test_discover_benchmarks() {
    // Create mock directory structure
    let temp_dir = create_mock_criterion_dir();

    let benchmarks = discover_benchmarks(&temp_dir).unwrap();
    assert_eq!(benchmarks.len(), 3);
    assert!(benchmarks.contains_key("real_parsing/parse_arithmetic/python_small"));
}
```

### Integration Tests

**Test**: End-to-end save/load
```bash
cargo bench --bench glr_performance_real
cargo xtask bench --save-baseline test_v1
# Verify baselines/test_v1.json exists and is valid
```

**Test**: Regression detection
```bash
# Save baseline
cargo xtask bench --save-baseline baseline_v1

# Modify code to be slower (inject delay)
# Re-run benchmarks

# Compare - should detect regression
cargo xtask compare-baseline baseline_v1 --threshold 5
# Should exit with code 1
```

---

## Success Metrics

### Quantitative
- [ ] Parses 100% of Criterion estimates.json files
- [ ] Discovers all benchmarks (6 from glr_performance_real)
- [ ] Baseline JSON < 10 KB (minimal data)
- [ ] Comparison runs in < 5 seconds

### Qualitative
- [ ] Clear, actionable regression reports
- [ ] Easy to use (`cargo xtask` commands)
- [ ] Well-documented with examples
- [ ] Integrates seamlessly with CI

---

## References

### Related Documents
- [V0.8.0_PERFORMANCE_CONTRACT.md](../contracts/V0.8.0_PERFORMANCE_CONTRACT.md) - AC-PERF5
- [V0.8.0_EXECUTION_PLAN.md](../sessions/V0.8.0_EXECUTION_PLAN.md) - Phase 1, Task 1.3

### External References
- [Criterion.rs Output Format](https://bheisler.github.io/criterion.rs/book/user_guide/csv_output.html)
- [Criterion JSON Schema](https://github.com/bheisler/criterion.rs/blob/master/book/src/user_guide/csv_output.md)

---

**Specification Version**: 1.0.0
**Last Updated**: 2025-11-20
**Status**: ACTIVE - Implementation starting
**Owner**: adze performance team

---

END OF SPECIFICATION
