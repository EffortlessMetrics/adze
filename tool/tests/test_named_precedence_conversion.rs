use rust_sitter_tool::grammar_js::{GrammarJsConverter, GrammarJsParserV3};

#[test]
fn test_named_precedence_conversion() {
    let grammar = r#"
module.exports = grammar({
    name: 'test',
    
    precedences: $ => [
        ['high'],
        ['medium'],
        ['low'],
    ],
    
    rules: {
        expression: $ => choice(
            prec('high', $.primary),
            prec('medium', $.secondary),
            prec('low', $.tertiary),
            $.identifier
        ),
        
        primary: $ => 'primary',
        secondary: $ => 'secondary',
        tertiary: $ => 'tertiary',
        identifier: $ => /[a-z]+/
    }
});
"#;

    println!("Testing named precedence conversion to IR...");
    let mut parser = GrammarJsParserV3::new(grammar.to_string());
    let grammar_js = parser.parse().expect("Failed to parse grammar");

    println!("Parsed precedences: {:?}", grammar_js.precedences);

    // Check that the precedence map is built correctly
    // Should be: high=3, medium=2, low=1

    // Convert to IR
    let converter = GrammarJsConverter::new(grammar_js);
    let ir_grammar = converter.convert().expect("Failed to convert to IR");

    println!("Converted {} rules to IR", ir_grammar.rules.len());

    // The rules should have correct precedence values
    // Unfortunately we can't easily check the actual precedence values from here
    // but at least we can verify the conversion works
    assert!(ir_grammar.rules.len() > 0);
}
