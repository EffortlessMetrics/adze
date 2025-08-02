use crate::arithmetic;
use crate::arithmetic::grammar::Expression;

pub fn test_precedence() {
    println!("Testing operator precedence:");

    let test_cases = vec![
        ("1 - 2 * 3", "Sub(1, Mul(2, 3))"),
        ("1 * 2 - 3", "Sub(Mul(1, 2), 3)"),
        ("1 + 2 * 3", "Add(1, Mul(2, 3))"),
        ("1 * 2 + 3", "Add(Mul(1, 2), 3)"),
        ("1 - 2 - 3", "Sub(Sub(1, 2), 3)"), // left associative
        ("1 * 2 * 3", "Mul(Mul(1, 2), 3)"), // left associative
    ];

    // Check if using pure-Rust parser
    if std::env::var("RUST_SITTER_USE_PURE_PARSER").is_ok() {
        println!("  Using pure-Rust parser");
    } else {
        println!("  Using C parser");
    }

    for (input, expected_desc) in test_cases {
        println!("\n  Test: '{}'", input);
        println!("  Expected: {}", expected_desc);

        match arithmetic::grammar::parse(input) {
            Ok(expr) => {
                println!("  Parsed: {:?}", expr);

                // Check specific cases
                match (input, &expr) {
                    ("1 - 2 * 3", Expression::Sub(_l, _, r)) => match r.as_ref() {
                        Expression::Mul(_, _, _) => {
                            println!(
                                "  ✓ Precedence correct: multiplication has higher precedence"
                            );
                        }
                        _ => {
                            println!("  ✗ PRECEDENCE ERROR: Expected Mul on right side of Sub");
                        }
                    },
                    ("1 - 2 * 3", Expression::Mul(l, _, _)) => {
                        println!("  ✗ PRECEDENCE ERROR: Top-level should be Sub, not Mul");
                        if let Expression::Sub(_, _, _) = l.as_ref() {
                            println!("    (Currently parsing as Mul(Sub(1, 2), 3))");
                        }
                    }
                    _ => {}
                }
            }
            Err(errs) => {
                println!("  Parse errors: {:?}", errs);
            }
        }
    }
}
