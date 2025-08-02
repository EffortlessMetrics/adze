use rust_sitter_tool::grammar_js::parse_grammar_js_v2;

#[test]
fn test_inline_array_parsing() {
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

    // First, let's see if the grammar content is correct
    assert!(
        grammar.contains("$.statement"),
        "Grammar should contain $.statement"
    );
    assert!(
        !grammar.contains("$.state'"),
        "Grammar should NOT contain $.state'"
    );

    let result = parse_grammar_js_v2(grammar);
    match result {
        Ok(parsed) => {
            println!("Inline rules: {:?}", parsed.inline);
            assert_eq!(parsed.inline.len(), 2);
            assert!(parsed.inline.contains(&"statement".to_string()));
            assert!(parsed.inline.contains(&"expression".to_string()));
        }
        Err(e) => {
            panic!("Failed to parse: {}", e);
        }
    }
}

#[test]
fn test_string_content_preservation() {
    // Test that the parser doesn't corrupt content
    let test_cases = vec![
        ("$.statement", "statement"),
        ("$.expr_stmt", "expr_stmt"),
        ("$.test_123", "test_123"),
    ];

    for (input, expected) in test_cases {
        let grammar = format!(
            r#"
module.exports = grammar({{
  name: 'test',
  rules: {{
    rule: $ => {}
  }}
}});
"#,
            input
        );

        let result = parse_grammar_js_v2(&grammar);
        match result {
            Ok(parsed) => {
                if let Some(rule) = parsed.rules.get("rule") {
                    match rule {
                        rust_sitter_tool::grammar_js::Rule::Symbol { name } => {
                            assert_eq!(name, expected, "Expected {} but got {}", expected, name);
                        }
                        _ => panic!("Expected Symbol rule"),
                    }
                }
            }
            Err(e) => {
                panic!("Failed to parse: {}", e);
            }
        }
    }
}
