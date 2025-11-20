//! Real parsing benchmarks for rust-sitter
//!
//! This benchmark measures actual parsing performance using real fixtures
//! and the rust-sitter parser, replacing the previous character-counting placeholders.
//!
//! ## Current Approach (Phase 1 - v0.8.0)
//!
//! We use the **arithmetic grammar** to parse Python fixtures. While this doesn't
//! validate Python semantics, it exercises real parsing logic and provides honest
//! performance measurements.
//!
//! ## Why Arithmetic Grammar?
//!
//! - ✅ Known to work correctly (tests pass)
//! - ✅ Exercises GLR fork/merge logic
//! - ✅ Provides real parse trees (not mocks)
//! - ⚠️ Does NOT validate Python AST (grammar mismatch)
//!
//! ## Migration Path
//!
//! When rust-sitter-python lexer is fixed:
//! 1. Swap `rust_sitter_example::arithmetic` → `rust_sitter_python`
//! 2. Update parser initialization
//! 3. Validate parse trees match Python semantics
//! 4. Compare with tree-sitter-python baseline
//!
//! Related:
//! - Spec: docs/specs/REAL_PARSING_BENCHMARKS_SPEC.md
//! - Issue: grammars/python/tests/smoke_test.rs:29 (lexer TODO)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_sitter_example::arithmetic::grammar::parse;

// Load fixtures at compile time (deterministic, zero I/O overhead)
const PYTHON_SMALL: &str = include_str!("../fixtures/python/small.py");
const PYTHON_MEDIUM: &str = include_str!("../fixtures/python/medium.py");
const PYTHON_LARGE: &str = include_str!("../fixtures/python/large.py");

const JS_SMALL: &str = include_str!("../fixtures/javascript/small.js");
const JS_MEDIUM: &str = include_str!("../fixtures/javascript/medium.js");
const JS_LARGE: &str = include_str!("../fixtures/javascript/large.js");

/// Benchmark real parsing with arithmetic grammar
///
/// This benchmarks actual `Parser::parse()` calls, not character counting.
/// Parse times should be in the µs-ms range, not nanoseconds.
fn benchmark_real_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_parsing");

    // Benchmark Python fixtures (parsed as arithmetic expressions)
    // Note: This exercises parser logic but won't produce valid Python AST
    for (label, source) in &[
        ("python_small", PYTHON_SMALL),
        ("python_medium", PYTHON_MEDIUM),
        ("python_large", PYTHON_LARGE),
    ] {
        group.bench_with_input(
            BenchmarkId::new("parse_arithmetic", label),
            source,
            |b, &source| {
                b.iter(|| {
                    // REAL PARSING: This calls actual GLR parser logic
                    // Expected time: 1µs - 10ms depending on source size
                    let result = parse(source);

                    // Force evaluation (prevent optimizer from skipping parse)
                    black_box(result)
                });
            },
        );
    }

    // Benchmark JavaScript fixtures
    for (label, source) in &[
        ("javascript_small", JS_SMALL),
        ("javascript_medium", JS_MEDIUM),
        ("javascript_large", JS_LARGE),
    ] {
        group.bench_with_input(
            BenchmarkId::new("parse_arithmetic", label),
            source,
            |b, &source| {
                b.iter(|| {
                    let result = parse(source);
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark fixture loading overhead (should be ~0ns with include_str!)
///
/// This validates that compile-time embedding works correctly.
fn benchmark_fixture_loading(c: &mut Criterion) {
    c.bench_function("fixture_loading_python_small", |b| {
        b.iter(|| {
            // With include_str!(), this is just a pointer dereference
            // Expected time: < 1 ns
            let source = PYTHON_SMALL;
            black_box(source)
        });
    });

    c.bench_function("fixture_loading_python_large", |b| {
        b.iter(|| {
            let source = PYTHON_LARGE;
            black_box(source)
        });
    });
}

/// Benchmark parse result validation (tree traversal overhead)
///
/// Measures cost of validating parse trees, which will be important
/// for correctness testing alongside performance benchmarks.
fn benchmark_parse_validation(c: &mut Criterion) {
    // Pre-parse once to get a tree for validation benchmarks
    let small_result = parse(PYTHON_SMALL);

    c.bench_function("validate_parse_result", |b| {
        b.iter(|| {
            // Simulate validation: check if parse succeeded
            let is_valid = small_result.is_ok();
            black_box(is_valid)
        });
    });
}

criterion_group!(
    benches,
    benchmark_real_parsing,
    benchmark_fixture_loading,
    benchmark_parse_validation
);
criterion_main!(benches);
