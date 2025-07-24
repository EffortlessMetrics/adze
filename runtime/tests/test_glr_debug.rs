// Debug GLR parsing

use rust_sitter::glr_parser::GLRParser;
use rust_sitter_glr_core::{
    build_lr1_automaton, FirstFollowSets,
};
use rust_sitter_ir::{
    Grammar, Rule, Symbol, Token, TokenPattern, 
    ProductionId, SymbolId,
};

#[test]
fn debug_simple_parse() {
    let mut grammar = Grammar::new("debug".to_string());
    
    // Single token 'a'
    grammar.tokens.insert(SymbolId(1), Token {
        name: "a".to_string(),
        pattern: TokenPattern::String("a".to_string()),
        fragile: false,
    });
    
    // S -> a
    let s_id = SymbolId(10);
    grammar.rules.insert(s_id, Rule {
        lhs: s_id,
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    
    println!("\nBuilding parse table...");
    let first_follow = FirstFollowSets::compute(&grammar);
    let table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    println!("Parse table:");
    println!("  States: {}", table.state_count);
    println!("  Symbols: {}", table.symbol_count);
    println!("  Symbol mapping:");
    for (sym, idx) in &table.symbol_to_index {
        println!("    SymbolId({}) -> index {}", sym.0, idx);
    }
    
    println!("\nCreating parser...");
    let mut parser = GLRParser::new(table, grammar);
    
    println!("\nProcessing token 'a'...");
    parser.process_token(SymbolId(1), "a", 0);
    
    println!("\nChecking result before EOF...");
    let result = parser.get_best_parse();
    println!("Result before EOF: {:?}", result.is_some());
    
    println!("\nProcessing EOF...");
    parser.process_eof();
    
    println!("\nChecking final result...");
    let final_result = parser.get_best_parse();
    println!("Final result: {:?}", final_result.is_some());
    if let Some(tree) = final_result {
        println!("Tree symbol: {}", tree.node.symbol_id.0);
    }
}