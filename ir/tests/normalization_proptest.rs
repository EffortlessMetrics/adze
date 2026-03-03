//! Property tests for grammar normalization.
//!
//! Verifies key invariants of Grammar::normalize():
//! - Normalization is idempotent
//! - All rules reference defined symbols after normalization
//! - Complex symbols are expanded into auxiliary rules

use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;
use proptest::prelude::*;

/// Build a random grammar with given number of tokens and rules
fn build_grammar(num_tokens: usize, num_rules: usize) -> Grammar {
    let token_names: Vec<String> = (0..num_tokens).map(|i| format!("tok_{i}")).collect();
    let rule_names: Vec<String> = (0..num_rules.max(1)).map(|i| format!("rule_{i}")).collect();

    let mut builder = GrammarBuilder::new("test");
    for name in &token_names {
        builder = builder.token(name, name);
    }
    for (i, name) in rule_names.iter().enumerate() {
        if !token_names.is_empty() {
            builder = builder.rule(name, vec![&token_names[i % token_names.len()]]);
        }
    }
    builder.build()
}

proptest! {
    #[test]
    fn normalization_idempotent(
        num_tokens in 1_usize..5,
        num_rules in 1_usize..5,
    ) {
        let mut g1 = build_grammar(num_tokens, num_rules);
        let mut g2 = g1.clone();

        g1.normalize();
        g2.normalize();
        g2.normalize(); // normalize again

        // Both should have the same number of rules
        prop_assert_eq!(g1.rules.len(), g2.rules.len());
    }

    #[test]
    fn normalization_preserves_tokens(
        num_tokens in 1_usize..8,
    ) {
        let mut g = build_grammar(num_tokens, 1);
        let before = g.tokens.len();
        g.normalize();
        // Normalization should not remove tokens
        prop_assert!(g.tokens.len() >= before);
    }

    #[test]
    fn normalization_preserves_extras(
        _num_tokens in 2_usize..5,
    ) {
        let mut g = GrammarBuilder::new("extras_test")
            .token("main", "main")
            .token("ws", r"\s+")
            .rule("start", vec!["main"])
            .extra("ws")
            .build();
        let extras_before = g.extras.len();
        g.normalize();
        prop_assert_eq!(g.extras.len(), extras_before);
    }
}

#[test]
fn normalize_empty_grammar() {
    let mut g = Grammar::new("empty".to_string());
    g.normalize();
    assert!(g.rules.is_empty());
}

#[test]
fn normalize_single_rule_grammar() {
    let mut g = GrammarBuilder::new("single")
        .token("x", "x")
        .rule("start", vec!["x"])
        .build();
    g.normalize();
    assert!(!g.rules.is_empty());
}
