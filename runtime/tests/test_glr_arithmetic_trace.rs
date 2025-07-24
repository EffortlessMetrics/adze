// Trace arithmetic grammar LR(1) construction
use rust_sitter_glr_core::{
    FirstFollowSets, ItemSetCollection, build_lr1_automaton,
};
use rust_sitter_ir::{
    Grammar, Rule, Symbol, Token, TokenPattern,
    ProductionId, SymbolId, Associativity, Precedence, PrecedenceKind,
};

fn build_arithmetic_grammar() -> Grammar {
    let mut grammar = Grammar::new("arithmetic".to_string());
    
    // Tokens (start from 1, as 0 is reserved for EOF)
    grammar.tokens.insert(SymbolId(1), Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(SymbolId(2), Token {
        name: "plus".to_string(),
        pattern: TokenPattern::String("+".to_string()),
        fragile: false,
    });
    
    // Non-terminals
    let expr_id = SymbolId(10);
    grammar.rule_names.insert(expr_id, "expr".to_string());
    
    // Rules
    // expr -> number
    grammar.rules.insert(expr_id, Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    
    // expr -> expr + expr
    grammar.rules.insert(SymbolId(11), Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(expr_id),
        ],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(1),
    });
    
    grammar
}

#[test]
fn test_arithmetic_trace() {
    let grammar = build_arithmetic_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    
    println!("Grammar rules:");
    for (id, rule) in &grammar.rules {
        println!("  Rule {} (prod {}): {:?}", id.0, rule.production_id.0, rule);
    }
    
    // Build the item sets
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &first_follow);
    
    println!("\nItem sets:");
    for (i, set) in collection.sets.iter().enumerate() {
        println!("\nState {}:", i);
        for item in &set.items {
            // Find the rule to print it nicely
            if let Some(rule) = grammar.rules.values().find(|r| r.production_id.0 == item.rule_id.0) {
                print!("  [{}] ", item.rule_id.0);
                print!("{} ->", rule.lhs.0);
                for (j, sym) in rule.rhs.iter().enumerate() {
                    if j == item.position {
                        print!(" •");
                    }
                    match sym {
                        Symbol::Terminal(id) => print!(" t{}", id.0),
                        Symbol::NonTerminal(id) => print!(" nt{}", id.0),
                        Symbol::External(id) => print!(" e{}", id.0),
                    }
                }
                if item.position == rule.rhs.len() {
                    print!(" •");
                }
                println!(", {}", item.lookahead.0);
            }
        }
    }
    
    // Build parse table to check actions
    let table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    println!("\nState 0 actions:");
    for (symbol, &idx) in &table.symbol_to_index {
        let action = &table.action_table[0][idx];
        if !matches!(action, rust_sitter_glr_core::Action::Error) {
            println!("  Symbol {} -> {:?}", symbol.0, action);
        }
    }
    
    println!("\nState 1 actions:");
    for (symbol, &idx) in &table.symbol_to_index {
        let action = &table.action_table[1][idx];
        if !matches!(action, rust_sitter_glr_core::Action::Error) {
            println!("  Symbol {} -> {:?}", symbol.0, action);
        }
    }
}