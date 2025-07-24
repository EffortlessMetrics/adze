// Test GLR parser with simple inputs

use rust_sitter::glr_parser::GLRParser;
use rust_sitter_glr_core::{
    build_lr1_automaton, FirstFollowSets,
};
use rust_sitter_ir::{
    Grammar, Rule, Symbol, Token, TokenPattern, 
    ProductionId, SymbolId,
};

#[test]
fn test_parse_single_token() {
    let mut grammar = Grammar::new("single".to_string());
    
    // Token 'a'
    grammar.tokens.insert(SymbolId(1), Token {
        name: "a".to_string(),
        pattern: TokenPattern::String("a".to_string()),
        fragile: false,
    });
    
    // Rule: S -> a
    let s_id = SymbolId(10);
    grammar.rules.insert(s_id, Rule {
        lhs: s_id,
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    
    let first_follow = FirstFollowSets::compute(&grammar);
    let table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    println!("\nParse table info:");
    println!("  States: {}", table.state_count);
    println!("  Symbols: {}", table.symbol_count);
    println!("  Symbol mapping: {:?}", table.symbol_to_index);
    
    let mut parser = GLRParser::new(table, grammar.clone());
    
    // Parse just "a"
    parser.process_token(SymbolId(1), "a", 0);
    
    // Process EOF
    parser.process_eof();
    
    let result = parser.get_best_parse();
    assert!(result.is_some(), "Should have a parse result");
}

#[test]
fn test_parse_two_tokens() {
    let mut grammar = Grammar::new("two".to_string());
    
    // Tokens
    grammar.tokens.insert(SymbolId(1), Token {
        name: "a".to_string(),
        pattern: TokenPattern::String("a".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(SymbolId(2), Token {
        name: "b".to_string(),
        pattern: TokenPattern::String("b".to_string()),
        fragile: false,
    });
    
    // Rule: S -> a b
    let s_id = SymbolId(10);
    grammar.rules.insert(s_id, Rule {
        lhs: s_id,
        rhs: vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(2)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    
    let first_follow = FirstFollowSets::compute(&grammar);
    let table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    let mut parser = GLRParser::new(table, grammar.clone());
    
    // Parse "a b"
    parser.process_token(SymbolId(1), "a", 0);
    parser.process_token(SymbolId(2), "b", 2);
    parser.process_eof();
    
    let result = parser.get_best_parse();
    assert!(result.is_some(), "Should have a parse result");
}