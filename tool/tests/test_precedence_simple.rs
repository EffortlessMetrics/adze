use rust_sitter_tool::grammar_js::{GrammarJsParserV3};

#[test]
fn test_simple_precedence_array() {
    let grammar = r#"
module.exports = grammar({
    name: 'test',
    
    precedences: $ => [
        ['a', 'b'],
        ['c', 'd'],
        ['e', 'f']
    ],
    
    rules: {
        expression: $ => 'test'
    }
});
"#;

    println!("Testing simple precedence array parsing...");
    let mut parser = GrammarJsParserV3::new(grammar.to_string());
    match parser.parse() {
        Ok(g) => {
            println!("Success!");
            println!("Parsed {} precedence groups", g.precedences.len());
            for (i, group) in g.precedences.iter().enumerate() {
                println!("  Group {}: {:?}", i, group);
            }
        }
        Err(e) => {
            println!("Failed: {}", e);
        }
    }
}