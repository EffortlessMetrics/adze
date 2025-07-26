// Test the pure-Rust arithmetic parser

fn main() {
    #[cfg(feature = "pure-rust")]
    {
        println!("Testing pure-Rust arithmetic parser...");
        
        // Import the generated arithmetic grammar
        // Note: In examples, we use the crate relative path
        use crate::arithmetic::grammar;
        
        // Test parsing simple expressions
        let test_cases = vec![
            "1",
            "42",
            "1 + 2", 
            "1 - 2",
            "2 * 3",
            "1 + 2 * 3",
            "1 * 2 + 3",
        ];
        
        for input in test_cases {
            println!("\nParsing: '{}'", input);
            match grammar::parse(input) {
                Ok(expr) => println!("  Success: {:?}", expr),
                Err(errors) => {
                    println!("  Failed with {} errors:", errors.len());
                    for error in errors {
                        println!("    - {:?}", error);
                    }
                }
            }
        }
    }
    
    #[cfg(not(feature = "pure-rust"))]
    {
        println!("This example requires the 'pure-rust' feature");
    }
}