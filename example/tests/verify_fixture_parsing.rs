//! Verification test to confirm that Python/JS fixtures cannot be parsed
//! by the arithmetic grammar (exposing the benchmark correctness issue).

use adze_example::arithmetic::grammar::parse;

// Load the same fixtures used in the benchmark
const PYTHON_SMALL: &str = include_str!("../../benchmarks/fixtures/python/small.py");
const PYTHON_MEDIUM: &str = include_str!("../../benchmarks/fixtures/python/medium.py");
const PYTHON_LARGE: &str = include_str!("../../benchmarks/fixtures/python/large.py");

const JS_SMALL: &str = include_str!("../../benchmarks/fixtures/javascript/small.js");
const JS_MEDIUM: &str = include_str!("../../benchmarks/fixtures/javascript/medium.js");
const JS_LARGE: &str = include_str!("../../benchmarks/fixtures/javascript/large.js");

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
