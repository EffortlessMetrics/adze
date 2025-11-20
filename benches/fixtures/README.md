# Benchmark Fixtures

Test fixtures for performance benchmarking (v0.8.0 Performance Optimization).

## Directory Structure

```
fixtures/
├── python/
│   ├── small/      # <100 LOC
│   ├── medium/     # 1K-10K LOC
│   └── large/      # >10K LOC
├── javascript/
│   ├── small/
│   ├── medium/
│   └── large/
└── rust/
    ├── small/
    ├── medium/
    └── large/
```

## Size Categories

| Category | Lines of Code | Purpose |
|----------|---------------|---------|
| Small    | <100 LOC      | Quick feedback, regression detection |
| Medium   | 1K-10K LOC    | Realistic workloads, allocation patterns |
| Large    | >10K LOC      | Stress testing, memory usage, scalability |

## Current Fixtures

### Python
- `python/small/sample.py` (~50 LOC) - Basic functions and classes

### JavaScript
- `javascript/small/sample.js` (~100 LOC) - Functions, classes, utilities

### Rust
- `rust/small/sample.rs` (~75 LOC) - Functions, structs, traits

## Adding New Fixtures

**Guidelines**:
1. Use realistic, representative code (not contrived examples)
2. Include varied language features (functions, classes, control flow)
3. Avoid artificial complexity (maintain readability)
4. Document source if derived from real projects (with license)

**Medium and Large Fixtures** (to be added):
- Source from real open-source projects (with attribution)
- OR generate via script (maintain consistency)
- Ensure license compatibility

**Example Sources**:
- Python: Django, Flask, numpy
- JavaScript: React, Vue, lodash
- Rust: serde, tokio, clap

## Benchmarking Usage

Fixtures are used by:
- `benches/glr-performance.rs` - Criterion benchmark suite
- `scripts/profile-cpu.sh` - CPU profiling
- `scripts/profile-memory.sh` - Memory profiling
- `scripts/compare-tree-sitter.sh` - Tree-sitter comparison

## Baseline Measurements

Baseline measurements (v0.7.0) are documented in:
- `docs/baselines/PERFORMANCE_BASELINE_V0.7.0.md`

Target measurements (v0.8.0) will be documented in:
- `docs/reports/PERFORMANCE_REPORT_V0.8.0.md`

## License

All fixtures are part of rust-sitter and follow the same license as the project.

External fixtures (if any) retain their original licenses and include attribution.

---

**Last Updated**: November 20, 2025
**Version**: v0.8.0 (Week 3 Day 1)
