// Test that Fork actions are properly generated for ambiguous grammars
use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

#[test]
fn test_fork_action_generation() -> Result<(), Box<dyn std::error::Error>> {
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
            // Rule 1: E → a
            Rule {
                lhs: e_id,
                rhs: vec![Symbol::Terminal(a_id)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: vec![],
            },
            // Rule 2: E → E E
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
    let parse_table = build_lr1_automaton(&grammar, &first_follow)?;

    // Debug: Print parse table
    println!("\n=== Parse Table ===");
    println!(
        "States: {}, Symbols: {}",
        parse_table.state_count, parse_table.symbol_count
    );

    let mut has_fork = false;
    for (state_idx, state_actions) in parse_table.action_table.iter().enumerate() {
        for (sym_idx, actions) in state_actions.iter().enumerate() {
            // Now action_table[state][symbol] is Vec<Action>
            if actions.len() > 1 {
                has_fork = true;
                println!(
                    "State {}, Symbol {}: Multiple actions ({})",
                    state_idx,
                    sym_idx,
                    actions.len()
                );
            }
        }
    }

    assert!(
        has_fork,
        "Expected Fork actions in parse table for ambiguous grammar"
    );
    Ok(())
}
