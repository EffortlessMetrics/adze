use rust_sitter_tool::grammar_js::{GrammarJsParserV3, parse_grammar_js_v2};
use anyhow::Result;

#[test]
fn test_simple_function_block() {
    let grammar = r#"
module.exports = grammar({
    name: 'test',
    
    rules: {
        // Simple function block
        list: $ => {
            const items = ['a', 'b', 'c'];
            return choice(...items.map(item => seq(item, $.identifier)));
        },
        
        identifier: $ => /[a-z]+/
    }
});
"#;

    println!("Testing simple function block...");
    let mut parser = GrammarJsParserV3::new(grammar.to_string());
    match parser.parse() {
        Ok(g) => {
            println!("Success! Grammar: {:?}", g.name);
            println!("Rules: {:?}", g.rules.keys().collect::<Vec<_>>());
        }
        Err(e) => {
            println!("Failed to parse: {}", e);
        }
    }
}

#[test]
fn test_helper_function_block() {
    let grammar = r#"
module.exports = grammar({
    name: 'test',
    
    rules: {
        // Helper function usage
        list: $ => {
            const commaSep = (rule) => optional(seq(rule, repeat(seq(',', rule))));
            return commaSep($.expression);
        },
        
        expression: $ => /[a-z]+/
    }
});
"#;

    println!("Testing helper function block...");
    let mut parser = GrammarJsParserV3::new(grammar.to_string());
    match parser.parse() {
        Ok(g) => {
            println!("Success! Grammar: {:?}", g.name);
            println!("Rules: {:?}", g.rules.keys().collect::<Vec<_>>());
        }
        Err(e) => {
            println!("Failed to parse: {}", e);
        }
    }
}

#[test]
fn test_complex_function_block() {
    let grammar = r#"
module.exports = grammar({
    name: 'javascript',
    
    word: $ => $.identifier,
    
    rules: {
        program: $ => repeat($.statement),
        
        statement: $ => {
            const table = [
                ['+', 'left', 1],
                ['-', 'left', 1],
                ['*', 'left', 2],
                ['/', 'left', 2],
            ];
            
            return choice(
                $.expression_statement,
                $.block_statement
            );
        },
        
        expression_statement: $ => seq($.expression, ';'),
        block_statement: $ => seq('{', repeat($.statement), '}'),
        expression: $ => $.identifier,
        identifier: $ => /[a-zA-Z_]\w*/
    }
});
"#;

    println!("Testing complex function block...");
    let mut parser = GrammarJsParserV3::new(grammar.to_string());
    match parser.parse() {
        Ok(g) => {
            println!("Success! Grammar: {:?}", g.name);
            println!("Rules: {:?}", g.rules.keys().collect::<Vec<_>>());
        }
        Err(e) => {
            println!("Failed to parse: {}", e);
        }
    }
}