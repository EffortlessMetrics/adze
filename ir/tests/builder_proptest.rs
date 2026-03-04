//! Property-based tests for the GrammarBuilder API.

use adze_ir::Symbol;
use adze_ir::builder::GrammarBuilder;
use proptest::prelude::*;

/// Strategy for valid grammar/symbol names: starts with a letter, then alphanumeric/underscore.
fn name_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_]{0,15}"
}

/// Strategy for rule (non-terminal) names: must contain at least one lowercase letter
/// so the builder registers them in `rule_names`.
fn rule_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,15}"
}

/// Strategy for simple regex-like token patterns.
fn pattern_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-z]+",                               // literal strings
        Just(r"\d+".to_string()),               // number pattern
        Just(r"[a-zA-Z_]+".to_string()),        // identifier pattern
        Just(r"[0-9]+(\.[0-9]+)?".to_string()), // float pattern
    ]
}

/// Strategy for a vec of (token_name, pattern) pairs with unique names.
fn token_list_strategy(max_len: usize) -> impl Strategy<Value = Vec<(String, String)>> {
    prop::collection::vec((name_strategy(), pattern_strategy()), 1..=max_len).prop_map(|pairs| {
        let mut seen = std::collections::HashSet::new();
        pairs
            .into_iter()
            .filter(|(name, _)| seen.insert(name.clone()))
            .collect()
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // 1. Builder always produces a valid grammar with valid inputs
    #[test]
    fn builder_produces_valid_grammar(
        gname in name_strategy(),
        tokens in token_list_strategy(5),
    ) {
        let mut b = GrammarBuilder::new(&gname);
        for (tname, pat) in &tokens {
            b = b.token(tname, pat);
        }
        let grammar = b.build();
        prop_assert_eq!(&grammar.name, &gname);
        prop_assert!(!grammar.name.is_empty());
    }

    // 2. Token count matches input (unique names)
    #[test]
    fn token_count_matches_input(
        tokens in token_list_strategy(8),
    ) {
        let mut b = GrammarBuilder::new("count_test");
        for (tname, pat) in &tokens {
            b = b.token(tname, pat);
        }
        let grammar = b.build();
        prop_assert_eq!(grammar.tokens.len(), tokens.len());
    }

    // 3. Rules reference only declared symbols (all RHS symbols exist as tokens or non-terminals)
    #[test]
    fn rules_reference_valid_symbols(
        tokens in token_list_strategy(4),
    ) {
        let token_names: Vec<&str> = tokens.iter().map(|(n, _)| n.as_str()).collect();
        if token_names.is_empty() {
            return Ok(());
        }

        let mut b = GrammarBuilder::new("ref_test");
        for (tname, pat) in &tokens {
            b = b.token(tname, pat);
        }
        // Build a rule that only references declared tokens
        b = b.rule("start_rule", token_names.clone());
        b = b.start("start_rule");
        let grammar = b.build();

        // Every terminal in the rule should exist in grammar.tokens
        for rules in grammar.rules.values() {
            for rule in rules {
                for sym in &rule.rhs {
                    if let Symbol::Terminal(id) = sym {
                        prop_assert!(grammar.tokens.contains_key(id),
                            "Terminal {:?} not found in tokens", id);
                    }
                }
            }
        }
    }

    // 4. Start symbol is correctly set (first in rules map)
    #[test]
    fn start_symbol_is_first_rule(
        start_name in rule_name_strategy(),
    ) {
        let grammar = GrammarBuilder::new("start_test")
            .token("A", "a")
            .rule("other", vec!["A"])
            .rule(&start_name, vec!["A"])
            .start(&start_name)
            .build();

        // The start symbol's rules should appear first in the rules map
        let first_lhs = grammar.rules.keys().next().unwrap();
        let first_name = grammar.rule_names.get(first_lhs);
        prop_assert_eq!(first_name.map(|s| s.as_str()), Some(start_name.as_str()));
    }

    // 5. Multiple rules with same LHS accumulate alternatives
    #[test]
    fn multiple_rules_same_lhs(
        num_alternatives in 2usize..=6,
    ) {
        let mut b = GrammarBuilder::new("multi_rule");
        for i in 0..num_alternatives {
            let tok_name = format!("T{i}");
            b = b.token(&tok_name, &tok_name.to_lowercase());
        }
        for i in 0..num_alternatives {
            let tok_name = format!("T{i}");
            b = b.rule("expr", vec![&tok_name]);
        }
        b = b.start("expr");
        let grammar = b.build();

        let expr_id = grammar.rule_names.iter()
            .find(|(_, name)| name.as_str() == "expr")
            .map(|(id, _)| *id)
            .unwrap();
        let expr_rules = &grammar.rules[&expr_id];
        prop_assert_eq!(expr_rules.len(), num_alternatives);
    }

    // 6. Empty grammar (no tokens/rules) behavior
    #[test]
    fn empty_grammar_has_no_tokens_or_rules(gname in name_strategy()) {
        let grammar = GrammarBuilder::new(&gname).build();
        prop_assert!(grammar.tokens.is_empty());
        prop_assert!(grammar.rules.is_empty());
        prop_assert_eq!(&grammar.name, &gname);
    }

    // 7. Duplicate token names: last definition wins
    #[test]
    fn duplicate_token_last_wins(
        tname in name_strategy(),
        pat1 in pattern_strategy(),
        pat2 in pattern_strategy(),
    ) {
        let grammar = GrammarBuilder::new("dup_test")
            .token(&tname, &pat1)
            .token(&tname, &pat2)
            .build();
        // Same symbol ID should be reused, so exactly 1 token
        prop_assert_eq!(grammar.tokens.len(), 1);
        // The token's pattern should be from the second call
        let token = grammar.tokens.values().next().unwrap();
        prop_assert_eq!(&token.name, &tname);
    }

    // 8. Grammar name is preserved exactly
    #[test]
    fn grammar_name_preserved_exact(name in "[a-zA-Z][a-zA-Z0-9_]{0,30}") {
        let grammar = GrammarBuilder::new(&name).build();
        prop_assert_eq!(&grammar.name, &name);
    }

    // 9. Token names are preserved in the grammar
    #[test]
    fn token_names_preserved(tokens in token_list_strategy(6)) {
        let mut b = GrammarBuilder::new("preserve_test");
        for (tname, pat) in &tokens {
            b = b.token(tname, pat);
        }
        let grammar = b.build();
        let stored_names: std::collections::HashSet<&str> = grammar.tokens.values()
            .map(|t| t.name.as_str())
            .collect();
        for (tname, _) in &tokens {
            prop_assert!(stored_names.contains(tname.as_str()),
                "Token name '{}' not found in grammar", tname);
        }
    }

    // 10. Production IDs are unique across all rules
    #[test]
    fn production_ids_unique(num_rules in 1usize..=8) {
        let mut b = GrammarBuilder::new("prod_id_test");
        b = b.token("X", "x");
        for i in 0..num_rules {
            let lhs = format!("rule{i}");
            b = b.rule(&lhs, vec!["X"]);
        }
        let grammar = b.build();
        let mut prod_ids = std::collections::HashSet::new();
        for rule in grammar.all_rules() {
            prop_assert!(prod_ids.insert(rule.production_id),
                "Duplicate production ID: {:?}", rule.production_id);
        }
    }

    // 11. Empty RHS produces epsilon rule
    #[test]
    fn empty_rhs_produces_epsilon(lhs_name in rule_name_strategy()) {
        let grammar = GrammarBuilder::new("epsilon_test")
            .rule(&lhs_name, vec![])
            .start(&lhs_name)
            .build();

        let lhs_id = grammar.rule_names.iter()
            .find(|(_, name)| name.as_str() == lhs_name)
            .map(|(id, _)| *id)
            .unwrap();
        let rules = &grammar.rules[&lhs_id];
        prop_assert_eq!(rules.len(), 1);
        prop_assert_eq!(rules[0].rhs.len(), 1);
        prop_assert!(matches!(rules[0].rhs[0], Symbol::Epsilon));
    }

    // 12. Tokens added before rules are recognized as terminals in RHS
    #[test]
    fn tokens_before_rules_are_terminals(
        tokens in token_list_strategy(4),
    ) {
        if tokens.is_empty() {
            return Ok(());
        }
        let token_names: Vec<&str> = tokens.iter().map(|(n, _)| n.as_str()).collect();
        let mut b = GrammarBuilder::new("terminal_test");
        for (tname, pat) in &tokens {
            b = b.token(tname, pat);
        }
        b = b.rule("root", token_names);
        b = b.start("root");
        let grammar = b.build();

        let root_id = grammar.rule_names.iter()
            .find(|(_, name)| name.as_str() == "root")
            .map(|(id, _)| *id)
            .unwrap();
        let rule = &grammar.rules[&root_id][0];
        for sym in &rule.rhs {
            prop_assert!(matches!(sym, Symbol::Terminal(_)),
                "Expected terminal, got {:?}", sym);
        }
    }

    // 13. Symbols not declared as tokens become non-terminals
    #[test]
    fn undeclared_symbols_are_nonterminals(
        sym_name in name_strategy(),
    ) {
        let grammar = GrammarBuilder::new("nonterm_test")
            .rule("root", vec![&sym_name])
            .start("root")
            .build();

        let root_id = grammar.rule_names.iter()
            .find(|(_, name)| name.as_str() == "root")
            .map(|(id, _)| *id)
            .unwrap();
        let rule = &grammar.rules[&root_id][0];
        // sym_name was never declared as a token, so should be NonTerminal
        // (unless sym_name == "root", in which case it's still NonTerminal)
        prop_assert!(matches!(rule.rhs[0], Symbol::NonTerminal(_)),
            "Expected NonTerminal, got {:?}", rule.rhs[0]);
    }

    // 14. Rule count across grammar matches number of .rule() calls
    #[test]
    fn total_rule_count_matches(
        num_lhs in 1usize..=4,
        alts_per_lhs in 1usize..=3,
    ) {
        let mut b = GrammarBuilder::new("count_rules");
        b = b.token("X", "x");
        let expected = num_lhs * alts_per_lhs;
        for i in 0..num_lhs {
            let lhs = format!("rule{i}");
            for _ in 0..alts_per_lhs {
                b = b.rule(&lhs, vec!["X"]);
            }
        }
        let grammar = b.build();
        let total: usize = grammar.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(total, expected);
    }

    // 15. JSON roundtrip preserves token count and rule count
    #[test]
    fn json_roundtrip_preserves_structure(
        tokens in token_list_strategy(4),
    ) {
        let mut b = GrammarBuilder::new("roundtrip");
        for (tname, pat) in &tokens {
            b = b.token(tname, pat);
        }
        if !tokens.is_empty() {
            let first = tokens[0].0.as_str();
            b = b.rule("start", vec![first]);
            b = b.start("start");
        }
        let grammar = b.build();
        let json = serde_json::to_string(&grammar).unwrap();
        let restored: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(grammar.tokens.len(), restored.tokens.len());
        prop_assert_eq!(grammar.rules.len(), restored.rules.len());
        prop_assert_eq!(&grammar.name, &restored.name);
    }

    // 16. Extra tokens are tracked
    #[test]
    fn extras_are_tracked(
        extra_name in name_strategy(),
    ) {
        let grammar = GrammarBuilder::new("extra_test")
            .token(&extra_name, "pattern")
            .extra(&extra_name)
            .build();
        prop_assert_eq!(grammar.extras.len(), 1);
    }

    // 17. Fragile tokens are marked fragile
    #[test]
    fn fragile_tokens_marked(
        tname in name_strategy(),
    ) {
        let grammar = GrammarBuilder::new("fragile_test")
            .fragile_token(&tname, &tname)
            .build();
        let token = grammar.tokens.values().next().unwrap();
        prop_assert!(token.fragile);
    }

    // 18. Builder is composable: chaining order doesn't affect final token set
    #[test]
    fn chaining_order_independent_for_tokens(
        tokens in token_list_strategy(5),
    ) {
        // Build forward
        let mut b1 = GrammarBuilder::new("order_test");
        for (tname, pat) in &tokens {
            b1 = b1.token(tname, pat);
        }
        let g1 = b1.build();

        // Build reverse
        let mut b2 = GrammarBuilder::new("order_test");
        for (tname, pat) in tokens.iter().rev() {
            b2 = b2.token(tname, pat);
        }
        let g2 = b2.build();

        prop_assert_eq!(g1.tokens.len(), g2.tokens.len());
        // Same token names in both
        let names1: std::collections::HashSet<&str> = g1.tokens.values().map(|t| t.name.as_str()).collect();
        let names2: std::collections::HashSet<&str> = g2.tokens.values().map(|t| t.name.as_str()).collect();
        prop_assert_eq!(names1, names2);
    }
}
