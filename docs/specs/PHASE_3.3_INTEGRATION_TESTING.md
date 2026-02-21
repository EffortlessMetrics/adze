# Phase 3.3: GLR Runtime Integration Testing

**Status**: ACTIVE
**Dependencies**: Phase 3.2 Complete ✅
**Objective**: Validate end-to-end GLR parsing pipeline with real grammars
**Timeline**: 3-4 days
**Date**: 2025-11-19

---

## Executive Summary

Phase 3.2 successfully integrated the Tokenizer and ForestConverter into the Parser pipeline. Phase 3.3 validates this integration works correctly with real-world grammars by:

1. **Testing with ambiguous grammars** that GLR is designed to handle
2. **Comparing GLR vs LR output** for unambiguous grammars (parity testing)
3. **Performance benchmarking** to establish baselines
4. **Memory profiling** to ensure reasonable resource usage

### Current State (Phase 3.2 Complete)
- ✅ Tokenizer: 11 tests passing
- ✅ ForestConverter: 13 tests passing
- ✅ Parser Integration: 5 integration tests passing
- ✅ Total: 62 runtime2 tests passing
- ✅ API: `set_token_patterns()` working

### Target State (Phase 3.3 Complete)
- ✅ Example grammars parse with `pure-rust-glr` feature
- ✅ Ambiguous grammars produce valid parse trees
- ✅ GLR output matches LR output for unambiguous grammars
- ✅ Performance benchmarks established
- ✅ Memory usage profiled and reasonable
- ✅ Integration tests comprehensive

---

## Design Philosophy

Following **contract-first, test-driven, documentation-driven** development:

1. **Specify** contracts and success criteria BEFORE implementation
2. **Test** with BDD scenarios and contract tests
3. **Document** as we build, maintaining single source of truth
4. **Validate** at each step with measurable metrics

---

## Component 1: Example Grammar Integration

### Objective
Update example grammars to work with `pure-rust-glr` feature and validate parsing.

### Success Criteria

**Contract**: Example grammars parse correctly with GLR runtime

**Preconditions**:
- Example grammar has valid AST types
- Grammar compiles without errors
- Tests are not currently disabled

**Postconditions**:
- Grammar parses with `--features pure-rust-glr`
- All existing tests pass
- New GLR-specific tests added (if grammar is ambiguous)

**Invariants**:
- Output Tree structure matches expected AST
- No regressions in existing functionality
- Error messages remain helpful

### Grammars to Test

#### 1. Unambiguous Grammars (LR Parity)

These should produce identical results in GLR and LR modes:

**arithmetic.rs** ✅ Priority 1
- Has precedence and associativity annotations
- Should parse deterministically
- Tests: Addition, multiplication, precedence, associativity

**repetitions.rs**
- Tests REPEAT, REPEAT1
- Should parse deterministically
- Tests: Empty list, single item, multiple items

**optionals.rs**
- Tests OPTIONAL
- Should parse deterministically
- Tests: Present, absent

#### 2. Ambiguous Grammars (GLR Specific)

These require GLR to handle conflicts:

**ambiguous_expr.rs** ✅ Priority 1
- Expression grammar WITHOUT precedence
- Expected: Multiple shift/reduce conflicts
- Tests: Multiple parse trees, forest disambiguation

**dangling_else.rs** ✅ Priority 1
- Classic if/if-else ambiguity
- Expected: Shift/reduce conflict on "else"
- Tests: Both interpretations valid, disambiguation selects one

**ambiguous.rs**
- Simple ambiguous grammar
- Tests: Basic GLR fork/merge behavior

### Implementation Tasks

#### Task 1.1: Arithmetic Grammar GLR Support

**File**: `example/src/arithmetic.rs`

**Changes Needed**:
1. Add test with `#[cfg(feature = "pure-rust-glr")]`
2. Create GLR-specific parse test
3. Compare GLR and LR output for same input

**Test Code**:
```rust
#[cfg(all(test, feature = "pure-rust-glr"))]
mod glr_tests {
    use super::*;

    #[test]
    fn test_arithmetic_glr_matches_lr() {
        // Parse with GLR
        let glr_result = parse("1 + 2 * 3").expect("GLR parse failed");

        // Verify structure: 1 + (2 * 3) due to precedence
        match glr_result {
            Expr::Add(
                box Expr::Num(1),
                _,
                box Expr::Mul(box Expr::Num(2), _, box Expr::Num(3))
            ) => {
                // Correct precedence
            }
            _ => panic!("GLR produced wrong parse tree: {:?}", glr_result),
        }
    }

    #[test]
    fn test_arithmetic_associativity_glr() {
        // Left-associative: ((1 + 2) + 3)
        let result = parse("1 + 2 + 3").unwrap();
        match result {
            Expr::Add(
                box Expr::Add(box Expr::Num(1), _, box Expr::Num(2)),
                _,
                box Expr::Num(3)
            ) => {
                // Correct left-associativity
            }
            _ => panic!("Wrong associativity: {:?}", result),
        }
    }
}
```

**Success Criteria**:
- [ ] Arithmetic grammar compiles with `pure-rust-glr`
- [ ] All existing tests pass
- [ ] GLR tests pass and verify precedence
- [ ] No regressions

#### Task 1.2: Ambiguous Expression GLR Support

**File**: `example/src/ambiguous_expr.rs`

**Changes Needed**:
1. Enable GLR feature
2. Add tests that verify ambiguity is preserved
3. Test disambiguation strategies

**Test Code**:
```rust
#[cfg(all(test, feature = "pure-rust-glr"))]
mod glr_ambiguity_tests {
    use super::*;
    use adze_runtime::Parser;
    use adze_runtime::forest_converter::DisambiguationStrategy;

    #[test]
    fn test_ambiguous_expr_produces_tree() {
        // Parse ambiguous expression (no precedence)
        let result = parse("1 + 2 + 3");

        // Should succeed (pick one valid parse)
        assert!(result.is_ok());

        // Verify it's a valid expression
        let expr = result.unwrap();
        assert!(matches!(expr, Expr::Binary(_, _, _)));
    }

    #[test]
    fn test_ambiguous_expr_both_parses_valid() {
        // For "1 + 2 + 3" both parses are valid:
        // 1. ((1 + 2) + 3)
        // 2. (1 + (2 + 3))

        // GLR should handle this without error
        let result = parse("1 + 2 + 3");
        assert!(result.is_ok(), "GLR should handle ambiguity");
    }
}
```

**Success Criteria**:
- [ ] Ambiguous expressions parse without error
- [ ] GLR produces valid parse tree
- [ ] Disambiguation selects valid interpretation
- [ ] Tests document ambiguity behavior

#### Task 1.3: Dangling Else GLR Support

**File**: `example/src/dangling_else.rs`

**Test Code**:
```rust
#[cfg(all(test, feature = "pure-rust-glr"))]
mod glr_dangling_else_tests {
    use super::*;

    #[test]
    fn test_dangling_else_parses() {
        // Classic: "if a if b c else d"
        // Two interpretations:
        // 1. if a { if b c else d }  (else binds to inner if)
        // 2. if a { if b c } else d  (else binds to outer if)

        let result = parse("if a if b c else d");

        // GLR should succeed (pick one interpretation)
        assert!(result.is_ok());
    }

    #[test]
    fn test_unambiguous_if_else() {
        // Unambiguous: only one valid parse
        let result = parse("if a c else d");
        assert!(result.is_ok());

        match result.unwrap() {
            Stmt::IfElse(_, box Stmt::Expr(_), box Stmt::Expr(_)) => {
                // Correct
            }
            _ => panic!("Wrong parse structure"),
        }
    }
}
```

**Success Criteria**:
- [ ] Dangling else parses correctly
- [ ] GLR handles ambiguity
- [ ] Disambiguation is documented
- [ ] Tests cover both ambiguous and unambiguous cases

---

## Component 2: GLR vs LR Parity Testing

### Objective
Verify GLR produces identical output to LR for unambiguous grammars.

### Success Criteria

**Contract**: For unambiguous grammars, GLR ≡ LR

**Preconditions**:
- Grammar has no conflicts (or all resolved by precedence)
- Same input provided to both parsers
- Both parsers configured identically

**Postconditions**:
- Parse trees are structurally identical
- Node types match exactly
- Byte ranges match
- Field assignments match

**Test Strategy**:
Use property-based testing to verify parity across many inputs.

### Implementation Tasks

#### Task 2.1: Parity Test Framework

**File**: `runtime2/tests/glr_lr_parity_test.rs`

**Test Code**:
```rust
//! GLR vs LR Parity Tests
//!
//! Contract: For unambiguous grammars, GLR and LR produce identical trees.

#[cfg(all(test, feature = "pure-rust-glr"))]
mod parity_tests {
    use adze_runtime::{Parser, Language};

    /// Test helper: Compare GLR and LR parse trees
    fn assert_trees_equal(input: &str, grammar: &str) {
        // Parse with LR mode
        let lr_tree = parse_with_lr(input, grammar);

        // Parse with GLR mode
        let glr_tree = parse_with_glr(input, grammar);

        // Compare structure
        assert_eq!(
            lr_tree.root_kind(),
            glr_tree.root_kind(),
            "Root node types differ"
        );

        assert_eq!(
            lr_tree.root_node().child_count(),
            glr_tree.root_node().child_count(),
            "Child counts differ"
        );

        // Recursively compare trees
        assert_subtrees_equal(
            &lr_tree.root_node(),
            &glr_tree.root_node()
        );
    }

    #[test]
    fn test_arithmetic_parity() {
        // Test various arithmetic expressions
        let inputs = [
            "1 + 2",
            "1 * 2 + 3",
            "1 + 2 * 3",
            "(1 + 2) * 3",
            "1 + 2 + 3 + 4",
        ];

        for input in inputs {
            assert_trees_equal(input, "arithmetic");
        }
    }

    #[test]
    fn test_repetitions_parity() {
        let inputs = [
            "a",
            "a a a",
            "",  // Empty repetition
        ];

        for input in inputs {
            assert_trees_equal(input, "repetitions");
        }
    }

    #[test]
    fn test_optionals_parity() {
        let inputs = [
            "if a b",      // With optional
            "if a",        // Without optional
        ];

        for input in inputs {
            assert_trees_equal(input, "optionals");
        }
    }
}
```

**Success Criteria**:
- [ ] Parity test framework implemented
- [ ] All unambiguous grammars tested
- [ ] All parity tests pass
- [ ] Property-based tests added (optional)

---

## Component 3: Performance Benchmarking

### Objective
Establish performance baselines and ensure GLR is within acceptable overhead.

### Success Criteria

**Contract**: GLR performance is reasonable

**Performance Targets**:
- Unambiguous grammars: GLR ≤ 2x slower than LR
- Ambiguous grammars: GLR completes within timeout
- Memory: GLR uses ≤ 10x memory of input size
- Large files: GLR parses 1MB file in < 5 seconds

### Implementation Tasks

#### Task 3.1: Performance Benchmark Suite

**File**: `benches/glr_performance.rs`

**Benchmark Code**:
```rust
//! GLR Performance Benchmarks
//!
//! Establishes performance baselines for GLR runtime.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use adze_runtime::Parser;

fn bench_arithmetic_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("arithmetic_simple");

    // Benchmark LR mode
    group.bench_function("lr", |b| {
        b.iter(|| {
            let result = parse_lr(black_box("1 + 2 * 3"));
            black_box(result);
        });
    });

    // Benchmark GLR mode
    #[cfg(feature = "pure-rust-glr")]
    group.bench_function("glr", |b| {
        b.iter(|| {
            let result = parse_glr(black_box("1 + 2 * 3"));
            black_box(result);
        });
    });

    group.finish();
}

fn bench_arithmetic_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("arithmetic_complex");

    let complex_expr = "1 + 2 * 3 + 4 * 5 + 6 * 7 + 8 * 9 + 10 * 11";

    group.bench_function("lr", |b| {
        b.iter(|| {
            let result = parse_lr(black_box(complex_expr));
            black_box(result);
        });
    });

    #[cfg(feature = "pure-rust-glr")]
    group.bench_function("glr", |b| {
        b.iter(|| {
            let result = parse_glr(black_box(complex_expr));
            black_box(result);
        });
    });

    group.finish();
}

fn bench_ambiguous_expression(c: &mut Criterion) {
    #[cfg(feature = "pure-rust-glr")]
    {
        let mut group = c.benchmark_group("ambiguous");

        group.bench_function("ambiguous_expr_glr", |b| {
            b.iter(|| {
                let result = parse_ambiguous_glr(black_box("1 + 2 + 3"));
                black_box(result);
            });
        });

        group.finish();
    }
}

criterion_group!(
    benches,
    bench_arithmetic_simple,
    bench_arithmetic_complex,
    bench_ambiguous_expression
);
criterion_main!(benches);
```

**Success Criteria**:
- [ ] Benchmark suite implemented
- [ ] Baselines established for all grammars
- [ ] GLR overhead documented
- [ ] Performance targets met (or documented if not)

#### Task 3.2: Large File Performance Test

**File**: `runtime2/tests/performance_large_file_test.rs`

**Test Code**:
```rust
#[cfg(all(test, feature = "pure-rust-glr"))]
mod large_file_tests {
    use std::time::Instant;

    #[test]
    fn test_glr_large_file_performance() {
        // Generate large input (1MB)
        let large_input = generate_arithmetic_expr(1_000_000);

        let start = Instant::now();
        let result = parse_glr(&large_input);
        let duration = start.elapsed();

        assert!(result.is_ok(), "Parse should succeed");
        assert!(
            duration.as_secs() < 5,
            "Parse took {:?}, expected < 5s",
            duration
        );
    }

    #[test]
    fn test_glr_memory_usage() {
        use memory_stats::memory_stats;

        let before = memory_stats().unwrap();

        // Parse large input
        let input = generate_arithmetic_expr(100_000);
        let _result = parse_glr(&input);

        let after = memory_stats().unwrap();
        let used = after.physical_mem - before.physical_mem;

        // Memory usage should be ≤ 10x input size
        assert!(
            used <= input.len() * 10,
            "Used {}MB for {}KB input",
            used / 1_000_000,
            input.len() / 1_000
        );
    }
}
```

**Success Criteria**:
- [ ] Large file test passes
- [ ] Memory usage within limits
- [ ] Performance documented

---

## Component 4: Memory Profiling

### Objective
Profile memory usage and ensure GSS doesn't grow unbounded.

### Success Criteria

**Contract**: Memory usage is reasonable and bounded

**Memory Targets**:
- Simple grammar: ≤ 1MB overhead
- Ambiguous grammar: ≤ 10MB overhead
- Large file (1MB): ≤ 10MB total memory
- No memory leaks

### Implementation Tasks

#### Task 4.1: Memory Profiling Test Suite

**File**: `runtime2/tests/memory_profile_test.rs`

**Test Code**:
```rust
#[cfg(all(test, feature = "pure-rust-glr"))]
mod memory_tests {
    use memory_stats::memory_stats;

    #[test]
    fn test_no_memory_leaks() {
        let before = memory_stats().unwrap().physical_mem;

        // Parse many times
        for _ in 0..1000 {
            let _result = parse_glr("1 + 2 * 3");
        }

        let after = memory_stats().unwrap().physical_mem;
        let leaked = (after as i64) - (before as i64);

        // Allow some growth but not unbounded
        assert!(
            leaked < 10_000_000,  // 10MB
            "Memory leaked: {} bytes",
            leaked
        );
    }

    #[test]
    fn test_gss_bounded() {
        // Test that GSS doesn't grow unbounded with conflicts
        let ambiguous_input = "1 + 2 + 3 + 4 + 5";  // Multiple conflicts

        let before = memory_stats().unwrap().physical_mem;
        let _result = parse_ambiguous_glr(ambiguous_input);
        let after = memory_stats().unwrap().physical_mem;

        let used = after - before;

        // Should use reasonable memory despite conflicts
        assert!(
            used < 5_000_000,  // 5MB
            "GSS used too much memory: {} bytes",
            used
        );
    }
}
```

**Success Criteria**:
- [ ] Memory profiling tests pass
- [ ] No memory leaks detected
- [ ] GSS memory bounded
- [ ] Profile documented

---

## Component 5: End-to-End Integration Tests

### Objective
Comprehensive end-to-end tests validating full pipeline.

### Test Scenarios

#### Scenario 1: Simple Unambiguous Parse
**Given** an arithmetic grammar with precedence
**When** I parse "1 + 2 * 3"
**Then** the result should be `1 + (2 * 3)`

#### Scenario 2: Ambiguous Expression Handling
**Given** an ambiguous expression grammar
**When** I parse "1 + 2 + 3"
**Then** GLR produces a valid tree without error

#### Scenario 3: Dangling Else Disambiguation
**Given** a grammar with if/if-else
**When** I parse "if a if b c else d"
**Then** GLR selects one valid interpretation

#### Scenario 4: Large File Parsing
**Given** a 1MB arithmetic expression
**When** I parse with GLR
**Then** parsing completes in < 5 seconds

#### Scenario 5: Memory Efficiency
**Given** parsing 1000 expressions
**When** I monitor memory usage
**Then** memory doesn't leak unbounded

#### Scenario 6: Error Recovery
**Given** invalid input "1 + +"
**When** I parse with GLR
**Then** I get a helpful error message

**File**: `runtime2/tests/phase_3_3_e2e_integration_test.rs`

**Test Implementation**:
```rust
//! Phase 3.3 End-to-End Integration Tests
//!
//! Comprehensive tests validating the full GLR pipeline.

#[cfg(feature = "pure-rust-glr")]
mod e2e_tests {
    use adze_runtime::Parser;

    /// Scenario 1: Simple Unambiguous Parse
    #[test]
    fn scenario_1_simple_unambiguous() {
        // Given: arithmetic grammar with precedence
        let result = parse_arithmetic("1 + 2 * 3");

        // Then: result is 1 + (2 * 3)
        assert!(result.is_ok());
        let expr = result.unwrap();

        // Verify precedence: multiplication before addition
        match expr {
            Expr::Add(
                box Expr::Num(1),
                _,
                box Expr::Mul(box Expr::Num(2), _, box Expr::Num(3))
            ) => {
                // Success
            }
            _ => panic!("Wrong precedence: {:?}", expr),
        }
    }

    /// Scenario 2: Ambiguous Expression Handling
    #[test]
    fn scenario_2_ambiguous_handling() {
        // Given: ambiguous expression grammar (no precedence)
        let result = parse_ambiguous("1 + 2 + 3");

        // Then: GLR produces valid tree
        assert!(result.is_ok(), "GLR should handle ambiguity");

        let expr = result.unwrap();
        assert!(matches!(expr, Expr::Binary(_, _, _)));
    }

    /// Scenario 3: Dangling Else Disambiguation
    #[test]
    fn scenario_3_dangling_else() {
        // Given: if/if-else grammar
        let result = parse_dangling_else("if a if b c else d");

        // Then: GLR selects valid interpretation
        assert!(result.is_ok());
    }

    /// Scenario 4: Large File Parsing
    #[test]
    fn scenario_4_large_file() {
        use std::time::Instant;

        // Given: 1MB expression
        let large_expr = generate_arithmetic(1_000_000);

        // When: parse with GLR
        let start = Instant::now();
        let result = parse_arithmetic(&large_expr);
        let duration = start.elapsed();

        // Then: completes in < 5s
        assert!(result.is_ok());
        assert!(duration.as_secs() < 5);
    }

    /// Scenario 5: Memory Efficiency
    #[test]
    fn scenario_5_memory_efficiency() {
        use memory_stats::memory_stats;

        let before = memory_stats().unwrap().physical_mem;

        // Parse 1000 expressions
        for _ in 0..1000 {
            let _result = parse_arithmetic("1 + 2 * 3");
        }

        let after = memory_stats().unwrap().physical_mem;
        let leaked = (after as i64) - (before as i64);

        // Memory shouldn't leak
        assert!(leaked < 10_000_000);
    }

    /// Scenario 6: Error Recovery
    #[test]
    fn scenario_6_error_recovery() {
        // Given: invalid input
        let result = parse_arithmetic("1 + +");

        // Then: helpful error message
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Syntax error"));
    }
}
```

---

## Success Metrics

### Phase 3.3 Complete When:

#### Functional Requirements
- [ ] All example grammars parse with `pure-rust-glr`
- [ ] Ambiguous grammars (ambiguous_expr, dangling_else) work
- [ ] GLR matches LR for unambiguous grammars (parity tests pass)
- [ ] All integration tests pass (6 E2E scenarios)

#### Performance Requirements
- [ ] GLR ≤ 2x slower than LR for unambiguous grammars
- [ ] Large file (1MB) parses in < 5 seconds
- [ ] Memory usage ≤ 10x input size
- [ ] No memory leaks detected

#### Quality Requirements
- [ ] Test coverage >80%
- [ ] All tests passing
- [ ] Documentation complete
- [ ] Performance baselines established

---

## Documentation Updates

### Files to Update

#### CLAUDE.md
- Update Phase 3.3 status to COMPLETE
- Add performance benchmarks
- Document GLR vs LR comparison

#### PHASE_3_PURE_RUST_GLR_RUNTIME.md
- Mark Phase 3.3 complete
- Add performance results
- Document integration test results

#### README.md
- Add GLR performance numbers
- Update examples with GLR feature

---

## Test Plan Summary

| Component | Test Count | Status |
|-----------|------------|--------|
| Example Grammars | 6 | Pending |
| Parity Tests | 10+ | Pending |
| Performance Benchmarks | 5 | Pending |
| Memory Tests | 3 | Pending |
| E2E Integration | 6 | Pending |
| **TOTAL** | **30+** | **Pending** |

---

## Timeline

| Task | Estimated Time | Dependencies |
|------|----------------|--------------|
| Component 1: Example Grammars | 6 hours | None |
| Component 2: Parity Tests | 4 hours | Component 1 |
| Component 3: Performance | 4 hours | Component 1 |
| Component 4: Memory | 2 hours | Component 3 |
| Component 5: E2E Tests | 4 hours | All above |
| Documentation | 2 hours | All above |
| **TOTAL** | **22 hours** | **3-4 days** |

---

## Next Steps

1. ✅ Phase 3.3 specification complete
2. → Implement Component 1: Example Grammar Integration
3. → Execute test plan systematically
4. → Validate all success criteria
5. → Update documentation
6. → Move to Phase 3.4: Documentation and Stabilization

---

**Status**: Phase 3.3 Specification Complete - Ready for Implementation
**Next**: Begin Component 1 - Example Grammar Integration
**Timeline**: 3-4 days to full integration testing completion
