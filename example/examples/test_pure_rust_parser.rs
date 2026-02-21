// Test the pure-Rust parser runtime
#[cfg(feature = "pure-rust")]
fn main() {
    println!("Testing pure-Rust arithmetic parser...");

    // Get the language from generated code
    let language = unsafe { &LANGUAGE };

    println!(
        "Language: symbol_count={}, state_count={}",
        language.symbol_count, language.state_count
    );

    // Test parsing "42"
    let input = "42";
    println!("\nParsing: '{}'", input);

    // Create parser
    let mut parser = adze::pure_parser::Parser::new();
    parser.set_language(language);

    let result = parser.parse_string(input);

    if let Some(root) = result.root {
        println!("Success! Parsed tree with root: {:?}", root);
    } else if !result.errors.is_empty() {
        println!("Failed to parse with errors:");
        for error in &result.errors {
            println!("  - {:?}", error);
        }
    } else {
        println!("Failed to parse: no root node and no errors");
    }
}

#[cfg(not(feature = "pure-rust"))]
fn main() {
    println!("This example requires the 'pure-rust' feature");
}

// Include generated parser
#[cfg(feature = "pure-rust")]
include!(concat!(
    env!("OUT_DIR"),
    "/grammar_arithmetic/parser_arithmetic.rs"
));
