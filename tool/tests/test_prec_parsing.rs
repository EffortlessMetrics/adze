use rust_sitter_tool::grammar_js::GrammarJsParserV3;

#[test]
fn test_parse_precedence_rules() {
    // Test parsing various precedence patterns
    let test_cases = vec![
        ("prec(1, $.expression)", "Prec with simple symbol"),
        (
            "prec.left(2, seq($.expr, '+', $.expr))",
            "Left associative with sequence",
        ),
        (
            "prec.right(3, seq($.expr, '=', $.expr))",
            "Right associative assignment",
        ),
        ("prec.dynamic(1, $.member_expression)", "Dynamic precedence"),
    ];

    for (input, description) in test_cases {
        println!("\nTesting: {}", description);
        println!("Input: {}", input);

        // Create a minimal grammar to test parsing
        let grammar_content = format!(
            r#"
module.exports = grammar({{
  name: 'test',
  
  rules: {{
    test_rule: $ => {}
  }}
}})
"#,
            input
        );

        let mut parser = GrammarJsParserV3::new(grammar_content);
        match parser.parse() {
            Ok(grammar) => {
                println!("✓ Successfully parsed!");
                if let Some(rule) = grammar.rules.get("test_rule") {
                    println!("  Rule type: {:?}", std::mem::discriminant(rule));
                }
            }
            Err(e) => {
                println!("✗ Failed to parse: {}", e);
            }
        }
    }
}

#[test]
fn test_javascript_binary_expressions() {
    // Test a simplified version of JavaScript's binary expression rules
    let grammar_content = r#"
module.exports = grammar({
  name: 'javascript_binary',
  
  rules: {
    expression: $ => choice(
      $.binary_expression,
      $.number
    ),
    
    binary_expression: $ => choice(
      prec.left(1, seq(
        field('left', $.expression),
        field('operator', '+'),
        field('right', $.expression)
      )),
      prec.left(1, seq(
        field('left', $.expression),
        field('operator', '-'),
        field('right', $.expression)
      )),
      prec.left(2, seq(
        field('left', $.expression),
        field('operator', '*'),
        field('right', $.expression)
      )),
      prec.left(2, seq(
        field('left', $.expression),
        field('operator', '/'),
        field('right', $.expression)
      ))
    ),
    
    number: $ => /\d+/
  }
})
"#;

    let mut parser = GrammarJsParserV3::new(grammar_content.to_string());
    match parser.parse() {
        Ok(grammar) => {
            println!("Successfully parsed JavaScript-like binary expressions!");
            println!("Grammar name: {}", grammar.name);
            println!("Number of rules: {}", grammar.rules.len());

            // Check that we have the expected rules
            assert!(grammar.rules.contains_key("expression"));
            assert!(grammar.rules.contains_key("binary_expression"));
            assert!(grammar.rules.contains_key("number"));
        }
        Err(e) => {
            panic!("Failed to parse binary expression grammar: {}", e);
        }
    }
}
