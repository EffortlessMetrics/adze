//! Demo showing GLR runtime integration
//!
//! This example shows how the runtime would be used once the GLR engine
//! is fully integrated with grammar generation.

#[cfg(feature = "glr-core")]
use rust_sitter_runtime::Parser;

#[cfg(feature = "glr-core")]
fn main() {
    println!("GLR Runtime Demo");
    println!("================");

    // Create a parser
    let mut parser = Parser::new();

    // In a real scenario, you would:
    // 1. Load a language from a generated crate:
    //    let language = my_language::language();
    // 2. Set it on the parser:
    //    parser.set_language(language).unwrap();
    // 3. Parse some input:
    //    let tree = parser.parse("def foo(): pass", None).unwrap();

    // For now, we can show the error message that indicates the integration point:
    match parser.parse(b"def foo(): pass", None) {
        Ok(_tree) => println!("Parse successful!"),
        Err(e) => println!("Expected error (integration pending): {}", e),
    }

    println!("\nIntegration Status:");
    println!("- Runtime API: ✅ Complete");
    println!("- GLR Engine: ✅ Available");
    println!("- Grammar Link: ⏳ Pending (needs generated Language to include Grammar)");
}

#[cfg(not(feature = "glr-core"))]
fn main() {
    println!("This demo requires the 'glr-core' feature.");
    println!("Run with: cargo run --features glr-core --example glr_demo");
}
