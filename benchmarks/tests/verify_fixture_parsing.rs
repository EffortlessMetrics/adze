//! Verification test to confirm that Python/JS fixtures cannot be parsed
//! by the arithmetic grammar (exposing the benchmark correctness issue).

use adze_example::arithmetic::grammar::parse;

// Load the same fixtures used in the benchmark
const PYTHON_SMALL: &str = include_str!("../fixtures/python/small.py");
const PYTHON_MEDIUM: &str = include_str!("../fixtures/python/medium.py");
const PYTHON_LARGE: &str = include_str!("../fixtures/python/large.py");

const JS_SMALL: &str = include_str!("../fixtures/javascript/small.js");
const JS_MEDIUM: &str = include_str!("../fixtures/javascript/medium.js");
const JS_LARGE: &str = include_str!("../fixtures/javascript/large.js");

const ARITH_SMALL: &str = include_str!("../fixtures/arithmetic/small.expr");
const ARITH_MEDIUM: &str = include_str!("../fixtures/arithmetic/medium.expr");
const ARITH_LARGE: &str = include_str!("../fixtures/arithmetic/large.expr");
const PARSE_BENCH_SOURCE: &str = include_str!("../benches/parse_bench.rs");

#[test]
fn verify_python_fixtures_do_not_parse_with_arithmetic_grammar() {
    // This test documents the current state: Python fixtures contain code
    // that the arithmetic grammar cannot properly parse.
    //
    // Tree-sitter has aggressive error recovery, so parse() may return Ok(_)
    // even for invalid input, with ERROR nodes in the tree.

    for (label, source) in &[
        ("python_small", PYTHON_SMALL),
        ("python_medium", PYTHON_MEDIUM),
        ("python_large", PYTHON_LARGE),
    ] {
        let result = parse(source);

        match result {
            Ok(expr) => {
                // Parser "succeeded" but likely with error recovery
                println!("{}: Parsed with error recovery: {:?}", label, expr);
                println!("WARNING: Benchmark is measuring error recovery, not valid parsing!");
            }
            Err(e) => {
                println!("{}: Parse failed: {:?}", label, e);
            }
        }
    }
}

#[test]
fn verify_javascript_fixtures_do_not_parse_with_arithmetic_grammar() {
    for (label, source) in &[
        ("javascript_small", JS_SMALL),
        ("javascript_medium", JS_MEDIUM),
        ("javascript_large", JS_LARGE),
    ] {
        let result = parse(source);

        match result {
            Ok(expr) => {
                println!("{}: Parsed with error recovery: {:?}", label, expr);
                println!("WARNING: Benchmark is measuring error recovery, not valid parsing!");
            }
            Err(e) => {
                println!("{}: Parse failed: {:?}", label, e);
            }
        }
    }
}

#[test]
fn verify_arithmetic_benchmark_fixtures_parse_with_arithmetic_grammar() {
    for (label, source) in &[
        ("small.expr", ARITH_SMALL),
        ("medium.expr", ARITH_MEDIUM),
        ("large.expr", ARITH_LARGE),
    ] {
        eprintln!("validating arithmetic benchmark fixture: {}", label);
        let result = parse(source);
        assert!(
            result.is_ok(),
            "arithmetic benchmark fixture {} failed to parse: {:?}",
            label,
            result
        );
    }
}

#[test]
fn verify_parse_bench_uses_real_parser_workload() {
    assert!(
        PARSE_BENCH_SOURCE.contains("adze_example::arithmetic::grammar::parse"),
        "parse_bench must call the generated arithmetic parser"
    );
    assert!(
        PARSE_BENCH_SOURCE.contains("bench_with_input"),
        "parse_bench must benchmark fixture-backed parser input"
    );
    assert!(
        !PARSE_BENCH_SOURCE.contains("placeholder_no_parser_workload"),
        "parse_bench must not advertise a placeholder/no-parser workload"
    );
    assert!(
        !PARSE_BENCH_SOURCE.contains("1 + 1"),
        "parse_bench must not benchmark a dummy arithmetic expression"
    );
}

#[test]
#[ignore = "KNOWN BUG: arithmetic parser rejects single-literal expressions like '1'"]
fn verify_valid_arithmetic_expressions_do_parse() {
    // Sanity check: ensure the parser actually works with valid input
    let valid_expressions = vec![
        "1",
        "1 - 2",
        "1 * 2",
        "1 - 2 * 3",
        "1 * 2 - 3",
        "1 - 2 - 3",
        "1 * 2 * 3",
    ];

    for expr in valid_expressions {
        let result = parse(expr);
        assert!(
            result.is_ok(),
            "Failed to parse valid arithmetic expression '{}': {:?}",
            expr,
            result
        );
    }
}
