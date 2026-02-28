// Property-based tests for FIRST/FOLLOW set computation and parse table invariants.

use adze_glr_core::FirstFollowSets;
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use proptest::prelude::*;

/// Build a simple grammar with N non-terminals and M terminals,
/// where each non-terminal has a rule producing a sequence of terminals.
fn build_simple_grammar(num_nt: usize, num_t: usize) -> Grammar {
    let mut grammar = Grammar::default();
    grammar.name = "prop_test".to_string();

    // Register terminals
    for i in 0..num_t {
        let id = SymbolId(i as u16);
        grammar.tokens.insert(
            id,
            Token {
                name: format!("t{i}"),
                pattern: TokenPattern::String(format!("tok{i}")),
                fragile: false,
            },
        );
    }

    // Register non-terminals with rules
    let nt_base = num_t as u16 + 10; // offset to avoid ID collisions
    for i in 0..num_nt {
        let lhs = SymbolId(nt_base + i as u16);
        grammar
            .rule_names
            .insert(lhs, format!("nt_{i}"));

        // Each non-terminal produces a terminal (if any exist)
        if num_t > 0 {
            let terminal_idx = (i % num_t) as u16;
            grammar.add_rule(Rule {
                lhs,
                rhs: vec![Symbol::Terminal(SymbolId(terminal_idx))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            });
        } else {
            // Just an epsilon rule
            grammar.add_rule(Rule {
                lhs,
                rhs: vec![Symbol::Epsilon],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            });
        }
    }

    grammar
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn first_follow_succeeds_for_simple_grammars(
        num_nt in 1usize..6,
        num_t in 1usize..6,
    ) {
        let grammar = build_simple_grammar(num_nt, num_t);
        let result = FirstFollowSets::compute(&grammar);
        prop_assert!(result.is_ok(), "FIRST/FOLLOW should succeed: {result:?}");
    }

    #[test]
    fn first_set_contains_own_terminal(t_id in 0u16..10) {
        let mut grammar = Grammar::default();
        grammar.name = "test".to_string();
        let sym = SymbolId(t_id);
        grammar.tokens.insert(
            sym,
            Token {
                name: format!("t{t_id}"),
                pattern: TokenPattern::String(format!("v{t_id}")),
                fragile: false,
            },
        );

        // Add a non-terminal that produces this terminal
        let nt = SymbolId(t_id + 100);
        grammar.rule_names.insert(nt, format!("nt_{t_id}"));
        grammar.add_rule(Rule {
            lhs: nt,
            rhs: vec![Symbol::Terminal(sym)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });

        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let first = ff.first(nt);
        prop_assert!(first.is_some(), "FIRST set should exist for non-terminal");
        let first = first.unwrap();
        // The FIRST set of the non-terminal should contain the terminal
        prop_assert!(
            first.contains(t_id as usize),
            "FIRST({nt}) should contain terminal {t_id}"
        );
    }

    #[test]
    fn epsilon_rule_makes_nullable(nt_id in 10u16..20) {
        let mut grammar = Grammar::default();
        grammar.name = "test".to_string();
        // Add a dummy terminal so the grammar has token definitions
        grammar.tokens.insert(
            SymbolId(0),
            Token {
                name: "dummy".to_string(),
                pattern: TokenPattern::String("d".to_string()),
                fragile: false,
            },
        );

        let nt = SymbolId(nt_id);
        grammar.rule_names.insert(nt, format!("nt_{nt_id}"));
        grammar.add_rule(Rule {
            lhs: nt,
            rhs: vec![Symbol::Epsilon],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });

        let ff = FirstFollowSets::compute(&grammar).unwrap();
        prop_assert!(ff.is_nullable(nt), "Non-terminal with epsilon rule should be nullable");
    }

    #[test]
    fn non_nullable_terminal_rule(t_id in 0u16..5) {
        let mut grammar = Grammar::default();
        grammar.name = "test".to_string();
        let t = SymbolId(t_id);
        grammar.tokens.insert(
            t,
            Token {
                name: format!("t{t_id}"),
                pattern: TokenPattern::String(format!("v{t_id}")),
                fragile: false,
            },
        );

        let nt = SymbolId(t_id + 100);
        grammar.rule_names.insert(nt, format!("nt_{t_id}"));
        grammar.add_rule(Rule {
            lhs: nt,
            rhs: vec![Symbol::Terminal(t)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });

        let ff = FirstFollowSets::compute(&grammar).unwrap();
        prop_assert!(!ff.is_nullable(nt), "Non-terminal with only terminal rule should not be nullable");
    }

    #[test]
    fn first_follow_deterministic(num_nt in 1usize..4, num_t in 1usize..4) {
        let grammar = build_simple_grammar(num_nt, num_t);

        let ff1 = FirstFollowSets::compute(&grammar).unwrap();
        let ff2 = FirstFollowSets::compute(&grammar).unwrap();

        // FIRST/FOLLOW computation should be deterministic
        let nt_base = (num_t as u16) + 10;
        for i in 0..num_nt {
            let nt = SymbolId(nt_base + i as u16);
            let first1 = ff1.first(nt);
            let first2 = ff2.first(nt);
            prop_assert_eq!(first1, first2, "FIRST sets should be deterministic for {:?}", nt);

            let follow1 = ff1.follow(nt);
            let follow2 = ff2.follow(nt);
            prop_assert_eq!(follow1, follow2, "FOLLOW sets should be deterministic for {:?}", nt);
        }
    }
}
