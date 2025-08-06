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
    // For now, we'll just test that we can create a parser
    // Full parsing will require runtime integration
    
    // Register the scanner
    rust_sitter_python::register_scanner();
    
    // Create a simple Python source
    let source = b"def hello():\n    pass\n";
    
    // This test validates that the scanner registration works
    // Actual parsing would require creating a Parser with Grammar and ParseTable
    // which we'd need to extract from the LANGUAGE struct
    
    println!("Test source: {:?}", std::str::from_utf8(source).unwrap());
    println!("Scanner registered successfully");
    
    // TODO: Once parser runtime integration is complete, add actual parsing test here
    // let mut parser = rust_sitter::parser_v4::Parser::new(...);
    // parser.set_language(&rust_sitter_python::grammar_python::LANGUAGE);
    // let tree = parser.parse(std::str::from_utf8(source).unwrap()).unwrap();
    // assert_eq!(tree.root_node().kind(), "module");
}