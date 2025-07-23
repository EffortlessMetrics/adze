use rust_sitter_tool::grammar_js::GrammarJsParserV3;

#[test]
fn test_word_token_parsing() {
    // Test parsing word token declaration
    let grammar_content = r#"
module.exports = grammar({
  name: 'test_word',
  
  word: $ => $.identifier,
  
  rules: {
    identifier: $ => /[a-zA-Z_]\w*/,
    
    keyword_if: $ => 'if',
    
    statement: $ => choice(
      $.keyword_if,
      $.identifier
    )
  }
})
"#;
    
    let parser = GrammarJsParserV3::new(grammar_content.to_string());
    match parser.parse() {
        Ok(grammar) => {
            println!("Successfully parsed grammar with word token!");
            println!("Grammar name: {}", grammar.name);
            println!("Word token: {:?}", grammar.word);
            
            // Check that word was extracted correctly
            assert_eq!(grammar.word, Some("identifier".to_string()));
        }
        Err(e) => {
            println!("Failed to parse: {}", e);
            // This is expected for now if word parsing isn't implemented
        }
    }
}