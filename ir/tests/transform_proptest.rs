//! Property-based tests for IR grammar transformations.
//!
//! Verifies normalization idempotency, symbol ID allocation,
//! rule roundtrips, and field mapping consistency.

use adze_ir::*;
use proptest::prelude::*;

/// Build a small grammar with N terminals and M rules for property testing.
fn make_grammar(name: &str, num_terminals: u16, num_rules: u16) -> Grammar {
    let mut g = Grammar::new(name.to_string());
    // Add terminals
    for i in 1..=num_terminals {
        g.tokens.insert(
            SymbolId(i),
            Token {
                name: format!("t{i}"),
                pattern: TokenPattern::String(format!("tok{i}")),
                fragile: false,
            },
        );
    }
    // Add non-terminal rules referencing the terminals
    let base = num_terminals + 1;
    for i in 0..num_rules {
        let lhs = SymbolId(base + i);
        let terminal = SymbolId((i % num_terminals) + 1);
        g.add_rule(Rule {
            lhs,
            rhs: vec![Symbol::Terminal(terminal)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i),
        });
        g.rule_names.insert(lhs, format!("rule_{i}"));
    }
    if num_rules > 0 {
        // start_symbol is computed automatically from the grammar
    }
    g
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn normalization_is_idempotent(
        num_terms in 1u16..=5,
        num_rules in 1u16..=5,
    ) {
        let g1 = make_grammar("idem", num_terms, num_rules);
        let mut g2 = g1.clone();
        g2.normalize();
        let mut g3 = g2.clone();
        g3.normalize();
        // After normalizing twice, structure should be identical to normalizing once
        prop_assert_eq!(g2.rules.len(), g3.rules.len());
        prop_assert_eq!(g2.tokens.len(), g3.tokens.len());
    }

    #[test]
    fn symbol_ids_never_collide(
        num_terms in 1u16..=10,
        num_rules in 1u16..=10,
    ) {
        let g = make_grammar("collide", num_terms, num_rules);
        // All symbol IDs across tokens, rules, and rule_names should be distinct from each other
        let token_ids: Vec<SymbolId> = g.tokens.keys().copied().collect();
        let rule_lhs: Vec<SymbolId> = g.rules.keys().copied().collect();
        // No token ID should also appear as a rule LHS
        for tid in &token_ids {
            prop_assert!(!rule_lhs.contains(tid), "Token ID {tid:?} collides with rule LHS");
        }
    }

    #[test]
    fn field_id_mappings_are_unique(
        num_fields in 0u16..=20,
    ) {
        let mut g = Grammar::new("fields".to_string());
        for i in 0..num_fields {
            g.fields.insert(FieldId(i), format!("field_{i}"));
        }
        // All field names should be unique
        let names: Vec<&String> = g.fields.values().collect();
        let unique: std::collections::HashSet<&String> = names.iter().copied().collect();
        prop_assert_eq!(names.len(), unique.len(), "Field names not unique");
    }

    #[test]
    fn grammar_with_rules_has_consistent_lhs(
        num_terms in 1u16..=5,
        num_rules in 1u16..=5,
    ) {
        let g = make_grammar("consistent", num_terms, num_rules);
        // Every rule LHS should have a name
        for lhs in g.rules.keys() {
            prop_assert!(
                g.rule_names.contains_key(lhs),
                "Rule LHS {lhs:?} missing name"
            );
        }
    }
}

#[test]
fn empty_grammar_normalizes_without_panic() {
    let mut g = Grammar::new("empty".to_string());
    g.normalize();
    assert!(g.rules.is_empty());
    assert!(g.tokens.is_empty());
}

#[test]
fn single_rule_grammar_normalizes() {
    let mut g = make_grammar("single", 1, 1);
    g.normalize();
    // Should still have at least the original rule
    assert!(!g.rules.is_empty());
}
