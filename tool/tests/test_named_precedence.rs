use adze_tool::grammar_js::GrammarJsParserV3;

#[test]
fn test_named_precedence() {
    let grammar = r#"
module.exports = grammar({
    name: 'test',
    
    precedences: $ => [
        ['call', 'member'],
        ['unary', 'binary'],
        ['multiply', 'add'],
    ],
    
    rules: {
        expression: $ => choice(
            prec('call', seq($.expression, '(', ')')),
            prec('member', seq($.expression, '.', $.identifier)),
            prec('unary', seq('-', $.expression)),
            prec('binary', seq($.expression, '+', $.expression)),
            prec('multiply', seq($.expression, '*', $.expression)),
            prec('add', seq($.expression, '+', $.expression)),
            $.identifier
        ),
        
        identifier: $ => /[a-z]+/
    }
});
"#;

    println!("Testing named precedence levels...");
    let mut parser = GrammarJsParserV3::new(grammar.to_string());
    match parser.parse() {
        Ok(g) => {
            println!("Success! Grammar: {:?}", g.name);
            println!("Precedences: {:?}", g.precedences);
            println!("Rules: {:?}", g.rules.keys().collect::<Vec<_>>());
        }
        Err(e) => {
            println!("Failed to parse: {}", e);
        }
    }
}

#[test]
fn test_named_precedence_with_numeric() {
    let grammar = r#"
module.exports = grammar({
    name: 'test',
    
    precedences: $ => [
        ['high', 10],
        ['medium', 5],
        ['low', 1],
    ],
    
    rules: {
        expression: $ => choice(
            prec('high', $.primary),
            prec('medium', $.secondary),
            prec('low', $.tertiary),
            prec(0, $.base)
        ),
        
        primary: $ => 'primary',
        secondary: $ => 'secondary',
        tertiary: $ => 'tertiary',
        base: $ => 'base'
    }
});
"#;

    println!("Testing mixed named and numeric precedence...");
    let mut parser = GrammarJsParserV3::new(grammar.to_string());
    match parser.parse() {
        Ok(g) => {
            println!("Success! Grammar: {:?}", g.name);
            println!("Precedences: {:?}", g.precedences);
        }
        Err(e) => {
            println!("Failed to parse: {}", e);
        }
    }
}
