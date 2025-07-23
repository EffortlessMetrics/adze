use rust_sitter_tool::grammar_js::{GrammarJsParserV3};

#[test]
fn test_precedence_parsing_debug() {
    let grammar = r#"
module.exports = grammar({
    name: 'test',
    
    precedences: $ => [
        ['high'],
        ['medium'],
        ['low'],
    ],
    
    rules: {
        expression: $ => prec('high', $.identifier),
        identifier: $ => /[a-z]+/
    }
});
"#;

    println!("Testing precedence parsing with debug...");
    let mut parser = GrammarJsParserV3::new(grammar.to_string());
    match parser.parse() {
        Ok(g) => {
            println!("Success!");
            println!("Parsed precedences: {:?}", g.precedences);
            for (i, group) in g.precedences.iter().enumerate() {
                println!("  Group {}: {:?}", i, group);
            }
        }
        Err(e) => {
            println!("Failed: {}", e);
            println!("Error chain: {:?}", e.chain().collect::<Vec<_>>());
        }
    }
}