use rust_sitter_ir::{FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use rust_sitter_tablegen::NodeTypesGenerator;

fn create_arithmetic_grammar() -> Grammar {
    let mut grammar = Grammar::new("arithmetic".to_string());
    
    // Define tokens
    let number_token = Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    };
    let minus_token = Token {
        name: "minus_op".to_string(),
        pattern: TokenPattern::String("-".to_string()),
        fragile: false,
    };
    let star_token = Token {
        name: "star_op".to_string(), 
        pattern: TokenPattern::String("*".to_string()),
        fragile: false,
    };
    let whitespace_token = Token {
        name: "whitespace".to_string(),
        pattern: TokenPattern::Regex(r"\s+".to_string()),
        fragile: false,
    };
    
    // Symbol IDs
    let number_token_id = SymbolId(0);
    let minus_token_id = SymbolId(1);
    let star_token_id = SymbolId(2);
    let whitespace_token_id = SymbolId(3);
    let expression_id = SymbolId(4);
    let binary_expr_id = SymbolId(5);
    let number_expr_id = SymbolId(6);
    
    // Insert tokens
    grammar.tokens.insert(number_token_id, number_token);
    grammar.tokens.insert(minus_token_id, minus_token);
    grammar.tokens.insert(star_token_id, star_token);
    grammar.tokens.insert(whitespace_token_id, whitespace_token);
    
    // Define field IDs
    let left_field = FieldId(0);
    let operator_field = FieldId(1);
    let right_field = FieldId(2);
    
    grammar.fields.insert(left_field, "left".to_string());
    grammar.fields.insert(operator_field, "operator".to_string());
    grammar.fields.insert(right_field, "right".to_string());
    
    // Number expression rule
    let number_expr_rule = Rule {
        lhs: number_expr_id,
        rhs: vec![Symbol::Terminal(number_token_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    grammar.rules.insert(number_expr_id, number_expr_rule);
    
    // Binary expression rule: left operator right
    let binary_expr_rule = Rule {
        lhs: binary_expr_id,
        rhs: vec![
            Symbol::NonTerminal(expression_id), // left
            Symbol::Terminal(minus_token_id),    // operator (simplified - just minus for now)
            Symbol::NonTerminal(expression_id), // right
        ],
        precedence: None,
        associativity: None,
        fields: vec![
            (left_field, 0),     // left field at position 0
            (operator_field, 1), // operator field at position 1
            (right_field, 2),    // right field at position 2
        ],
        production_id: ProductionId(1),
    };
    grammar.rules.insert(binary_expr_id, binary_expr_rule);
    
    // Expression rule (choice - simplified for now)
    // In a real implementation, this would be multiple rules with the same LHS
    let expr_rule = Rule {
        lhs: expression_id,
        rhs: vec![Symbol::NonTerminal(number_expr_id)], // Simplified - just number for now
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    };
    grammar.rules.insert(expression_id, expr_rule);
    
    grammar
}

#[test]
fn test_arithmetic_node_types_generation() {
    let grammar = create_arithmetic_grammar();
    let generator = NodeTypesGenerator::new(&grammar);
    
    let result = generator.generate().expect("Failed to generate NODE_TYPES");
    println!("Generated NODE_TYPES.json:\n{}", result);
    
    // Parse and validate the structure
    let node_types: serde_json::Value = serde_json::from_str(&result)
        .expect("Invalid JSON generated");
    
    assert!(node_types.is_array());
    let nodes = node_types.as_array().unwrap();
    
    // Check that we have some node types
    assert!(!nodes.is_empty());
    
    // Check for literal tokens
    let literal_nodes: Vec<&str> = nodes.iter()
        .filter_map(|n| {
            if n.get("named")?.as_bool()? == false {
                n.get("type")?.as_str()
            } else {
                None
            }
        })
        .collect();
    
    assert!(literal_nodes.contains(&"-"));
    assert!(literal_nodes.contains(&"*"));
}