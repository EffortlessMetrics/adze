#[cfg(test)]
mod tests {
    use super::super::parser_v2::parse_grammar_js_v2;
    
    #[test]
    fn test_basic_grammar() {
        let grammar_js = r#"
module.exports = grammar({
  name: 'test',
  
  rules: {
    source_file: $ => repeat($.statement),
    
    statement: $ => choice(
      $.expression,
      $.declaration
    ),
    
    expression: $ => seq(
      $.identifier,
      ';'
    ),
    
    declaration: $ => seq(
      'let',
      $.identifier,
      ';'
    ),
    
    identifier: $ => /[a-zA-Z_]\w*/
  }
});
        "#;
        
        let result = parse_grammar_js_v2(grammar_js);
        assert!(result.is_ok());
        
        let grammar = result.unwrap();
        assert_eq!(grammar.name, "test");
        assert_eq!(grammar.rules.len(), 5);
        assert!(grammar.rules.contains_key("source_file"));
        assert!(grammar.rules.contains_key("identifier"));
    }
}