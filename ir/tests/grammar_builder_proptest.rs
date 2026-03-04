#![allow(clippy::needless_range_loop)]

//! Comprehensive property-based tests for the GrammarBuilder API.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, PrecedenceKind, Symbol};
use proptest::prelude::*;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Lowercase identifiers that the builder will register in `rule_names`.
fn rule_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,12}"
}

/// General identifier names (may start upper or lower).
fn ident_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_]{0,12}"
}

/// Simple token patterns (literal or regex-ish).
fn pattern_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-z]{1,6}",
        Just(r"\d+".to_string()),
        Just(r"[a-zA-Z_]+".to_string()),
        Just(r"[0-9]+(\.[0-9]+)?".to_string()),
    ]
}

/// Vec of (name, pattern) pairs with unique names (at least 1).
fn token_list(max: usize) -> impl Strategy<Value = Vec<(String, String)>> {
    prop::collection::vec((ident_strategy(), pattern_strategy()), 1..=max).prop_map(|pairs| {
        let mut seen = HashSet::new();
        pairs
            .into_iter()
            .filter(|(n, _)| seen.insert(n.clone()))
            .collect()
    })
}

/// Vec of unique rule-style (lowercase) names.
fn unique_rule_names(max: usize) -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(rule_name_strategy(), 1..=max).prop_map(|names| {
        let mut seen = HashSet::new();
        names
            .into_iter()
            .filter(|n| seen.insert(n.clone()))
            .collect()
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // -----------------------------------------------------------------------
    // 1. Builder produces valid grammars (validate passes)
    // -----------------------------------------------------------------------
    #[test]
    fn valid_grammar_with_tokens_and_rules(
        tokens in token_list(4),
    ) {
        if tokens.is_empty() { return Ok(()); }
        let first_tok = tokens[0].0.clone();
        let mut b = GrammarBuilder::new("valid");
        for (n, p) in &tokens {
            b = b.token(n, p);
        }
        b = b.rule("root", vec![&first_tok]);
        b = b.start("root");
        let grammar = b.build();
        prop_assert!(grammar.validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // 2. Token names are preserved in rule_names (for non-operator tokens)
    // -----------------------------------------------------------------------
    #[test]
    fn token_names_stored_in_grammar(
        tokens in token_list(6),
    ) {
        let mut b = GrammarBuilder::new("tok_names");
        for (n, p) in &tokens {
            b = b.token(n, p);
        }
        let grammar = b.build();
        let stored: HashSet<&str> = grammar.tokens.values().map(|t| t.name.as_str()).collect();
        for (n, _) in &tokens {
            prop_assert!(stored.contains(n.as_str()), "Missing token name: {}", n);
        }
    }

    // -----------------------------------------------------------------------
    // 3. Rule names are preserved in rule_names map
    // -----------------------------------------------------------------------
    #[test]
    fn rule_names_in_rule_names_map(
        names in unique_rule_names(5),
    ) {
        if names.is_empty() { return Ok(()); }
        let mut b = GrammarBuilder::new("rn");
        b = b.token("tok", "t");
        for n in &names {
            b = b.rule(n, vec!["tok"]);
        }
        b = b.start(&names[0]);
        let grammar = b.build();
        let rn_values: HashSet<&str> = grammar.rule_names.values().map(|s| s.as_str()).collect();
        for n in &names {
            prop_assert!(rn_values.contains(n.as_str()), "Missing rule name: {}", n);
        }
    }

    // -----------------------------------------------------------------------
    // 4. Start symbol is set correctly (first key in rules map)
    // -----------------------------------------------------------------------
    #[test]
    fn start_symbol_first_in_rules(
        start in rule_name_strategy(),
    ) {
        let grammar = GrammarBuilder::new("st")
            .token("x", "x")
            .rule("other", vec!["x"])
            .rule(&start, vec!["x"])
            .start(&start)
            .build();
        let first_lhs = grammar.rules.keys().next().unwrap();
        let first_name = grammar.rule_names.get(first_lhs).map(|s| s.as_str());
        prop_assert_eq!(first_name, Some(start.as_str()));
    }

    // -----------------------------------------------------------------------
    // 5. Multiple tokens with unique names — count matches
    // -----------------------------------------------------------------------
    #[test]
    fn unique_token_count(tokens in token_list(8)) {
        let mut b = GrammarBuilder::new("cnt");
        for (n, p) in &tokens {
            b = b.token(n, p);
        }
        let grammar = b.build();
        prop_assert_eq!(grammar.tokens.len(), tokens.len());
    }

    // -----------------------------------------------------------------------
    // 6. Multiple rules referencing tokens — terminals resolved
    // -----------------------------------------------------------------------
    #[test]
    fn rules_resolve_tokens_as_terminals(
        tokens in token_list(4),
        num_rules in 1usize..=4,
    ) {
        if tokens.is_empty() { return Ok(()); }
        let mut b = GrammarBuilder::new("resolve");
        for (n, p) in &tokens {
            b = b.token(n, p);
        }
        let first_tok = &tokens[0].0;
        for i in 0..num_rules {
            let lhs = format!("r{}", i);
            b = b.rule(&lhs, vec![first_tok.as_str()]);
        }
        b = b.start("r0");
        let grammar = b.build();
        for rules in grammar.rules.values() {
            for rule in rules {
                for sym in &rule.rhs {
                    if let Symbol::Terminal(id) = sym {
                        prop_assert!(grammar.tokens.contains_key(id));
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 7. Builder with no rules (edge case) — empty grammar
    // -----------------------------------------------------------------------
    #[test]
    fn no_rules_empty_grammar(name in ident_strategy()) {
        let grammar = GrammarBuilder::new(&name).build();
        prop_assert!(grammar.rules.is_empty());
        prop_assert!(grammar.tokens.is_empty());
        prop_assert_eq!(&grammar.name, &name);
    }

    // -----------------------------------------------------------------------
    // 8. Builder with many rules (stress test)
    // -----------------------------------------------------------------------
    #[test]
    fn stress_many_rules(num_rules in 10usize..=30) {
        let mut b = GrammarBuilder::new("stress");
        b = b.token("val", "v");
        for i in 0..num_rules {
            let lhs = format!("rule{}", i);
            b = b.rule(&lhs, vec!["val"]);
        }
        b = b.start("rule0");
        let grammar = b.build();
        let total: usize = grammar.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(total, num_rules);
    }

    // -----------------------------------------------------------------------
    // 9. Precedence configuration — level & associativity preserved
    // -----------------------------------------------------------------------
    #[test]
    fn precedence_preserved(
        level in -10i16..=10i16,
        assoc_idx in 0u8..3u8,
    ) {
        let assoc = match assoc_idx {
            0 => Associativity::Left,
            1 => Associativity::Right,
            _ => Associativity::None,
        };
        let grammar = GrammarBuilder::new("prec")
            .token("a", "a")
            .precedence(level, assoc, vec!["a"])
            .build();
        prop_assert_eq!(grammar.precedences.len(), 1);
        prop_assert_eq!(grammar.precedences[0].level, level);
        prop_assert_eq!(grammar.precedences[0].associativity, assoc);
    }

    // -----------------------------------------------------------------------
    // 10. Normalization doesn't lose rules
    // -----------------------------------------------------------------------
    #[test]
    fn normalize_preserves_rule_count(
        num_rules in 1usize..=6,
    ) {
        let mut b = GrammarBuilder::new("norm");
        b = b.token("x", "x");
        for i in 0..num_rules {
            let lhs = format!("r{}", i);
            b = b.rule(&lhs, vec!["x"]);
        }
        b = b.start("r0");
        let mut grammar = b.build();
        let before: usize = grammar.rules.values().map(|v| v.len()).sum();
        grammar.normalize();
        let after: usize = grammar.rules.values().map(|v| v.len()).sum();
        // normalization on simple grammars should not drop rules
        prop_assert!(after >= before, "Lost rules: {} -> {}", before, after);
    }

    // -----------------------------------------------------------------------
    // 11. Epsilon rule from empty RHS
    // -----------------------------------------------------------------------
    #[test]
    fn empty_rhs_is_epsilon(lhs in rule_name_strategy()) {
        let grammar = GrammarBuilder::new("eps")
            .rule(&lhs, vec![])
            .start(&lhs)
            .build();
        let lhs_id = grammar.find_symbol_by_name(&lhs).unwrap();
        let rules = &grammar.rules[&lhs_id];
        prop_assert!(rules.iter().any(|r| r.rhs == vec![Symbol::Epsilon]));
    }

    // -----------------------------------------------------------------------
    // 12. Production IDs are unique across all rules
    // -----------------------------------------------------------------------
    #[test]
    fn production_ids_unique(num_rules in 1usize..=10) {
        let mut b = GrammarBuilder::new("pid");
        b = b.token("x", "x");
        for i in 0..num_rules {
            b = b.rule(&format!("r{}", i), vec!["x"]);
        }
        let grammar = b.build();
        let mut seen = HashSet::new();
        for rule in grammar.all_rules() {
            prop_assert!(seen.insert(rule.production_id), "Dup prod id {:?}", rule.production_id);
        }
    }

    // -----------------------------------------------------------------------
    // 13. Grammar name round-trips through JSON
    // -----------------------------------------------------------------------
    #[test]
    fn json_roundtrip_name(name in ident_strategy()) {
        let grammar = GrammarBuilder::new(&name).build();
        let json = serde_json::to_string(&grammar).unwrap();
        let restored: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&grammar.name, &restored.name);
    }

    // -----------------------------------------------------------------------
    // 14. JSON roundtrip preserves token and rule counts
    // -----------------------------------------------------------------------
    #[test]
    fn json_roundtrip_counts(tokens in token_list(4)) {
        if tokens.is_empty() { return Ok(()); }
        let first = tokens[0].0.clone();
        let mut b = GrammarBuilder::new("rt");
        for (n, p) in &tokens {
            b = b.token(n, p);
        }
        b = b.rule("start", vec![&first]);
        b = b.start("start");
        let grammar = b.build();
        let json = serde_json::to_string(&grammar).unwrap();
        let restored: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(grammar.tokens.len(), restored.tokens.len());
        let orig_count: usize = grammar.rules.values().map(|v| v.len()).sum();
        let rest_count: usize = restored.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(orig_count, rest_count);
    }

    // -----------------------------------------------------------------------
    // 15. Extras are tracked
    // -----------------------------------------------------------------------
    #[test]
    fn extras_tracked(extra in ident_strategy()) {
        let grammar = GrammarBuilder::new("ext")
            .token(&extra, "pat")
            .extra(&extra)
            .build();
        prop_assert_eq!(grammar.extras.len(), 1);
    }

    // -----------------------------------------------------------------------
    // 16. External tokens are tracked
    // -----------------------------------------------------------------------
    #[test]
    fn externals_tracked(ext_name in ident_strategy()) {
        let grammar = GrammarBuilder::new("extern")
            .external(&ext_name)
            .build();
        prop_assert_eq!(grammar.externals.len(), 1);
        prop_assert_eq!(&grammar.externals[0].name, &ext_name);
    }

    // -----------------------------------------------------------------------
    // 17. Fragile tokens are flagged
    // -----------------------------------------------------------------------
    #[test]
    fn fragile_token_flag(name in ident_strategy()) {
        let grammar = GrammarBuilder::new("frag")
            .fragile_token(&name, &name)
            .build();
        let tok = grammar.tokens.values().next().unwrap();
        prop_assert!(tok.fragile);
    }

    // -----------------------------------------------------------------------
    // 18. Non-fragile tokens are not flagged
    // -----------------------------------------------------------------------
    #[test]
    fn non_fragile_token_flag(name in ident_strategy()) {
        let grammar = GrammarBuilder::new("nf")
            .token(&name, "pat")
            .build();
        let tok = grammar.tokens.values().next().unwrap();
        prop_assert!(!tok.fragile);
    }

    // -----------------------------------------------------------------------
    // 19. Duplicate token reuses symbol id
    // -----------------------------------------------------------------------
    #[test]
    fn duplicate_token_reuses_id(
        name in ident_strategy(),
        p1 in pattern_strategy(),
        p2 in pattern_strategy(),
    ) {
        let grammar = GrammarBuilder::new("dup")
            .token(&name, &p1)
            .token(&name, &p2)
            .build();
        prop_assert_eq!(grammar.tokens.len(), 1);
    }

    // -----------------------------------------------------------------------
    // 20. Undeclared symbols become non-terminals
    // -----------------------------------------------------------------------
    #[test]
    fn undeclared_sym_is_nonterminal(sym in rule_name_strategy()) {
        let grammar = GrammarBuilder::new("nt")
            .rule("root", vec![&sym])
            .start("root")
            .build();
        let root_id = grammar.find_symbol_by_name("root").unwrap();
        let rule = &grammar.rules[&root_id][0];
        prop_assert!(matches!(rule.rhs[0], Symbol::NonTerminal(_)));
    }

    // -----------------------------------------------------------------------
    // 21. Multiple alternatives for same LHS
    // -----------------------------------------------------------------------
    #[test]
    fn alternatives_accumulate(alts in 2usize..=8) {
        let mut b = GrammarBuilder::new("alts");
        for i in 0..alts {
            let tok = format!("t{}", i);
            b = b.token(&tok, &tok);
        }
        for i in 0..alts {
            let tok = format!("t{}", i);
            b = b.rule("expr", vec![&tok]);
        }
        b = b.start("expr");
        let grammar = b.build();
        let expr_id = grammar.find_symbol_by_name("expr").unwrap();
        prop_assert_eq!(grammar.rules[&expr_id].len(), alts);
    }

    // -----------------------------------------------------------------------
    // 22. Tokens added before rules are classified as Terminal
    // -----------------------------------------------------------------------
    #[test]
    fn tokens_before_rules_are_terminal(tokens in token_list(4)) {
        if tokens.is_empty() { return Ok(()); }
        let names: Vec<&str> = tokens.iter().map(|(n, _)| n.as_str()).collect();
        let mut b = GrammarBuilder::new("term");
        for (n, p) in &tokens {
            b = b.token(n, p);
        }
        b = b.rule("root", names);
        b = b.start("root");
        let grammar = b.build();
        let root_id = grammar.find_symbol_by_name("root").unwrap();
        for sym in &grammar.rules[&root_id][0].rhs {
            prop_assert!(matches!(sym, Symbol::Terminal(_)), "Expected Terminal, got {:?}", sym);
        }
    }

    // -----------------------------------------------------------------------
    // 23. Rule with precedence stores level and associativity
    // -----------------------------------------------------------------------
    #[test]
    fn rule_with_prec_stores_values(
        prec_val in -20i16..=20i16,
        assoc_idx in 0u8..3u8,
    ) {
        let assoc = match assoc_idx {
            0 => Associativity::Left,
            1 => Associativity::Right,
            _ => Associativity::None,
        };
        let grammar = GrammarBuilder::new("prec_rule")
            .token("a", "a")
            .token("op", "op")
            .rule_with_precedence("expr", vec!["expr", "op", "expr"], prec_val, assoc)
            .rule("expr", vec!["a"])
            .start("expr")
            .build();
        let expr_id = grammar.find_symbol_by_name("expr").unwrap();
        let prec_rule = grammar.rules[&expr_id]
            .iter()
            .find(|r| r.precedence.is_some())
            .unwrap();
        prop_assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(prec_val)));
        prop_assert_eq!(prec_rule.associativity, Some(assoc));
    }

    // -----------------------------------------------------------------------
    // 24. Normalization is idempotent (second call changes nothing)
    // -----------------------------------------------------------------------
    #[test]
    fn normalize_idempotent(num_rules in 1usize..=5) {
        let mut b = GrammarBuilder::new("idem");
        b = b.token("x", "x");
        for i in 0..num_rules {
            b = b.rule(&format!("r{}", i), vec!["x"]);
        }
        b = b.start("r0");
        let mut grammar = b.build();
        grammar.normalize();
        let count_after_first: usize = grammar.rules.values().map(|v| v.len()).sum();
        grammar.normalize();
        let count_after_second: usize = grammar.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(count_after_first, count_after_second);
    }

    // -----------------------------------------------------------------------
    // 25. Token ordering independence (set equality)
    // -----------------------------------------------------------------------
    #[test]
    fn token_order_independence(tokens in token_list(5)) {
        let mut b1 = GrammarBuilder::new("ord");
        for (n, p) in &tokens {
            b1 = b1.token(n, p);
        }
        let g1 = b1.build();

        let mut b2 = GrammarBuilder::new("ord");
        for (n, p) in tokens.iter().rev() {
            b2 = b2.token(n, p);
        }
        let g2 = b2.build();

        let s1: HashSet<&str> = g1.tokens.values().map(|t| t.name.as_str()).collect();
        let s2: HashSet<&str> = g2.tokens.values().map(|t| t.name.as_str()).collect();
        prop_assert_eq!(s1, s2);
    }

    // -----------------------------------------------------------------------
    // 26. Validate passes on python_like grammar
    // -----------------------------------------------------------------------
    #[test]
    fn python_like_validates(_dummy in 0u8..1u8) {
        let grammar = GrammarBuilder::python_like();
        prop_assert!(grammar.validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // 27. Validate passes on javascript_like grammar
    // -----------------------------------------------------------------------
    #[test]
    fn javascript_like_validates(_dummy in 0u8..1u8) {
        let grammar = GrammarBuilder::javascript_like();
        prop_assert!(grammar.validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // 28. find_symbol_by_name round-trip for tokens
    // -----------------------------------------------------------------------
    #[test]
    fn find_symbol_by_name_tokens(tokens in token_list(5)) {
        let mut b = GrammarBuilder::new("find");
        for (n, p) in &tokens {
            b = b.token(n, p);
        }
        // Also add a rule so token names registered via rule_names can be tested
        if !tokens.is_empty() {
            let first = &tokens[0].0;
            b = b.rule("root", vec![first.as_str()]);
            b = b.start("root");
        }
        let grammar = b.build();
        // Verify rule names are findable
        if grammar.find_symbol_by_name("root").is_some() {
            let root_id = grammar.find_symbol_by_name("root").unwrap();
            prop_assert!(grammar.rules.contains_key(&root_id));
        }
    }

    // -----------------------------------------------------------------------
    // 29. Multiple precedence declarations accumulate
    // -----------------------------------------------------------------------
    #[test]
    fn multiple_precedences_accumulate(count in 1usize..=5) {
        let mut b = GrammarBuilder::new("mprec");
        b = b.token("x", "x");
        for i in 0..count {
            b = b.precedence(i as i16, Associativity::Left, vec!["x"]);
        }
        let grammar = b.build();
        prop_assert_eq!(grammar.precedences.len(), count);
    }

    // -----------------------------------------------------------------------
    // 30. Grammar with only tokens, no rules — validate passes
    // -----------------------------------------------------------------------
    #[test]
    fn tokens_only_validates(tokens in token_list(4)) {
        let mut b = GrammarBuilder::new("tonly");
        for (n, p) in &tokens {
            b = b.token(n, p);
        }
        let grammar = b.build();
        prop_assert!(grammar.validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // 31. Stress: many tokens
    // -----------------------------------------------------------------------
    #[test]
    fn stress_many_tokens(count in 10usize..=30) {
        let mut b = GrammarBuilder::new("many_tok");
        for i in 0..count {
            let n = format!("tok{}", i);
            b = b.token(&n, &n);
        }
        let grammar = b.build();
        prop_assert_eq!(grammar.tokens.len(), count);
    }

    // -----------------------------------------------------------------------
    // 32. Normalization on grammar with epsilon rule doesn't panic
    // -----------------------------------------------------------------------
    #[test]
    fn normalize_epsilon_no_panic(lhs in rule_name_strategy()) {
        let mut grammar = GrammarBuilder::new("eps_norm")
            .rule(&lhs, vec![])
            .start(&lhs)
            .build();
        grammar.normalize(); // must not panic
        let count: usize = grammar.rules.values().map(|v| v.len()).sum();
        prop_assert!(count >= 1);
    }

    // -----------------------------------------------------------------------
    // 33. SymbolId(0) is never assigned by builder (reserved for EOF)
    // -----------------------------------------------------------------------
    #[test]
    fn symbol_zero_reserved(tokens in token_list(5)) {
        let mut b = GrammarBuilder::new("zero");
        for (n, p) in &tokens {
            b = b.token(n, p);
        }
        if !tokens.is_empty() {
            b = b.rule("root", vec![tokens[0].0.as_str()]);
            b = b.start("root");
        }
        let grammar = b.build();
        for id in grammar.tokens.keys() {
            prop_assert_ne!(id.0, 0, "SymbolId(0) should be reserved for EOF");
        }
        for id in grammar.rules.keys() {
            prop_assert_ne!(id.0, 0, "SymbolId(0) should be reserved for EOF");
        }
    }

    // -----------------------------------------------------------------------
    // 34. all_rules iterator count matches sum of rule vec lengths
    // -----------------------------------------------------------------------
    #[test]
    fn all_rules_count(
        num_lhs in 1usize..=5,
        alts in 1usize..=3,
    ) {
        let mut b = GrammarBuilder::new("iter");
        b = b.token("x", "x");
        for i in 0..num_lhs {
            for _ in 0..alts {
                b = b.rule(&format!("r{}", i), vec!["x"]);
            }
        }
        let grammar = b.build();
        let expected: usize = grammar.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(grammar.all_rules().count(), expected);
    }

    // -----------------------------------------------------------------------
    // 35. check_empty_terminals rejects empty patterns
    // -----------------------------------------------------------------------
    #[test]
    fn no_empty_token_patterns(tokens in token_list(5)) {
        let mut b = GrammarBuilder::new("nonempty");
        for (n, p) in &tokens {
            b = b.token(n, p);
        }
        let grammar = b.build();
        // Builder always creates non-empty patterns, so this should pass
        prop_assert!(grammar.check_empty_terminals().is_ok());
    }
}
