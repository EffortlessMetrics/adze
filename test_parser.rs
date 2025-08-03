fn main() {
    println!("Testing parser with '42'");
    
    // Load the generated parser module
    include!("target/debug/build/rust-sitter-python-simple-5942df5a7bccb953/out/grammar_python_simple/parser_python_simple.rs");
    
    let language = &LANGUAGE;
    let source = b"42";
    
    // Create a parser
    let mut parser = rust_sitter::pure_parser::Parser::new();
    
    // Parse the source
    let result = parser.parse(language, source, None);
    
    println!("Parse result: {:?}", result);
}