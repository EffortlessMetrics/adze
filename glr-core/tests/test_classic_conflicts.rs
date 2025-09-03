// Test classic shift-reduce conflicts
use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

#[test]
fn test_classic_shift_reduce_conflict() -> Result<(), Box<dyn std::error::Error>> {
    // Classic if-then-else grammar with dangling else problem
    let mut grammar = Grammar::new("if_then_else".to_string());

    // Terminals
    let if_id = SymbolId(1);
    let then_id = SymbolId(2);
    let else_id = SymbolId(3);
    let expr_id = SymbolId(4);
    let stmt_id = SymbolId(5);

    grammar.tokens.insert(
        if_id,
        Token {
            name: "if".to_string(),
            pattern: TokenPattern::String("if".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        then_id,
        Token {
            name: "then".to_string(),
            pattern: TokenPattern::String("then".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        else_id,
        Token {
            name: "else".to_string(),
            pattern: TokenPattern::String("else".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        expr_id,
        Token {
            name: "expr".to_string(),
            pattern: TokenPattern::String("expr".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        stmt_id,
        Token {
            name: "stmt".to_string(),
            pattern: TokenPattern::String("stmt".to_string()),
            fragile: false,
        },
    );

    // Non-terminal S
    let s_id = SymbolId(10);
    grammar.rule_names.insert(s_id, "S".to_string());

    // Rules creating the dangling else problem
    grammar.rules.insert(
        s_id,
        vec![
            // S → if expr then S
            Rule {
                lhs: s_id,
                rhs: vec![
                    Symbol::Terminal(if_id),
                    Symbol::Terminal(expr_id),
                    Symbol::Terminal(then_id),
                    Symbol::NonTerminal(s_id),
                ],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: vec![],
            },
            // S → if expr then S else S
            Rule {
                lhs: s_id,
                rhs: vec![
                    Symbol::Terminal(if_id),
                    Symbol::Terminal(expr_id),
                    Symbol::Terminal(then_id),
                    Symbol::NonTerminal(s_id),
                    Symbol::Terminal(else_id),
                    Symbol::NonTerminal(s_id),
                ],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
            // S → stmt
            Rule {
                lhs: s_id,
                rhs: vec![Symbol::Terminal(stmt_id)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(2),
                fields: vec![],
            },
        ],
    );

    let first_follow = FirstFollowSets::compute(&grammar)?;
    let parse_table = build_lr1_automaton(&grammar, &first_follow)?;

    println!("\n=== Testing Classic Shift-Reduce Conflict (Dangling Else) ===");
    println!("Grammar:");
    println!("  S → if expr then S");
    println!("  S → if expr then S else S");
    println!("  S → stmt");

    // Look for conflicts (multiple actions)
    let mut fork_count = 0;
    for state in 0..parse_table.state_count {
        for sym in 0..parse_table.symbol_count {
            let actions = &parse_table.action_table[state][sym];
            if actions.len() > 1 {
                fork_count += 1;
                println!(
                    "\nFound conflict in state {}, symbol {}: {} actions",
                    state,
                    sym,
                    actions.len()
                );
                for (i, action) in actions.iter().enumerate() {
                    println!("  [{}] {:?}", i, action);
                }
            }
        }
    }

    // The dangling else problem should create a shift-reduce conflict
    // When we see 'else' after 'if expr then if expr then stmt',
    // we can either:
    // 1. Shift the 'else' (attach to inner if)
    // 2. Reduce the inner if-then (attach to outer if)

    assert!(
        fork_count > 0,
        "Expected shift-reduce conflicts for dangling else problem, but found none"
    );
    Ok(())
}

#[test]
fn test_expression_ambiguity() -> Result<(), Box<dyn std::error::Error>> {
    // Arithmetic expression grammar: E → E + E | E * E | num
    let mut grammar = Grammar::new("expr".to_string());

    // Terminals
    let num_id = SymbolId(1);
    let plus_id = SymbolId(2);
    let times_id = SymbolId(3);

    grammar.tokens.insert(
        num_id,
        Token {
            name: "num".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        plus_id,
        Token {
            name: "+".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        times_id,
        Token {
            name: "*".to_string(),
            pattern: TokenPattern::String("*".to_string()),
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
            // E → E + E (no precedence)
            Rule {
                lhs: e_id,
                rhs: vec![
                    Symbol::NonTerminal(e_id),
                    Symbol::Terminal(plus_id),
                    Symbol::NonTerminal(e_id),
                ],
                precedence: None, // No precedence!
                associativity: None,
                production_id: ProductionId(0),
                fields: vec![],
            },
            // E → E * E (no precedence)
            Rule {
                lhs: e_id,
                rhs: vec![
                    Symbol::NonTerminal(e_id),
                    Symbol::Terminal(times_id),
                    Symbol::NonTerminal(e_id),
                ],
                precedence: None, // No precedence!
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
            // E → num
            Rule {
                lhs: e_id,
                rhs: vec![Symbol::Terminal(num_id)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(2),
                fields: vec![],
            },
        ],
    );

    let first_follow = FirstFollowSets::compute(&grammar)?;
    let parse_table = build_lr1_automaton(&grammar, &first_follow)?;

    println!("\n=== Testing Expression Ambiguity ===");
    println!("Grammar (no precedence):");
    println!("  E → E + E");
    println!("  E → E * E");
    println!("  E → num");

    // Count conflicts (multiple actions)
    let mut fork_count = 0;
    for state in 0..parse_table.state_count {
        for sym in 0..parse_table.symbol_count {
            if parse_table.action_table[state][sym].len() > 1 {
                fork_count += 1;
            }
        }
    }

    println!("Found {} Fork actions", fork_count);

    // Without precedence, expressions like "1 + 2 * 3" are ambiguous
    assert!(
        fork_count > 0,
        "Expected conflicts in expression grammar without precedence"
    );
    Ok(())
}
