// Test that Fork actions are properly generated for ambiguous grammars
use rust_sitter_ir::{Grammar, Rule, Symbol, Token, TokenPattern, SymbolId, ProductionId};
use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton, Action};

#[test]
fn test_fork_action_generation() {
    let mut grammar = Grammar::new("ambiguous".to_string());
    
    // Terminal 'a'
    let a_id = SymbolId(1);
    grammar.tokens.insert(a_id, Token {
        name: "a".to_string(),
        pattern: TokenPattern::String("a".to_string()),
        fragile: false,
    });
    
    // Non-terminal E
    let e_id = SymbolId(10);
    grammar.rule_names.insert(e_id, "E".to_string());
    
    // Rule 1: E → a
    grammar.rules.insert(SymbolId(20), Rule {
        lhs: e_id,
        rhs: vec![Symbol::Terminal(a_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
        fields: vec![],
    });
    
    // Rule 2: E → E E
    grammar.rules.insert(SymbolId(21), Rule {
        lhs: e_id,
        rhs: vec![Symbol::NonTerminal(e_id), Symbol::NonTerminal(e_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(1),
        fields: vec![],
    });
    
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    // Debug: Print parse table
    println!("\n=== Parse Table ===");
    println!("States: {}, Symbols: {}", parse_table.state_count, parse_table.symbol_count);
    
    let mut has_fork = false;
    for (state_idx, state_actions) in parse_table.action_table.iter().enumerate() {
        for (sym_idx, action) in state_actions.iter().enumerate() {
            if let Action::Fork(actions) = action {
                has_fork = true;
                println!("State {}, Symbol {}: Fork with {} actions", state_idx, sym_idx, actions.len());
            }
        }
    }
    
    assert!(has_fork, "Expected Fork actions in parse table for ambiguous grammar");
}