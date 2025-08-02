// Test LR(1) item set generation to understand why conflicts aren't appearing
use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

#[test]
fn test_lr1_conflict_detection() {
    let mut grammar = Grammar::new("ambiguous".to_string());

    // Terminal 'a'
    let a_id = SymbolId(1);
    grammar.tokens.insert(
        a_id,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );

    // Non-terminal E
    let e_id = SymbolId(10);
    grammar.rule_names.insert(e_id, "E".to_string());

    // Rules for E
    grammar.rules.insert(
        e_id,
        vec![
            // Rule 0: E → a (production_id = 0)
            Rule {
                lhs: e_id,
                rhs: vec![Symbol::Terminal(a_id)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: vec![],
            },
            // Rule 1: E → E E (production_id = 1)
            Rule {
                lhs: e_id,
                rhs: vec![Symbol::NonTerminal(e_id), Symbol::NonTerminal(e_id)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
        ],
    );

    let first_follow = FirstFollowSets::compute(&grammar);
    println!("\n=== First/Follow Sets ===");
    // Build parse table to get internal state info
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    println!("\n=== Parse Table Analysis ===");
    println!("Total states: {}", parse_table.state_count);

    // Analyze each state for conflicts
    for state_idx in 0..parse_table.state_count {
        println!("\nState {}:", state_idx);
        let mut has_conflict = false;

        // Check each symbol for multiple actions
        for sym_idx in 0..parse_table.symbol_count {
            let action = &parse_table.action_table[state_idx][sym_idx];
            match action {
                rust_sitter_glr_core::Action::Fork(actions) => {
                    has_conflict = true;
                    println!("  Symbol {}: Fork with {} actions", sym_idx, actions.len());
                }
                rust_sitter_glr_core::Action::Error => {}
                _ => {
                    // Find symbol for this index
                    let symbol = parse_table
                        .symbol_to_index
                        .iter()
                        .find(|(_, idx)| **idx == sym_idx)
                        .map(|(sym, _)| sym);
                    if let Some(sym) = symbol {
                        println!("  Symbol {} (idx {}): {:?}", sym.0, sym_idx, action);
                    }
                }
            }
        }

        if has_conflict {
            println!("  *** CONFLICT FOUND ***");
        }
    }

    // Check if any Fork actions were generated
    let has_forks = (0..parse_table.state_count).any(|state| {
        (0..parse_table.symbol_count).any(|sym| {
            matches!(
                &parse_table.action_table[state][sym],
                rust_sitter_glr_core::Action::Fork(_)
            )
        })
    });

    assert!(
        has_forks,
        "Expected Fork actions in parse table for ambiguous grammar"
    );
}
