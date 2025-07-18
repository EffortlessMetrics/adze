// Test binary to reproduce RUST_SITTER_EMIT_ARTIFACTS issue
use std::env;
use std::path::Path;

fn main() {
    // Set the environment variables that would normally be set by Cargo
    env::set_var("RUST_SITTER_EMIT_ARTIFACTS", "true");
    env::set_var("TARGET", "x86_64-pc-windows-msvc"); // Set appropriate target
    
    // Create test output directory
    let test_dir = "./test_output";
    std::fs::create_dir_all(test_dir).unwrap();
    env::set_var("OUT_DIR", test_dir);
    
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
    let grammar_file = "test_grammar.rs";
    std::fs::write(grammar_file, test_grammar).unwrap();
    
    println!("Testing RUST_SITTER_EMIT_ARTIFACTS with file: {}", grammar_file);
    println!("OUT_DIR: {}", env::var("OUT_DIR").unwrap());
    println!("RUST_SITTER_EMIT_ARTIFACTS: {}", env::var("RUST_SITTER_EMIT_ARTIFACTS").unwrap());
    
    // Try to build parsers - this should reproduce the issue
    match std::panic::catch_unwind(|| {
        rust_sitter_tool::build_parsers(Path::new(grammar_file));
    }) {
        Ok(_) => println!("✅ Test completed successfully!"),
        Err(e) => {
            println!("❌ Error occurred: {:?}", e);
            // Try to get more details about the error
            if let Some(s) = e.downcast_ref::<String>() {
                println!("Error string: {}", s);
            } else if let Some(s) = e.downcast_ref::<&str>() {
                println!("Error str: {}", s);
            }
        }
    }
    
    // Clean up
    let _ = std::fs::remove_file(grammar_file);
    let _ = std::fs::remove_dir_all(test_dir);
}