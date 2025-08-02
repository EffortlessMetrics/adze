use crate::arithmetic;

pub fn test_whitespace_parsing() {
    println!("Testing whitespace parsing:");

    // Test 1: Simple number
    println!("\nTest 1: Parsing '42'");
    match arithmetic::grammar::parse("42") {
        Ok(expr) => {
            println!("  Success! Parsed: {:?}", expr);
        }
        Err(errs) => {
            println!("  Failed with errors: {:?}", errs);
        }
    }

    // Test 2: Number with leading whitespace
    println!("\nTest 2: Parsing '  42'");
    match arithmetic::grammar::parse("  42") {
        Ok(expr) => {
            println!("  Success! Parsed: {:?}", expr);
        }
        Err(errs) => {
            println!("  Failed with errors: {:?}", errs);
        }
    }

    // Test 3: Number with trailing whitespace
    println!("\nTest 3: Parsing '42  '");
    match arithmetic::grammar::parse("42  ") {
        Ok(expr) => {
            println!("  Success! Parsed: {:?}", expr);
        }
        Err(errs) => {
            println!("  Failed with errors: {:?}", errs);
        }
    }

    // Test 4: Expression with whitespace
    println!("\nTest 4: Parsing '1 + 2'");
    match arithmetic::grammar::parse("1 + 2") {
        Ok(expr) => {
            println!("  Success! Parsed: {:?}", expr);
        }
        Err(errs) => {
            println!("  Failed with errors: {:?}", errs);
        }
    }

    // Test 5: Complex expression with whitespace
    println!("\nTest 5: Parsing '  10  -  5  *  2  '");
    match arithmetic::grammar::parse("  10  -  5  *  2  ") {
        Ok(expr) => {
            println!("  Success! Parsed: {:?}", expr);
        }
        Err(errs) => {
            println!("  Failed with errors: {:?}", errs);
        }
    }
}
