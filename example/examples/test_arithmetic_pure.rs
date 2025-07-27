// Test the pure-Rust arithmetic parser

// Include the arithmetic module from main.rs  
include!("../src/arithmetic.rs");

fn main() {
    #[cfg(feature = "pure-rust")]
    {
        println!("Testing pure-Rust arithmetic parser...");
        
        // Import the generated arithmetic grammar
        use arithmetic::grammar;
        
        // Test parsing simple expressions
        let test_cases = vec![
            "1",
        ];
        
        for input in test_cases {
            println!("\nParsing: '{}'", input);
            println!("Starting parse...");
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