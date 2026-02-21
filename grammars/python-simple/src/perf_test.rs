use adze::unified_parser::Parser;
use std::time::Instant;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::LANGUAGE;

    #[test]
    fn actual_parsing_performance() {
        let test_cases = vec![
            ("simple", "x = 42"),
            ("medium", "x = 42\ny = x + 1\nz = x * y"),
            (
                "complex",
                "x = 42\ny = x + 1\nz = x * y + 10\nresult = z + x * 2\nfinal = result * x + y",
            ),
        ];

        println!("\n=== ACTUAL PARSING PERFORMANCE ===");
        println!("Code           | Time      | Chars/sec | Lines/sec");
        println!("---------------|-----------|-----------|----------");

        for (name, code) in test_cases {
            let mut parser = Parser::new();
            parser.set_language(&LANGUAGE).unwrap();

            // Warmup
            for _ in 0..100 {
                let _ = parser.parse_with_error(code);
            }

            // Measure
            let iterations = 10000;
            let start = Instant::now();

            for _ in 0..iterations {
                let _tree = parser.parse_with_error(code).unwrap();
            }

            let elapsed = start.elapsed();
            let avg_time = elapsed.as_nanos() / iterations as u128;
            let chars_per_sec = (code.len() as f64) / (avg_time as f64 / 1_000_000_000.0);
            let lines_per_sec = (code.lines().count() as f64) / (avg_time as f64 / 1_000_000_000.0);

            println!(
                "{:14} | {:7}ns | {:9.0} | {:9.0}",
                name, avg_time, chars_per_sec, lines_per_sec
            );
        }

        // Large file test
        let mut large_code = String::new();
        for i in 0..1000 {
            large_code.push_str(&format!("var{} = {} * 2 + {}\n", i, i, i + 1));
        }

        let mut parser = Parser::new();
        parser.set_language(&LANGUAGE).unwrap();

        let start = Instant::now();
        let iterations = 100;

        for _ in 0..iterations {
            let _tree = parser.parse_with_error(&large_code).unwrap();
        }

        let elapsed = start.elapsed();
        let avg_time = elapsed.as_micros() / iterations as u128;
        let mb_per_sec = (large_code.len() as f64 / 1_000_000.0) / (avg_time as f64 / 1_000_000.0);

        println!("\nLarge file (1000 lines):");
        println!(
            "Time: {}µs | Size: {} chars | Throughput: {:.1} MB/sec",
            avg_time,
            large_code.len(),
            mb_per_sec
        );
    }
}
