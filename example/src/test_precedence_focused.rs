use crate::arithmetic::grammar::Expression;

pub fn test_precedence_focused() {
    println!("=== FOCUSED PRECEDENCE TEST ===");
    
    // Test case: "1 - 2 * 3"
    // Should parse as: Sub(1, Mul(2, 3)) if precedence is correct
    // Would parse as: Mul(Sub(1, 2), 3) if precedence is wrong
    
    let input = "1 - 2 * 3";
    println!("Parsing: '{}'", input);
    
    match crate::arithmetic::grammar::parse(input) {
        Ok(expr) => {
            println!("Result: {:?}", expr);
            
            // Check the structure
            match &expr {
                Expression::Sub(left, _op, right) => {
                    println!("Top-level: Subtraction");
                    println!("  Left: {:?}", left);
                    println!("  Right: {:?}", right);
                    
                    // Check if right side is multiplication
                    match right.as_ref() {
                        Expression::Mul(ml, _mop, mr) => {
                            println!("  ✓ RIGHT SIDE IS MULTIPLICATION");
                            println!("    Mul left: {:?}", ml);
                            println!("    Mul right: {:?}", mr);
                            println!("\n✓ PRECEDENCE IS CORRECT: multiplication binds tighter than subtraction");
                        }
                        _ => {
                            println!("  ✗ RIGHT SIDE IS NOT MULTIPLICATION");
                            println!("\n✗ PRECEDENCE ERROR: Expected Mul(2, 3) on right side");
                        }
                    }
                }
                Expression::Mul(left, _op, right) => {
                    println!("Top-level: Multiplication");
                    println!("  Left: {:?}", left);
                    println!("  Right: {:?}", right);
                    println!("\n✗ PRECEDENCE ERROR: Top level should be subtraction, not multiplication");
                    
                    // Check if it's parsing as Mul(Sub(1, 2), 3)
                    if let Expression::Sub(_, _, _) = left.as_ref() {
                        println!("  (Parsing as Mul(Sub(1, 2), 3) - precedence is reversed!)");
                    }
                }
                _ => {
                    println!("Unexpected expression type: {:?}", expr);
                }
            }
        }
        Err(e) => {
            println!("Parse error: {:?}", e);
        }
    }
    
    println!("\n=== Additional test cases ===");
    
    // Test more complex expressions
    let test_cases = vec![
        ("1 * 2 - 3", "Should be Sub(Mul(1, 2), 3)"),
        ("1 - 2 - 3", "Should be Sub(Sub(1, 2), 3) - left associative"),
        ("1 * 2 * 3", "Should be Mul(Mul(1, 2), 3) - left associative"),
    ];
    
    for (input, expected) in test_cases {
        println!("\nTest: '{}' ({})", input, expected);
        match crate::arithmetic::grammar::parse(input) {
            Ok(expr) => println!("  Parsed: {:?}", expr),
            Err(e) => println!("  Error: {:?}", e),
        }
    }
}