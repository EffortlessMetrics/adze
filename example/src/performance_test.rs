// Performance comparison test for rust-sitter

#[cfg(test)]
mod tests {
    
    use crate::arithmetic;
    use std::time::Instant;

    #[test]
    fn measure_parsing_performance() {
        let test_cases = vec![
            ("simple", "1 - 2"),
            ("medium", "1 - 2 * 3 - 4 * 5"),
            ("complex", "1 - 2 * 3 * 5 - 6 * 7 * 9 - 10"),
            (
                "deeply_nested",
                "1 - 2 * 3 - 4 * 5 - 6 * 7 - 8 * 9 - 10 * 11",
            ),
        ];

        println!("\nPerformance Test Results:");
        println!("=========================");

        for (name, input) in test_cases {
            // Warm up
            for _ in 0..10 {
                let _ = arithmetic::grammar::parse(input);
            }

            // Measure
            let iterations = 1000;
            let start = Instant::now();

            for _ in 0..iterations {
                let result = arithmetic::grammar::parse(input);
                assert!(result.is_ok());
            }

            let elapsed = start.elapsed();
            let avg_time = elapsed.as_nanos() / iterations;

            println!(
                "{}: {} ns/parse (total: {:?} for {} iterations)",
                name, avg_time, elapsed, iterations
            );
        }

        // Test with a large expression
        let mut large_expr = String::new();
        for i in 0..50 {
            if i > 0 {
                large_expr.push_str(" - ");
            }
            large_expr.push_str(&i.to_string());
            if i % 5 == 0 && i > 0 {
                large_expr.push_str(" * 2");
            }
        }

        let start = Instant::now();
        let iterations = 100;

        for _ in 0..iterations {
            let result = arithmetic::grammar::parse(&large_expr);
            assert!(result.is_ok());
        }

        let elapsed = start.elapsed();
        let avg_time = elapsed.as_micros() / iterations;

        println!(
            "large (50 terms): {} µs/parse (total: {:?} for {} iterations)",
            avg_time, elapsed, iterations
        );
    }

    #[test]
    fn measure_memory_usage() {
        // Simple memory usage test
        let input = "1 - 2 * 3 - 4 * 5 - 6 * 7 - 8 * 9 - 10";

        // Parse multiple times to see if there's memory growth
        let mut results = Vec::new();
        for _ in 0..100 {
            let result = arithmetic::grammar::parse(input).unwrap();
            results.push(result);
        }

        // Keep results alive to prevent optimization
        assert_eq!(results.len(), 100);
        println!("Parsed {} times without issues", results.len());
    }
}
