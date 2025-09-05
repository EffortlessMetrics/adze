//! Simple example showing the Tree-sitter-compatible API

use rust_sitter_runtime::{test_helpers::stub_language, Language, Parser};

fn main() {
    // Create a parser
    let mut parser = Parser::new();

    // In a real scenario, you'd load a language from a generated crate
    // For now, we build a stub language
    let language = stub_language();

    // Set the language
    parser
        .set_language(language)
        .expect("Failed to set language");

    // Parse some input (will return a stub tree for now)
    let input = "def hello():\n    print('Hello, world!')";
    match parser.parse_utf8(input, None) {
        Ok(tree) => {
            println!("Parse successful!");
            println!("Root node: {:?}", tree.root_node());

            // In a real implementation, you could walk the tree
            let root = tree.root_node();
            println!("Node kind: {}", root.kind());
            println!("Byte range: {:?}", root.byte_range());
            println!("Child count: {}", root.child_count());
        }
        Err(e) => {
            eprintln!("Parse failed: {}", e);
        }
    }

    // Example of incremental parsing (when implemented)
    println!("\nIncremental parsing example:");
    let edited_input = "def hello():\n    print('Hello, Rust!')";
    match parser.parse_utf8(edited_input, None) {
        // Would pass old_tree in real usage
        Ok(_tree) => println!("Incremental parse successful!"),
        Err(e) => eprintln!("Incremental parse failed: {}", e),
    }
}
