use crate::arithmetic;

pub fn test_manual_parse() {
    println!("=== MANUAL PARSE TEST ===");
    
    // Manually invoke parse on specific input
    let input = "1 - 2 * 3";
    println!("Parsing input: '{}'", input);
    
    match arithmetic::grammar::parse(input) {
        Ok(expr) => {
            println!("Parse result: {:?}", expr);
            
            // Print the structure with detailed formatting
            match &expr {
                arithmetic::grammar::Expression::Sub(left, op, right) => {
                    println!("Structure: Sub(");
                    println!("  left: {:?},", left);
                    println!("  op: {:?},", op);
                    println!("  right: {:?}", right);
                    println!(")");
                    
                    // Check if right is multiplication
                    if let arithmetic::grammar::Expression::Mul(_, _, _) = right.as_ref() {
                        println!("\n✓ PRECEDENCE CORRECT: Multiplication on right side of subtraction");
                    } else {
                        println!("\n✗ PRECEDENCE ERROR: Right side is not multiplication!");
                    }
                }
                arithmetic::grammar::Expression::Mul(left, op, right) => {
                    println!("Structure: Mul(");
                    println!("  left: {:?},", left);
                    println!("  op: {:?},", op);
                    println!("  right: {:?}", right);
                    println!(")");
                    println!("\n✗ PRECEDENCE ERROR: Top level should be subtraction!");
                }
                arithmetic::grammar::Expression::Number(n) => {
                    println!("Structure: Number({})", n);
                    println!("\n✗ ERROR: Expected compound expression, got single number!");
                }
            }
        }
        Err(e) => {
            println!("Parse error: {:?}", e);
        }
    }
    
    // Also test other expressions
    println!("\n--- Testing other expressions ---");
    
    let test_cases = vec![
        "1 * 2 - 3",
        "1 - 2 - 3", 
        "1 * 2 * 3",
        "1",
        "1 - 2",
        "2 * 3",
    ];
    
    for input in test_cases {
        match arithmetic::grammar::parse(input) {
            Ok(expr) => println!("{:?} => {:?}", input, expr),
            Err(_) => println!("{:?} => Parse error", input),
        }
    }
}