//! Property-based tests for grammar invariants using proptest.
//!
//! These tests verify that core grammar invariants hold for any grammar
//! constructed through the builder API.

use adze_ir::builder::GrammarBuilder;
use proptest::prelude::*;
use std::collections::HashSet;

// -- Test 1: Building a grammar preserves the name --

proptest! {
    #[test]
    fn grammar_preserves_name(name in "[a-z]{1,10}") {
        let grammar = GrammarBuilder::new(&name)
            .token("TOK", "x")
            .rule("start", vec!["TOK"])
            .start("start")
            .build();
        prop_assert_eq!(grammar.name, name);
    }
}

// -- Test 2: Every token added appears in the grammar's tokens map --

proptest! {
    #[test]
    fn tokens_all_present(count in 1..20usize) {
        let token_names: Vec<String> = (0..count).map(|i| format!("tok{i}")).collect();
        let mut builder = GrammarBuilder::new("g");
        for name in &token_names {
            builder = builder.token(name, name);
        }
        let grammar = builder.build();
        let actual_names: HashSet<&str> = grammar.tokens.values().map(|t| t.name.as_str()).collect();
        for name in &token_names {
            prop_assert!(actual_names.contains(name.as_str()), "missing token: {name}");
        }
    }
}

// -- Test 3: Every rule added appears in the grammar's rules map --

proptest! {
    #[test]
    fn rules_all_present(count in 1..20usize) {
        let mut builder = GrammarBuilder::new("g").token("TOK", "x");
        let mut rule_names = Vec::new();
        for i in 0..count {
            let name = format!("rule{i}");
            builder = builder.rule(&name, vec!["TOK"]);
            rule_names.push(name);
        }
        let grammar = builder.build();
        for name in &rule_names {
            let found = grammar.rule_names.values().any(|n| n == name);
            prop_assert!(found, "missing rule: {name}");
        }
        // Each rule name should have entries in grammar.rules
        for (sym_id, rule_name) in &grammar.rule_names {
            if rule_names.contains(rule_name) {
                prop_assert!(grammar.rules.contains_key(sym_id),
                    "rule_names references {rule_name} at {sym_id:?} but rules map lacks it");
            }
        }
    }
}

// -- Test 4: Start symbol is present when set explicitly --

proptest! {
    #[test]
    fn start_symbol_present(name in "[a-z]{1,10}") {
        let grammar = GrammarBuilder::new("g")
            .token("TOK", "x")
            .rule(&name, vec!["TOK"])
            .start(&name)
            .build();
        // The start rule should be the first entry in rules
        let first_sym = grammar.rules.keys().next().unwrap();
        let first_name = grammar.rule_names.get(first_sym);
        prop_assert_eq!(first_name.map(|s| s.as_str()), Some(name.as_str()));
    }
}

// -- Test 5: Token count equals number of unique token names added --

proptest! {
    #[test]
    fn token_count_matches_unique_names(count in 1..20usize) {
        let mut builder = GrammarBuilder::new("g");
        let mut unique_names = HashSet::new();
        for i in 0..count {
            let name = format!("tok{i}");
            unique_names.insert(name.clone());
            builder = builder.token(&name, &name);
        }
        // Add a duplicate to verify dedup
        builder = builder.token("tok0", "tok0");
        let grammar = builder.build();
        prop_assert_eq!(grammar.tokens.len(), unique_names.len());
    }
}

// -- Test 6: Grammar with extras has non-empty extras field --

proptest! {
    #[test]
    fn extras_non_empty(extra_count in 1..10usize) {
        let mut builder = GrammarBuilder::new("g");
        for i in 0..extra_count {
            let name = format!("extra{i}");
            builder = builder.token(&name, &name).extra(&name);
        }
        let grammar = builder.build();
        prop_assert!(!grammar.extras.is_empty());
        prop_assert_eq!(grammar.extras.len(), extra_count);
    }
}

// -- Test 7: Grammar with externals has non-empty externals field --

proptest! {
    #[test]
    fn externals_non_empty(ext_count in 1..10usize) {
        let mut builder = GrammarBuilder::new("g");
        for i in 0..ext_count {
            let name = format!("ext{i}");
            builder = builder.external(&name);
        }
        let grammar = builder.build();
        prop_assert!(!grammar.externals.is_empty());
        prop_assert_eq!(grammar.externals.len(), ext_count);
    }
}

// -- Test 8: Normalize is idempotent --

proptest! {
    #[test]
    fn normalize_is_idempotent(rule_count in 1..10usize) {
        let mut builder = GrammarBuilder::new("g").token("TOK", "x");
        for i in 0..rule_count {
            builder = builder.rule(&format!("r{i}"), vec!["TOK"]);
        }
        let mut g1 = builder.build();
        g1.normalize();
        let snapshot1 = serde_json::to_string(&g1).unwrap();

        g1.normalize();
        let snapshot2 = serde_json::to_string(&g1).unwrap();

        prop_assert_eq!(snapshot1, snapshot2, "normalize must be idempotent");
    }
}

// -- Test 9: Grammar serializes to valid JSON --

proptest! {
    #[test]
    fn grammar_serializes_to_valid_json(
        name in "[a-z]{1,10}",
        tok_count in 1..10usize,
        rule_count in 1..10usize,
    ) {
        let mut builder = GrammarBuilder::new(&name);
        for i in 0..tok_count {
            builder = builder.token(&format!("tok{i}"), &format!("tok{i}"));
        }
        for i in 0..rule_count {
            builder = builder.rule(&format!("rule{i}"), vec!["tok0"]);
        }
        let grammar = builder.start("rule0").build();
        let json = serde_json::to_string(&grammar);
        prop_assert!(json.is_ok(), "grammar must serialize to JSON");
        // Verify it round-trips
        let parsed: Result<adze_ir::Grammar, _> = serde_json::from_str(&json.unwrap());
        prop_assert!(parsed.is_ok(), "grammar JSON must round-trip");
    }
}
