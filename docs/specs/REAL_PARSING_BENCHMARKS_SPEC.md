# Real Parsing Benchmarks Specification

**Version**: 1.0.0
**Date**: 2025-11-20
**Status**: ACTIVE
**Related Contract**: [V0.8.0_PERFORMANCE_CONTRACT.md](../contracts/V0.8.0_PERFORMANCE_CONTRACT.md)

---

## Executive Summary

This specification defines how benchmarks transition from placeholder logic (character counting) to real parsing, enabling honest performance measurement and comparison with Tree-sitter.

**Problem**: Current benchmarks use character iteration, producing false performance claims (100x faster than Tree-sitter).

**Solution**: Use actual rust-sitter parsers with real fixtures, measure true parse time.

---

## Requirements

### REQ-1: Real Parsing (MUST)

**Requirement**: Benchmarks MUST invoke actual parser logic, not simulations.

**Acceptance Criteria**:
- [ ] Parser is initialized with a grammar
- [ ] `parser.parse(source)` is called on real code
- [ ] Parse tree is returned and validated
- [ ] No character counting, token simulation, or placeholder logic

**BDD Scenario**:
```gherkin
Feature: Real Parsing in Benchmarks
  As a performance engineer
  I want benchmarks to measure actual parsing
  So that I can trust performance claims

  Scenario: Parse Python fixture with arithmetic grammar
    Given a Parser initialized with arithmetic grammar
    And a Python source file from fixtures/python/small.py
    When I call parser.parse(source)
    Then a valid Tree is returned
    And the tree contains real nodes (not mocked)
    And parse time is in the expected range (1µs - 100ms)
```

---

### REQ-2: Fixture Integration (MUST)

**Requirement**: Benchmarks MUST use generated fixtures from `benchmarks/fixtures/`.

**Acceptance Criteria**:
- [ ] Fixtures loaded via `include_str!()` at compile time
- [ ] Small, medium, large sizes all available
- [ ] No dynamic code generation during benchmark execution
- [ ] Deterministic: same input every run

**Implementation**:
```rust
// Load fixtures at compile time
const PYTHON_SMALL: &str = include_str!("../fixtures/python/small.py");
const PYTHON_MEDIUM: &str = include_str!("../fixtures/python/medium.py");
const PYTHON_LARGE: &str = include_str!("../fixtures/python/large.py");

fn benchmark_python_parsing(c: &mut Criterion) {
    for (label, source) in &[
        ("python_small", PYTHON_SMALL),
        ("python_medium", PYTHON_MEDIUM),
        ("python_large", PYTHON_LARGE),
    ] {
        // ... benchmark code
    }
}
```

---

### REQ-3: Grammar Selection (SHOULD)

**Requirement**: Benchmarks SHOULD use appropriate grammars for each language.

**Priority Order**:
1. **Arithmetic** (Phase 1): Known to work, simple, fast iteration
2. **Python** (Phase 2): Production grammar, when lexer is fixed
3. **JavaScript** (Phase 3): Additional language coverage

**Rationale**: Start with working grammar, expand as parsers stabilize.

**Current Decision**:
- Use **arithmetic grammar** for initial implementation
- Parse Python/JS *syntax* (treat as expression language)
- Document limitation clearly
- Swap to Python grammar when ready

**Acceptance Criteria**:
- [ ] Arithmetic parser initialized correctly
- [ ] Python source parsed as arithmetic expressions (graceful degradation)
- [ ] Clear comment explaining grammar mismatch
- [ ] TODO item for Python grammar integration

---

### REQ-4: Error Handling (MUST)

**Requirement**: Benchmarks MUST handle parse errors gracefully.

**Scenarios**:
- Valid syntax → successful parse
- Invalid syntax → parse error reported, benchmark continues
- Grammar mismatch → documented, results excluded from perf claims

**Implementation**:
```rust
let tree = match parser.parse(source) {
    Ok(tree) => tree,
    Err(e) => {
        eprintln!("Parse failed: {}. This is expected for grammar mismatches.", e);
        return; // Skip this benchmark iteration
    }
};
black_box(tree);
```

---

### REQ-5: Performance Expectations (SHOULD)

**Requirement**: Benchmark results SHOULD fall within expected ranges.

**Expected Parse Times** (for arithmetic grammar on Python syntax):
- Small (100-150 LOC): **1-10 µs** (best case: simple expressions)
- Medium (1-3k LOC): **50-500 µs** (depending on expression complexity)
- Large (5-15k LOC): **200-2000 µs** (linear or sub-linear growth)

**Red Flags** (investigate if these occur):
- **< 1 µs**: Likely still using placeholder logic
- **> 100 ms**: Parser is blocked or incorrect
- **Linear growth >> O(n)**: Algorithm issue

**Validation**:
```bash
cargo bench --bench glr_performance
# Inspect Criterion output:
# - parse_python_small: ~5 µs (OK)
# - parse_python_medium: ~150 µs (OK)
# - parse_python_large: ~800 µs (OK)
```

---

## Design Decisions

### Decision 1: Arithmetic Grammar for Phase 1

**Context**: Python parser has known lexer issues (tests are ignored).

**Options**:
1. Wait for Python parser to be fixed
2. Use arithmetic grammar on Python fixtures
3. Use mock parser with realistic timing

**Decision**: **Option 2 - Arithmetic grammar on Python fixtures**

**Rationale**:
- Unblocks performance work immediately
- Provides real parsing (not mocks)
- Fixtures already generated and validated
- Clear migration path when Python parser ready

**Trade-offs**:
- ✅ Real parsing logic exercised
- ✅ Honest measurement of current capabilities
- ⚠️ Parse trees won't match Python semantics
- ⚠️ Can't compare directly with tree-sitter-python yet

**Documentation**:
```rust
// NOTE: Currently using arithmetic grammar on Python fixtures.
// This exercises real parsing logic but doesn't validate Python semantics.
// TODO (#123): Switch to rust-sitter-python when lexer is fixed.
```

---

### Decision 2: Include Fixtures at Compile Time

**Context**: Fixtures could be loaded at runtime or compile time.

**Options**:
1. Load from filesystem during benchmark (`std::fs::read_to_string()`)
2. Embed via `include_str!()` at compile time

**Decision**: **Option 2 - Compile-time embedding**

**Rationale**:
- **Determinism**: Same binary always benchmarks same input
- **Performance**: No I/O overhead in benchmark loop
- **Simplicity**: No path resolution or error handling needed
- **Portability**: Benchmarks run in any directory

**Trade-offs**:
- ✅ Zero I/O overhead
- ✅ Deterministic results
- ⚠️ Increases binary size (~600 KB for all fixtures)
- ⚠️ Requires rebuild to update fixtures

---

### Decision 3: Criterion for Benchmarking

**Context**: Rust has multiple benchmark frameworks.

**Options**:
1. `cargo bench` with built-in framework
2. Criterion (already in use)
3. Custom timing harness

**Decision**: **Option 2 - Criterion (retain current choice)**

**Rationale**:
- Already integrated in project
- Statistical analysis (mean, stddev, outlier detection)
- Comparison with baselines
- HTML report generation
- Industry standard for Rust

---

## Implementation Plan

### Phase 1: Arithmetic Grammar (This PR)

**Goal**: Get real parsing working with minimal changes.

**Tasks**:
1. ✅ Generate fixtures (DONE - Task 1.1)
2. ⏳ Wire arithmetic parser into benchmarks
3. ⏳ Load fixtures via `include_str!()`
4. ⏳ Remove character-counting logic
5. ⏳ Run benchmarks, verify realistic times
6. ⏳ Document grammar limitation

**Files to Modify**:
- `benchmarks/benches/glr_performance.rs`
- `benchmarks/Cargo.toml` (if dependencies needed)

**Success Criteria**:
- `cargo bench --bench glr_performance` runs without errors
- Output shows parse times in µs range (not ns)
- No placeholder comments remain
- Clear TODO for Python grammar integration

---

### Phase 2: Python Grammar (Future)

**Goal**: Use correct grammar for Python fixtures.

**Blockers**:
- Python lexer issues (see `grammars/python/tests/smoke_test.rs:29`)

**Tasks**:
1. Fix Python lexer/tokenizer
2. Un-ignore Python smoke tests
3. Update benchmarks to use `rust-sitter-python`
4. Validate parse trees match expected Python AST
5. Compare with tree-sitter-python

**Estimated**: v0.9.0 or v0.10.0

---

### Phase 3: Multi-Grammar Benchmarks (Future)

**Goal**: Benchmark multiple grammars for comprehensive perf data.

**Grammars**:
- Python (production grammar)
- JavaScript (production grammar)
- Rust (if available)
- Synthetic ambiguous grammars (for GLR stress testing)

**Estimated**: v0.11.0+

---

## Testing Strategy

### Unit Tests

**Test**: Fixture loading
```rust
#[test]
fn test_fixtures_load() {
    const PYTHON_SMALL: &str = include_str!("../fixtures/python/small.py");
    assert!(!PYTHON_SMALL.is_empty());
    assert!(PYTHON_SMALL.lines().count() > 50);
}
```

### Integration Tests

**Test**: Benchmark compiles and runs
```bash
cargo bench --bench glr_performance --no-run  # Compile only
cargo bench --bench glr_performance -- --test  # Quick run
```

### Validation

**Manual Check**:
```bash
cargo bench --bench glr_performance 2>&1 | tee benchmark_output.txt
grep -E "(time:|µs)" benchmark_output.txt

# Expected:
# parse_python_small   time:   [4.123 µs ... 4.567 µs]
# parse_python_medium  time:   [156.3 µs ... 178.9 µs]
# parse_python_large   time:   [789.2 µs ... 892.1 µs]
```

---

## Success Metrics

### Quantitative

- [ ] All benchmarks use `Parser::parse()` (0% placeholder logic)
- [ ] Parse times in expected ranges (1 µs - 2 ms)
- [ ] Criterion reports generated successfully
- [ ] No benchmark failures due to parse errors

### Qualitative

- [ ] Code is clear and well-documented
- [ ] Grammar limitation documented with TODO
- [ ] Easy to swap in Python grammar when ready
- [ ] No false performance claims in output

---

## References

### Related Documents
- [V0.8.0_PERFORMANCE_CONTRACT.md](../contracts/V0.8.0_PERFORMANCE_CONTRACT.md)
- [V0.8.0_EXECUTION_PLAN.md](../sessions/V0.8.0_EXECUTION_PLAN.md)
- [PERFORMANCE_BASELINE.md](../PERFORMANCE_BASELINE.md)

### Code References
- Arithmetic grammar: `/example/src/arithmetic.rs`
- Python fixtures: `/benchmarks/fixtures/python/`
- Current benchmarks: `/benchmarks/benches/glr_performance.rs`

---

**Specification Version**: 1.0.0
**Last Updated**: 2025-11-20
**Status**: ACTIVE - Implementation in progress
**Owner**: rust-sitter performance team

---

END OF SPECIFICATION
