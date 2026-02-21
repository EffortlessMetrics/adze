// Test LR(1) item set generation to understand why conflicts aren't appearing
use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

#[test]
fn test_lr1_conflict_detection() -> Result<(), Box<dyn std::error::Error>> {
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

    let first_follow = FirstFollowSets::compute(&grammar)?;
    println!("\n=== First/Follow Sets ===");
    // Build parse table to get internal state info
    let parse_table = build_lr1_automaton(&grammar, &first_follow)?;

    println!("\n=== Parse Table Analysis ===");
    println!("Total states: {}", parse_table.state_count);

    // Analyze each state for conflicts
    for state_idx in 0..parse_table.state_count {
        println!("\nState {}:", state_idx);
        let mut has_conflict = false;

        // Check each symbol for multiple actions
        for sym_idx in 0..parse_table.symbol_count {
            let actions = &parse_table.action_table[state_idx][sym_idx];
            // Now action_table[state][symbol] is Vec<Action>
            if actions.len() > 1 {
                has_conflict = true;
                println!("  Symbol {}: {} actions", sym_idx, actions.len());
                for action in actions {
                    println!("    - {:?}", action);
                }
            } else if !actions.is_empty() {
                // Find symbol for this index
                let symbol = parse_table
                    .symbol_to_index
                    .iter()
                    .find(|(_, idx)| **idx == sym_idx)
                    .map(|(sym, _)| sym);
                if let Some(sym) = symbol
                    && !matches!(actions[0], adze_glr_core::Action::Error)
                {
                    println!("  Symbol {} (idx {}): {:?}", sym.0, sym_idx, actions[0]);
                }
            }
        }

        if has_conflict {
            println!("  *** CONFLICT FOUND ***");
        }
    }

    // Check if any states have multiple actions (conflicts)
    let has_forks = (0..parse_table.state_count).any(|state| {
        (0..parse_table.symbol_count).any(|sym| parse_table.action_table[state][sym].len() > 1)
    });

    assert!(
        has_forks,
        "Expected Fork actions in parse table for ambiguous grammar"
    );
    Ok(())
}
