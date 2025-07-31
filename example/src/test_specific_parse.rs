use crate::arithmetic;

pub fn test_specific_parse() {
    println!("=== SPECIFIC PARSE TEST FOR '1 - 2 * 3' ===");
    
    // Enable more detailed logging
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }
    
    let input = "1 - 2 * 3";
    println!("Input: '{}'", input);
    println!("Input bytes: {:?}", input.as_bytes());
    println!("Expected: Sub(Number(1), Mul(Number(2), Number(3)))");
    println!();
    
    match arithmetic::grammar::parse(input) {
        Ok(expr) => {
            println!("ACTUAL RESULT: {:?}", expr);
            
            // Analyze what we got
            match expr {
                arithmetic::grammar::Expression::Sub(left, _op, right) => {
                    println!("\nParsed as subtraction:");
                    println!("  Left operand: {:?}", left);
                    println!("  Right operand: {:?}", right);
                    
                    // Check if this is the expected structure
                    match (left.as_ref(), right.as_ref()) {
                        (arithmetic::grammar::Expression::Number(1), arithmetic::grammar::Expression::Number(3)) => {
                            println!("\n✗ ERROR: Parser skipped '2 *' entirely!");
                            println!("  The parser is treating '1 - 2 * 3' as '1 - 3'");
                        }
                        (arithmetic::grammar::Expression::Number(1), arithmetic::grammar::Expression::Mul(_, _, _)) => {
                            println!("\n✓ SUCCESS: Correct precedence!");
                            println!("  Multiplication binds tighter than subtraction");
                        }
                        _ => {
                            println!("\n? Unexpected structure");
                        }
                    }
                }
                _ => {
                    println!("\n✗ ERROR: Not parsed as subtraction at top level!");
                }
            }
        }
        Err(errors) => {
            println!("PARSE ERROR: {:?}", errors);
        }
    }
}