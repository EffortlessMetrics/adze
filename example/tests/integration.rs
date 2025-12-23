// Integration tests for rust-sitter example grammars
// These tests demonstrate real-world usage of macro-based grammars

#[test]
fn test_arithmetic_simple_numbers() {
    use rust_sitter_example::arithmetic::grammar::{Expression, parse};

    // Test simple number parsing
    let result = parse("42");
    assert!(result.is_ok(), "Failed to parse '42': {:?}", result.err());
    let expr = result.unwrap();
    assert_eq!(expr, Expression::Number(42));

    // Test different numbers
    let result = parse("999");
    assert!(result.is_ok(), "Failed to parse '999': {:?}", result.err());
    assert_eq!(result.unwrap(), Expression::Number(999));
}

#[test]
fn test_arithmetic_subtraction() {
    use rust_sitter_example::arithmetic::grammar::{Expression, parse};

    // Test simple subtraction
    let result = parse("10 - 5");
    assert!(
        result.is_ok(),
        "Failed to parse '10 - 5': {:?}",
        result.err()
    );

    if let Expression::Sub(left, _, right) = result.unwrap() {
        assert_eq!(*left, Expression::Number(10));
        assert_eq!(*right, Expression::Number(5));
    } else {
        panic!("Expected subtraction expression");
    }

    // Test chained subtraction (left associative)
    let result = parse("20 - 10 - 5");
    assert!(
        result.is_ok(),
        "Failed to parse '20 - 10 - 5': {:?}",
        result.err()
    );

    // Should parse as (20 - 10) - 5 due to left associativity
    if let Expression::Sub(left, _, right) = result.unwrap() {
        assert_eq!(*right, Expression::Number(5));
        if let Expression::Sub(inner_left, _, inner_right) = *left {
            assert_eq!(*inner_left, Expression::Number(20));
            assert_eq!(*inner_right, Expression::Number(10));
        } else {
            panic!("Expected left side to be subtraction");
        }
    } else {
        panic!("Expected top-level subtraction");
    }
}

#[test]
fn test_arithmetic_multiplication() {
    use rust_sitter_example::arithmetic::grammar::{Expression, parse};

    // Test simple multiplication
    let result = parse("3 * 4");
    assert!(
        result.is_ok(),
        "Failed to parse '3 * 4': {:?}",
        result.err()
    );

    if let Expression::Mul(left, _, right) = result.unwrap() {
        assert_eq!(*left, Expression::Number(3));
        assert_eq!(*right, Expression::Number(4));
    } else {
        panic!("Expected multiplication expression");
    }
}

#[test]
fn test_arithmetic_precedence() {
    use rust_sitter_example::arithmetic::grammar::{Expression, parse};

    // Test precedence: "1 - 2 * 3" should parse as "1 - (2 * 3)"
    // Multiplication has higher precedence than subtraction
    let result = parse("1 - 2 * 3");
    assert!(
        result.is_ok(),
        "Failed to parse '1 - 2 * 3': {:?}",
        result.err()
    );

    if let Expression::Sub(left, _, right) = result.unwrap() {
        assert_eq!(*left, Expression::Number(1));

        // Right side should be multiplication due to precedence
        if let Expression::Mul(mul_left, _, mul_right) = *right {
            assert_eq!(*mul_left, Expression::Number(2));
            assert_eq!(*mul_right, Expression::Number(3));
        } else {
            panic!(
                "Expected multiplication on right side of subtraction, got: {:?}",
                right
            );
        }
    } else {
        panic!("Expected subtraction at top level");
    }

    // Test reverse: "2 * 3 - 1" should parse as "(2 * 3) - 1"
    let result = parse("2 * 3 - 1");
    assert!(
        result.is_ok(),
        "Failed to parse '2 * 3 - 1': {:?}",
        result.err()
    );

    if let Expression::Sub(left, _, right) = result.unwrap() {
        assert_eq!(*right, Expression::Number(1));

        // Left side should be multiplication
        if let Expression::Mul(mul_left, _, mul_right) = *left {
            assert_eq!(*mul_left, Expression::Number(2));
            assert_eq!(*mul_right, Expression::Number(3));
        } else {
            panic!("Expected multiplication on left side of subtraction");
        }
    } else {
        panic!("Expected subtraction at top level");
    }
}

#[test]
fn test_arithmetic_whitespace() {
    use rust_sitter_example::arithmetic::grammar::{Expression, parse};

    // Test with no whitespace
    let result = parse("1-2");
    assert!(result.is_ok(), "Failed to parse '1-2'");

    // Test with extra whitespace
    let result = parse("  1   -   2  ");
    assert!(result.is_ok(), "Failed to parse with extra whitespace");

    // Test with newlines
    let result = parse("1\n-\n2");
    assert!(result.is_ok(), "Failed to parse with newlines");
}

#[test]
fn test_arithmetic_error_handling() {
    use rust_sitter_example::arithmetic::grammar::parse;

    // Test invalid operators
    let result = parse("++");
    assert!(result.is_err(), "Expected error for '++'");

    // Test incomplete expression
    let result = parse("1 -");
    assert!(result.is_err(), "Expected error for '1 -'");

    // Note: Parser may accept partial input, so "1 @ 2" might parse as just "1"
    // and "- 5" as just "5". These are valid behaviors for error recovery.
}

// Benchmarks for performance testing
#[cfg(test)]
mod bench {
    use std::time::Instant;

    #[test]
    #[ignore] // Run with cargo test -- --ignored
    fn bench_deep_subtraction_tree() {
        use rust_sitter_example::arithmetic::grammar::parse;

        // Generate large expression: "1 - 2 - 3 - ... - 100"
        // This creates a deep left-associative tree
        let mut input = String::from("1");
        for i in 2..=100 {
            input.push_str(&format!(" - {}", i));
        }

        let start = Instant::now();
        let result = parse(&input);
        let duration = start.elapsed();

        assert!(result.is_ok(), "Failed to parse large expression");
        println!("Parsed 100-term subtraction expression in {:?}", duration);

        // Should complete in reasonable time (< 500ms)
        assert!(
            duration.as_millis() < 500,
            "Parsing took too long: {:?}",
            duration
        );
    }

    #[test]
    #[ignore] // Run with cargo test -- --ignored
    fn bench_complex_precedence() {
        use rust_sitter_example::arithmetic::grammar::parse;

        // Generate expression with many precedence levels
        // Pattern: "1 - 2 * 3 - 4 * 5 - 6 * 7 ..."
        let mut input = String::new();
        for i in 0..50 {
            if i > 0 {
                input.push_str(" - ");
            }
            let left = i * 2 + 1;
            let right = i * 2 + 2;
            input.push_str(&format!("{} * {}", left, right));
        }

        let start = Instant::now();
        let result = parse(&input);
        let duration = start.elapsed();

        assert!(
            result.is_ok(),
            "Failed to parse complex precedence expression"
        );
        println!("Parsed complex precedence expression in {:?}", duration);

        // Should complete in reasonable time (< 1 second)
        assert!(
            duration.as_millis() < 1000,
            "Parsing took too long: {:?}",
            duration
        );
    }
}
