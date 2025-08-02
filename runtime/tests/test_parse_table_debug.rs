// Debug parse table generation for ambiguous grammars
use rust_sitter_ir::{Grammar, Rule, Symbol, Token, TokenPattern, SymbolId, ProductionId};
use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton, Action};

fn create_ambiguous_grammar() -> Grammar {
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
    let rule1 = Rule {
        lhs: e_id,
        rhs: vec![Symbol::Terminal(a_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
        fields: vec![],
    };
    grammar.rules.entry(SymbolId(20)).or_insert_with(Vec::new).push(rule1);
    
    // Rule 2: E → E E
    let rule2 = Rule {
        lhs: e_id,
        rhs: vec![Symbol::NonTerminal(e_id), Symbol::NonTerminal(e_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(1),
        fields: vec![],
    };
    grammar.rules.entry(SymbolId(21)).or_insert_with(Vec::new).push(rule2);
    
    println!("\n=== Checking Grammar ===\nRules count: {}\nTokens count: {}\nRule names count: {}", 
        grammar.rules.len(), grammar.tokens.len(), grammar.rule_names.len());
    
    grammar
}

#[test]
fn test_parse_table_has_conflicts() {
    let grammar = create_ambiguous_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    
    println!("\n=== Grammar Rules ===");
    for (symbol_id, rules) in &grammar.rules {
        for rule in rules {
            println!("Rule for symbol {}: {:?} -> {:?}", symbol_id.0, rule.lhs, rule.rhs);
        }
    }
    
    println!("\n=== First/Follow Sets ===\nFirst sets:");
    for (symbol, set) in first_follow.first() {
        if !set.is_empty() {
            println!("  Symbol {}: {:?}", symbol.0, set.iter().map(|s| s.0).collect::<Vec<_>>());
        }
    }
    
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    println!("\n=== Parse Table Analysis ===");
    println!("States: {}", parse_table.state_count);
    println!("Symbols: {}", parse_table.symbol_count);
    
    // Check for Fork actions
    let mut has_fork = false;
    let mut conflict_count = 0;
    
    println!("\n=== Action Table ===");
    for (state_idx, state_actions) in parse_table.action_table.iter().enumerate() {
        println!("\nState {}:", state_idx);
        for (sym_idx, action) in state_actions.iter().enumerate() {
            // Find symbol for this index
            let symbol = parse_table.symbol_to_index.iter()
                .find(|(_, idx)| **idx == sym_idx)
                .map(|(sym, _)| sym);
            
            if let Some(symbol) = symbol {
                let action_str = match action {
                    Action::Fork(actions) => {
                        has_fork = true;
                        conflict_count += 1;
                        println!("  Symbol {} (idx {}): Fork with {} actions:", symbol.0, sym_idx, actions.len());
                        for (i, fork_action) in actions.iter().enumerate() {
                            println!("    Fork[{}]: {:?}", i, fork_action);
                        }
                        continue;
                    }
                    Action::Error => continue, // Skip errors
                    Action::Shift(s) => format!("Shift({})", s.0),
                    Action::Reduce(r) => format!("Reduce({})", r.0),
                    Action::Accept => "Accept".to_string(),
                };
                println!("  Symbol {} (idx {}): {}", symbol.0, sym_idx, action_str);
            }
        }
    }
    
    println!("\n=== Summary ===");
    println!("Has Fork actions: {}", has_fork);
    println!("Number of conflicts: {}", conflict_count);
    
    // This ambiguous grammar SHOULD have conflicts
    assert!(has_fork || conflict_count > 0, 
            "Ambiguous grammar should have conflicts/forks in parse table");
}