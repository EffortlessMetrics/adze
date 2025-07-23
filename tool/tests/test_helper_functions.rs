use rust_sitter_tool::grammar_js::{GrammarJsParserV3, GrammarJsConverter};

#[test]
fn test_comma_sep_helper() {
    let grammar = r#"
module.exports = grammar({
    name: 'test',
    
    rules: {
        list: $ => {
            const commaSep = (rule) => optional(seq(rule, repeat(seq(',', rule))));
            return commaSep($.expression);
        },
        
        expression: $ => /[a-z]+/
    }
});
"#;

    println!("Testing commaSep helper function...");
    let mut parser = GrammarJsParserV3::new(grammar.to_string());
    let grammar_js = parser.parse().expect("Failed to parse grammar");
    
    // Convert to IR
    let mut converter = GrammarJsConverter::new(grammar_js);
    let ir_grammar = converter.convert().expect("Failed to convert to IR");
    
    // Check that the list rule was properly expanded
    println!("IR Grammar rules: {:?}", ir_grammar.rules.len());
    assert!(ir_grammar.rules.len() > 0);
}

#[test]
fn test_comma_sep1_helper() {
    let grammar = r#"
module.exports = grammar({
    name: 'test',
    
    rules: {
        list: $ => {
            const commaSep1 = (rule) => seq(rule, repeat(seq(',', rule)));
            return commaSep1($.item);
        },
        
        item: $ => /\w+/
    }
});
"#;

    println!("Testing commaSep1 helper function...");
    let mut parser = GrammarJsParserV3::new(grammar.to_string());
    let grammar_js = parser.parse().expect("Failed to parse grammar");
    
    // Convert to IR
    let mut converter = GrammarJsConverter::new(grammar_js);
    let ir_grammar = converter.convert().expect("Failed to convert to IR");
    
    println!("Rules converted: {:?}", ir_grammar.rules.len());
    assert!(ir_grammar.rules.len() > 0);
}

#[test]
fn test_parens_helper() {
    let grammar = r#"
module.exports = grammar({
    name: 'test',
    
    rules: {
        expression: $ => {
            const parens = (rule) => seq('(', rule, ')');
            return choice(
                $.number,
                parens($.expression)
            );
        },
        
        number: $ => /\d+/
    }
});
"#;

    println!("Testing parens helper function...");
    let mut parser = GrammarJsParserV3::new(grammar.to_string());
    let grammar_js = parser.parse().expect("Failed to parse grammar");
    
    // Check the parsed structure
    if let Some(expr_rule) = grammar_js.rules.get("expression") {
        println!("Expression rule: {:?}", expr_rule);
    }
    
    // Convert to IR
    let mut converter = GrammarJsConverter::new(grammar_js);
    let ir_grammar = converter.convert().expect("Failed to convert to IR");
    
    println!("Grammar converted successfully with {} rules", ir_grammar.rules.len());
    assert!(ir_grammar.rules.len() >= 2);
}

#[test]
fn test_multiple_helpers() {
    let grammar = r#"
module.exports = grammar({
    name: 'test',
    
    rules: {
        program: $ => {
            const commaSep = (rule) => optional(seq(rule, repeat(seq(',', rule))));
            const braces = (rule) => seq('{', rule, '}');
            
            return braces(commaSep($.statement));
        },
        
        statement: $ => /[a-z]+/
    }
});
"#;

    println!("Testing multiple helper functions...");
    let mut parser = GrammarJsParserV3::new(grammar.to_string());
    let grammar_js = parser.parse().expect("Failed to parse grammar");
    
    // Convert to IR
    let mut converter = GrammarJsConverter::new(grammar_js);
    let ir_grammar = converter.convert().expect("Failed to convert to IR");
    
    println!("Grammar converted with {} rules", ir_grammar.rules.len());
    assert!(ir_grammar.rules.len() >= 2);
}