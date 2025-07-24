// Minimal test for parentheses parsing issue

use rust_sitter_ir::{Grammar, Rule, Symbol, Token, TokenPattern, SymbolId, ProductionId};
use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
use std::collections::HashMap;

fn main() {
    let mut grammar = Grammar::new("test".to_string());
    
    // Terminals
    let num_id = SymbolId(1);
    let lparen_id = SymbolId(2);
    let rparen_id = SymbolId(3);
    
    grammar.tokens.insert(num_id, Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(lparen_id, Token {
        name: "lparen".to_string(),
        pattern: TokenPattern::String("(".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(rparen_id, Token {
        name: "rparen".to_string(),
        pattern: TokenPattern::String(")".to_string()),
        fragile: false,
    });
    
    // Non-terminal
    let expr_id = SymbolId(10);
    grammar.rule_names.insert(expr_id, "expr".to_string());
    
    // Rules
    // expr → number
    grammar.rules.insert(SymbolId(20), Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
        fields: vec![],
    });
    
    // expr → ( expr )
    grammar.rules.insert(SymbolId(21), Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(lparen_id), Symbol::NonTerminal(expr_id), Symbol::Terminal(rparen_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(1),
        fields: vec![],
    });
    
    println!("Grammar created with {} rules", grammar.rules.len());
    
    // Build parse table
    let first_follow = FirstFollowSets::compute(&grammar);
    println!("\nFirst sets:");
    for (symbol, first_set) in &first_follow.first {
        println!("  FIRST({}) = {:?}", symbol.0, first_set);
    }
    
    println!("\nFollow sets:");
    for (symbol, follow_set) in &first_follow.follow {
        println!("  FOLLOW({}) = {:?}", symbol.0, follow_set);
    }
    
    match build_lr1_automaton(&grammar, &first_follow) {
        Ok(table) => {
            println!("\nParse table built successfully!");
            println!("Number of states: {}", table.states.len());
            
            // Print actions for each state
            for (state_idx, state) in table.states.iter().enumerate() {
                println!("\nState {}:", state_idx);
                for (symbol, action) in &state.actions {
                    println!("  On symbol {}: {:?}", symbol.0, action);
                }
                if let Some(goto) = &state.goto {
                    for (symbol, target) in goto {
                        println!("  GOTO on {}: state {}", symbol.0, target.0);
                    }
                }
            }
        }
        Err(e) => {
            println!("\nError building parse table: {:?}", e);
        }
    }
}