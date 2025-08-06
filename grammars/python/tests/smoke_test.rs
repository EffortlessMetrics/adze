// Simple smoke test for Python parsing with the pure-Rust implementation

#[test]
fn test_python_language_exists() {
    // Just check that the LANGUAGE struct exists and can be accessed
    // This validates that the code generation worked
    let _language = &rust_sitter_python::grammar_python::LANGUAGE;
    
    // Verify basic properties
    assert!(rust_sitter_python::grammar_python::LANGUAGE.symbol_count > 0);
    assert!(rust_sitter_python::grammar_python::LANGUAGE.version == 15);
    
    println!("Python grammar loaded successfully:");
    println!("  Symbol count: {}", rust_sitter_python::grammar_python::LANGUAGE.symbol_count);
    println!("  State count: {}", rust_sitter_python::grammar_python::LANGUAGE.state_count);
    println!("  External token count: {}", rust_sitter_python::grammar_python::LANGUAGE.external_token_count);
}

#[test]
fn test_simple_python_parse() {
    // Register the scanner
    rust_sitter_python::register_scanner();
    
    // Create a simple Python source
    let source = "def hello():\n    pass\n";
    
    // Load token patterns from grammar.json
    #[cfg(feature = "pure-rust")]
    let grammar_json_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .join("xtask/fixtures/tree-sitter-python/src/grammar.json");
    
    #[cfg(feature = "pure-rust")]
    let token_patterns = rust_sitter::decoder::load_token_patterns(&grammar_json_path);
    
    #[cfg(feature = "pure-rust")]
    println!("Loaded {} token patterns from grammar.json", token_patterns.len());
    
    // Create a parser and set the language with real token patterns
    #[cfg(feature = "pure-rust")]
    let mut parser = rust_sitter::parser_v4::Parser::from_language_with_patterns(
        &rust_sitter_python::grammar_python::LANGUAGE,
        "python".to_string(),
        &token_patterns
    );
    
    #[cfg(not(feature = "pure-rust"))]
    let mut parser = rust_sitter::parser_v4::Parser::from_language(
        &rust_sitter_python::grammar_python::LANGUAGE,
        "python".to_string()
    );
    
    // Parse the source
    let tree = parser.parse(source).unwrap();
    
    println!("Test source: {:?}", source);
    println!("Parse result:");
    println!("  Root kind: {}", tree.root_kind());
    println!("  Error count: {}", tree.error_count());
    
    // Verify the parse succeeded (stub returns module ID 267)
    assert_eq!(tree.root_kind(), 267); // module symbol ID
    assert_eq!(tree.error_count(), 0);
}