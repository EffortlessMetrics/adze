use adze_tool::grammar_js::parse_grammar_js_v2;

#[test]
fn test_minimal_javascript() {
    let grammar = r#"
module.exports = grammar({
  name: 'javascript',

  extras: $ => [
    /\s/,
    $.comment
  ],

  inline: $ => [
    $.statement,
    $.expression
  ],

  precedences: $ => [
    [6, 5, 4, 3],
    [1, 0]
  ],

  conflicts: $ => [
    [$.primary_expression, $.pattern],
    [$.primary_expression, $.rest_pattern]
  ],

  word: $ => $.identifier,

  rules: {
    program: $ => repeat($.statement),
    
    statement: $ => 'stmt',
    expression: $ => 'expr',
    
    primary_expression: $ => 'primary',
    pattern: $ => 'pattern',
    rest_pattern: $ => 'rest',
    identifier: $ => 'id',
    comment: $ => 'comment'
  }
});
"#;

    let result = parse_grammar_js_v2(grammar);
    match result {
        Ok(parsed) => {
            println!("Successfully parsed minimal JavaScript grammar!");
            println!("Name: {}", parsed.name);
            println!("Inline rules: {:?}", parsed.inline);
            println!("Conflicts: {:?}", parsed.conflicts);
        }
        Err(e) => {
            panic!("Failed to parse minimal grammar: {}", e);
        }
    }
}
