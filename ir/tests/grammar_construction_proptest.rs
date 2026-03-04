//! Property-based tests for Grammar construction and invariants.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, PrecedenceKind, Symbol};
use proptest::prelude::*;

/// Strategy for valid lowercase rule names.
fn name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9]{0,5}".prop_map(|s| s)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn grammar_name_preserved(name in "[a-z][a-z0-9]{0,10}") {
        let grammar = GrammarBuilder::new(&name)
            .token("num", "[0-9]+")
            .rule("start", vec!["num"])
            .start("start")
            .build();
        prop_assert_eq!(&grammar.name, &name);
    }

    #[test]
    fn tokens_registered_in_grammar(
        tok_name in name_strategy(),
    ) {
        let grammar = GrammarBuilder::new("test")
            .token(&tok_name, "x")
            .rule("start", vec![&tok_name])
            .start("start")
            .build();

        // Grammar should have at least one token
        prop_assert!(!grammar.tokens.is_empty());
    }

    #[test]
    fn rules_count_matches_added(
        rule_count in 1usize..5,
    ) {
        let mut builder = GrammarBuilder::new("test")
            .token("num", "[0-9]+");

        for i in 0..rule_count {
            let name = format!("alt{}", i);
            builder = builder.token(&name, &format!("t{}", i));
        }

        builder = builder.rule("start", vec!["num"]);
        for i in 0..rule_count {
            let name = format!("alt{}", i);
            builder = builder.rule("start", vec![&name]);
        }

        let grammar = builder.start("start").build();

        let total_rules: usize = grammar.rules.values().map(|v| v.len()).sum();
        prop_assert!(total_rules > rule_count,
            "Expected > {} rules, got {}", rule_count, total_rules);
    }

    #[test]
    fn start_symbol_is_set(
        name in name_strategy(),
    ) {
        let grammar = GrammarBuilder::new("test")
            .token("tok", "x")
            .rule(&name, vec!["tok"])
            .start(&name)
            .build();
        prop_assert!(grammar.start_symbol().is_some());
    }

    #[test]
    fn precedence_value_preserved(
        prec_val in -50i16..50i16,
    ) {
        let grammar = GrammarBuilder::new("test")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .rule_with_precedence(
                "expr",
                vec!["num", "plus", "num"],
                prec_val,
                Associativity::Left,
            )
            .start("expr")
            .build();

        // Find the rule and check precedence
        let all_rules: Vec<_> = grammar.rules.values().flatten().collect();
        let has_prec = all_rules.iter().any(|r| {
            matches!(r.precedence, Some(PrecedenceKind::Static(p)) if p == prec_val)
        });
        prop_assert!(has_prec, "Precedence {} not found in any rule", prec_val);
    }

    #[test]
    fn associativity_preserved(
        assoc_idx in 0u8..3u8,
    ) {
        let assoc = match assoc_idx {
            0 => Associativity::Left,
            1 => Associativity::Right,
            _ => Associativity::None,
        };

        let grammar = GrammarBuilder::new("test")
            .token("num", "[0-9]+")
            .token("op", "\\+")
            .rule_with_precedence("expr", vec!["num", "op", "num"], 1, assoc)
            .start("expr")
            .build();

        let all_rules: Vec<_> = grammar.rules.values().flatten().collect();
        let has_assoc = all_rules.iter().any(|r| {
            r.associativity.as_ref() == Some(&assoc)
        });
        prop_assert!(has_assoc, "Associativity {:?} not found", assoc);
    }

    #[test]
    fn multiple_alternatives_for_same_lhs(
        n_alts in 2usize..6,
    ) {
        let mut builder = GrammarBuilder::new("test");
        for i in 0..n_alts {
            let name = format!("tok{}", i);
            builder = builder.token(&name, &format!("t{}", i));
        }

        for i in 0..n_alts {
            let name = format!("tok{}", i);
            builder = builder.rule("start", vec![&name]);
        }

        let grammar = builder.start("start").build();

        // Find the start rule alternatives
        let start_rules: usize = grammar.rules.values()
            .flatten()
            .filter(|r| grammar.rule_names.get(&r.lhs).is_some_and(|n| n == "start"))
            .count();
        prop_assert!(start_rules >= n_alts,
            "Expected >= {} alternatives, got {}", n_alts, start_rules);
    }

    #[test]
    fn rule_names_registered_for_lowercase(
        name in "[a-z][a-z0-9]{0,5}",
    ) {
        let grammar = GrammarBuilder::new("test")
            .token("tok", "x")
            .rule(&name, vec!["tok"])
            .start(&name)
            .build();

        let has_name = grammar.rule_names.values().any(|n| n == &name);
        prop_assert!(has_name, "Name '{}' not found in rule_names", name);
    }

    #[test]
    fn tokens_have_valid_patterns(
        n_tokens in 1usize..5,
    ) {
        let mut builder = GrammarBuilder::new("test");
        for i in 0..n_tokens {
            let name = format!("tok{}", i);
            builder = builder.token(&name, &format!("[a-z]{}", i + 1));
        }

        builder = builder.rule("start", vec!["tok0"]).start("start");
        let grammar = builder.build();

        for tok in grammar.tokens.values() {
            prop_assert!(!tok.name.is_empty(), "Token name should not be empty");
        }
    }
}

// ── Non-proptest tests ──

#[test]
fn python_like_preset_has_tokens() {
    let grammar = GrammarBuilder::python_like();
    assert!(!grammar.tokens.is_empty());
    assert!(grammar.start_symbol().is_some());
}

#[test]
fn javascript_like_preset_has_tokens() {
    let grammar = GrammarBuilder::javascript_like();
    assert!(!grammar.tokens.is_empty());
    assert!(grammar.start_symbol().is_some());
}

#[test]
fn rule_lhs_rhs_correctly_set() {
    let grammar = GrammarBuilder::new("test")
        .token("num", "[0-9]+")
        .rule("expr", vec!["num"])
        .start("expr")
        .build();

    let all_rules: Vec<_> = grammar.rules.values().flatten().collect();
    assert!(!all_rules.is_empty());
    // At least one rule should have a non-empty RHS
    assert!(all_rules.iter().any(|r| !r.rhs.is_empty()));
}

#[test]
fn epsilon_rule_from_empty_rhs() {
    let grammar = GrammarBuilder::new("test")
        .token("num", "[0-9]+")
        .rule("start", vec![]) // Empty → epsilon
        .rule("start", vec!["num"])
        .start("start")
        .build();

    let start_rules: Vec<_> = grammar.rules.values().flatten().collect();
    // One rule should have Epsilon in its RHS
    let has_epsilon = start_rules
        .iter()
        .any(|r| r.rhs.iter().any(|s| matches!(s, Symbol::Epsilon)));
    assert!(has_epsilon);
}

#[test]
fn fragile_token_is_fragile() {
    let grammar = GrammarBuilder::new("test")
        .fragile_token("semi", ";")
        .token("num", "[0-9]+")
        .rule("start", vec!["num"])
        .start("start")
        .build();

    let fragile_count = grammar.tokens.values().filter(|t| t.fragile).count();
    assert_eq!(fragile_count, 1);
}

#[test]
fn production_ids_unique() {
    let grammar = GrammarBuilder::new("test")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["num", "plus", "num"])
        .start("expr")
        .build();

    let prod_ids: Vec<_> = grammar
        .rules
        .values()
        .flatten()
        .map(|r| r.production_id)
        .collect();
    let mut deduped = prod_ids.clone();
    deduped.sort();
    deduped.dedup();
    assert_eq!(deduped.len(), prod_ids.len());
}
