use std::path::Path;

fn main() {
    // Use pure Rust parser generation
    // SAFETY: This is safe in a build script as it runs in a single-threaded context
    unsafe {
        std::env::set_var("RUST_SITTER_USE_PURE_RUST", "1");
    }
    
    // Enable debug output
    // SAFETY: This is safe in a build script as it runs in a single-threaded context
    unsafe {
        std::env::set_var("RUST_SITTER_EMIT_ARTIFACTS", "true");
    }
    
    // Generate grammars first to see what's being generated
    let grammars = rust_sitter_tool::generate_grammars(Path::new("src/lib.rs"));
    
    // Print the generated grammar for debugging
    for grammar in &grammars {
        eprintln!("Generated rust-sitter grammar JSON:");
        eprintln!("{}", serde_json::to_string_pretty(&grammar).unwrap());
        
        // Also write to a file for easier debugging
        use std::io::Write;
        let mut file = std::fs::File::create("/tmp/test-vec-wrapper-grammar.json").unwrap();
        writeln!(file, "{}", serde_json::to_string_pretty(&grammar).unwrap()).unwrap();
    }
    
    // Now build the parsers
    rust_sitter_tool::build_parsers(Path::new("src/lib.rs"));
}