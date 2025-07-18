// Test file to reproduce RUST_SITTER_EMIT_ARTIFACTS issue
use std::env;
use std::path::Path;

fn main() {
    // Set the environment variable
    env::set_var("RUST_SITTER_EMIT_ARTIFACTS", "true");
    env::set_var("OUT_DIR", "./test_output");
    
    // Create a simple test grammar file
    let test_grammar = r#"
#[rust_sitter::grammar("test")]
mod grammar {
    #[rust_sitter::language]
    pub enum Expression {
        Number(
            #[rust_sitter::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
            i32
        ),
    }
}
"#;
    
    // Write test grammar to a file
    std::fs::write("test_grammar.rs", test_grammar).unwrap();
    
    // Try to build parsers
    println!("Testing RUST_SITTER_EMIT_ARTIFACTS...");
    
    // This should reproduce the issue
    rust_sitter_tool::build_parsers(Path::new("test_grammar.rs"));
    
    println!("Test completed successfully!");
}