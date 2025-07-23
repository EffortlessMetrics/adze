use rust_sitter_tool::grammar_js::GrammarJsParserV3;

#[test]
fn test_debug_function_block() {
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

    println!("Testing function block with spread operator...");
    let parser = GrammarJsParserV3::new(grammar.to_string());
    match parser.parse() {
        Ok(g) => {
            println!("Success!");
            println!("Grammar name: {:?}", g.name);
            println!("Rules found: {:?}", g.rules.keys().collect::<Vec<_>>());
            
            // Print the parsed rule
            if let Some(list_rule) = g.rules.get("list") {
                println!("List rule: {:?}", list_rule);
            }
        }
        Err(e) => {
            println!("Failed to parse: {}", e);
        }
    }
}