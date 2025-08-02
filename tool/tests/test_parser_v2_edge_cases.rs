use rust_sitter_tool::grammar_js::parse_grammar_js_v2;

#[test]
fn test_simple_inline_parsing() {
    let grammar = r#"
module.exports = grammar({
  name: 'test',
  inline: $ => [
    $.statement,
    $.expression
  ],
  rules: {
    statement: $ => 'stmt',
    expression: $ => 'expr'
  }
});
"#;

    let result = parse_grammar_js_v2(grammar);
    assert!(
        result.is_ok(),
        "Failed to parse simple grammar: {:?}",
        result.err()
    );

    let parsed = result.unwrap();
    assert_eq!(parsed.name, "test");
    assert_eq!(parsed.inline.len(), 2);
    assert!(parsed.inline.contains(&"statement".to_string()));
    assert!(parsed.inline.contains(&"expression".to_string()));
}

#[test]
fn test_regex_in_grammar() {
    let grammar = r#"
module.exports = grammar({
  name: 'test',
  rules: {
    whitespace: $ => /\s+/,
    identifier: $ => /[a-zA-Z_]\w*/,
    string: $ => /"[^"]*"/
  }
});
"#;

    let result = parse_grammar_js_v2(grammar);
    assert!(
        result.is_ok(),
        "Failed to parse grammar with regex: {:?}",
        result.err()
    );
}

#[test]
fn test_string_with_quotes() {
    let grammar = r#"
module.exports = grammar({
  name: 'test',
  rules: {
    single_quote: $ => "'",
    double_quote: $ => '"',
    backtick: $ => '`',
    mixed: $ => seq("'", $.content, "'")
  }
});
"#;

    let result = parse_grammar_js_v2(grammar);
    assert!(
        result.is_ok(),
        "Failed to parse grammar with quotes: {:?}",
        result.err()
    );
}
