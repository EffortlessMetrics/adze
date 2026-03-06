//! Property-based tests for GrammarBuilder

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Strategy for valid grammar names (non-empty alphanumeric + underscore).
fn grammar_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,15}".prop_map(|s| s)
}

/// Strategy for valid token names (uppercase identifiers).
fn token_name_strategy() -> impl Strategy<Value = String> {
    "[A-Z][A-Z0-9_]{0,7}".prop_map(|s| s)
}

/// Strategy for simple regex patterns used in tokens.
fn token_pattern_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(r"\d+".to_string()),
        Just(r"[a-z]+".to_string()),
        Just(r"[A-Za-z_][A-Za-z0-9_]*".to_string()),
        Just(r"\w+".to_string()),
        Just(r"[0-9]+".to_string()),
    ]
}

/// Strategy for associativity values.
fn assoc_strategy() -> impl Strategy<Value = Associativity> {
    prop_oneof![
        Just(Associativity::Left),
        Just(Associativity::Right),
        Just(Associativity::None),
    ]
}

// ---------------------------------------------------------------------------
// 1. Builder always produces valid Grammar (8 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_valid_grammar_single_token(
        name in grammar_name_strategy(),
        tname in token_name_strategy(),
        pat in token_pattern_strategy(),
    ) {
        let g = GrammarBuilder::new(&name)
            .token(&tname, &pat)
            .build();
        prop_assert!(!g.name.is_empty());
    }

    #[test]
    fn test_valid_grammar_single_rule(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("TOK", r"\d+")
            .rule("start", vec!["TOK"])
            .start("start")
            .build();
        prop_assert!(!g.rules.is_empty());
    }

    #[test]
    fn test_valid_grammar_empty_rhs(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .rule("empty", vec![])
            .start("empty")
            .build();
        let rules: Vec<&Rule> = g.all_rules().collect();
        prop_assert_eq!(rules.len(), 1);
        prop_assert!(matches!(rules[0].rhs[0], Symbol::Epsilon));
    }

    #[test]
    fn test_valid_grammar_multiple_tokens(
        name in grammar_name_strategy(),
        count in 1usize..=8,
    ) {
        let mut builder = GrammarBuilder::new(&name);
        for i in 0..count {
            let tname = format!("T{i}");
            builder = builder.token(&tname, &format!("pat{i}"));
        }
        let g = builder.build();
        prop_assert_eq!(g.tokens.len(), count);
    }

    #[test]
    fn test_valid_grammar_multiple_rules(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("A", "a")
            .token("B", "b")
            .rule("r1", vec!["A"])
            .rule("r2", vec!["B"])
            .start("r1")
            .build();
        // Two distinct LHS symbols → 2 entries in rules map
        prop_assert_eq!(g.rules.len(), 2);
    }

    #[test]
    fn test_valid_grammar_no_rules_no_tokens(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name).build();
        prop_assert!(g.rules.is_empty());
        prop_assert!(g.tokens.is_empty());
    }

    #[test]
    fn test_valid_grammar_rule_references_token(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("NUM", r"\d+")
            .rule("expr", vec!["NUM"])
            .start("expr")
            .build();
        let rule = g.all_rules().next().unwrap();
        prop_assert!(matches!(rule.rhs[0], Symbol::Terminal(_)));
    }

    #[test]
    fn test_valid_grammar_rule_references_nonterminal(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("NUM", r"\d+")
            .rule("atom", vec!["NUM"])
            .rule("expr", vec!["atom"])
            .start("expr")
            .build();
        // "expr" rule references "atom" which is a non-terminal
        let expr_rules: Vec<_> = g.rules.values().flat_map(|v| v.iter())
            .filter(|r| {
                g.rule_names.get(&r.lhs).map(|n| n == "expr").unwrap_or(false)
            })
            .collect();
        prop_assert_eq!(expr_rules.len(), 1);
        prop_assert!(matches!(expr_rules[0].rhs[0], Symbol::NonTerminal(_)));
    }
}

// ---------------------------------------------------------------------------
// 2. Grammar name preserved (5 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_name_preserved_simple(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name).build();
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn test_name_preserved_with_tokens(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("X", "x")
            .build();
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn test_name_preserved_with_rules(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("X", "x")
            .rule("start", vec!["X"])
            .start("start")
            .build();
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn test_name_preserved_with_precedence(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("A", "a")
            .rule_with_precedence("s", vec!["A"], 1, Associativity::Left)
            .start("s")
            .build();
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn test_name_preserved_with_extras(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("WS", r"\s+")
            .extra("WS")
            .build();
        prop_assert_eq!(&g.name, &name);
    }
}

// ---------------------------------------------------------------------------
// 3. Token count matches tokens added (8 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn test_token_count_zero(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name).build();
        prop_assert_eq!(g.tokens.len(), 0);
    }

    #[test]
    fn test_token_count_one(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("TOK", "tok")
            .build();
        prop_assert_eq!(g.tokens.len(), 1);
    }

    #[test]
    fn test_token_count_many(
        name in grammar_name_strategy(),
        n in 1usize..=10,
    ) {
        let mut b = GrammarBuilder::new(&name);
        for i in 0..n {
            let tname = format!("T{i}");
            b = b.token(&tname, &format!("p{i}"));
        }
        prop_assert_eq!(b.build().tokens.len(), n);
    }

    #[test]
    fn test_token_count_with_rules(
        name in grammar_name_strategy(),
        n in 1usize..=6,
    ) {
        let mut b = GrammarBuilder::new(&name);
        for i in 0..n {
            let tname = format!("T{i}");
            b = b.token(&tname, &format!("p{i}"));
        }
        b = b.rule("start", vec!["T0"]).start("start");
        prop_assert_eq!(b.build().tokens.len(), n);
    }

    #[test]
    fn test_duplicate_token_not_double_counted(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("DUP", "first")
            .token("DUP", "second")
            .build();
        // Same name → same SymbolId, so the token is overwritten not duplicated
        prop_assert_eq!(g.tokens.len(), 1);
    }

    #[test]
    fn test_token_count_with_extras(
        name in grammar_name_strategy(),
        n in 1usize..=5,
    ) {
        let mut b = GrammarBuilder::new(&name);
        for i in 0..n {
            let tname = format!("T{i}");
            b = b.token(&tname, &format!("p{i}"));
        }
        b = b.extra("T0");
        prop_assert_eq!(b.build().tokens.len(), n);
    }

    #[test]
    fn test_token_count_string_vs_regex(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("LIT", "hello")
            .token("RE", r"\d+")
            .build();
        prop_assert_eq!(g.tokens.len(), 2);
    }

    #[test]
    fn test_token_count_stable_across_builds(
        name in grammar_name_strategy(),
        n in 1usize..=8,
    ) {
        let build = || {
            let mut b = GrammarBuilder::new(&name);
            for i in 0..n {
                let tname = format!("T{i}");
                b = b.token(&tname, &format!("p{i}"));
            }
            b.build().tokens.len()
        };
        prop_assert_eq!(build(), build());
    }
}

// ---------------------------------------------------------------------------
// 4. Rule count matches rules added (8 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn test_rule_count_zero(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name).build();
        let total: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(total, 0);
    }

    #[test]
    fn test_rule_count_one(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("A", "a")
            .rule("s", vec!["A"])
            .build();
        let total: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(total, 1);
    }

    #[test]
    fn test_rule_count_multiple_distinct_lhs(
        name in grammar_name_strategy(),
        n in 1usize..=6,
    ) {
        let mut b = GrammarBuilder::new(&name).token("A", "a");
        for i in 0..n {
            let rname = format!("r{i}");
            b = b.rule(&rname, vec!["A"]);
        }
        let g = b.build();
        let total: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(total, n);
        // Each has a unique LHS
        prop_assert_eq!(g.rules.len(), n);
    }

    #[test]
    fn test_rule_count_alternatives_same_lhs(
        name in grammar_name_strategy(),
        n in 1usize..=6,
    ) {
        let mut b = GrammarBuilder::new(&name).token("A", "a").token("B", "b");
        for _ in 0..n {
            b = b.rule("s", vec!["A"]);
        }
        let g = b.build();
        let total: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(total, n);
        // All share same LHS → 1 entry
        prop_assert_eq!(g.rules.len(), 1);
    }

    #[test]
    fn test_rule_count_mixed(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("A", "a")
            .token("B", "b")
            .rule("x", vec!["A"])
            .rule("x", vec!["B"])
            .rule("y", vec!["A"])
            .build();
        let total: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(total, 3);
        prop_assert_eq!(g.rules.len(), 2);
    }

    #[test]
    fn test_rule_count_empty_rhs(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .rule("e", vec![])
            .build();
        let total: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(total, 1);
    }

    #[test]
    fn test_rule_count_with_precedence(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("A", "a")
            .rule_with_precedence("s", vec!["A"], 1, Associativity::Left)
            .rule_with_precedence("s", vec!["A"], 2, Associativity::Right)
            .build();
        let total: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(total, 2);
    }

    #[test]
    fn test_rule_count_each_call_adds_one(
        name in grammar_name_strategy(),
        n in 1usize..=10,
    ) {
        let mut b = GrammarBuilder::new(&name).token("X", "x");
        for _ in 0..n {
            b = b.rule("r", vec!["X"]);
        }
        let total: usize = b.build().rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(total, n);
    }
}

// ---------------------------------------------------------------------------
// 5. Start symbol correctly set (5 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn test_start_symbol_is_first_rule(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("A", "a")
            .token("B", "b")
            .rule("other", vec!["B"])
            .rule("main", vec!["A"])
            .start("main")
            .build();
        // Start symbol's rules should be first in the IndexMap
        let first_lhs = *g.rules.keys().next().unwrap();
        let first_name = g.rule_names.get(&first_lhs).unwrap();
        prop_assert_eq!(first_name, "main");
    }

    #[test]
    fn test_start_symbol_preserves_other_rules(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("A", "a")
            .rule("alpha", vec!["A"])
            .rule("beta", vec!["A"])
            .start("beta")
            .build();
        // Both rules should still be present
        prop_assert_eq!(g.rules.len(), 2);
    }

    #[test]
    fn test_no_start_symbol_keeps_insertion_order(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("A", "a")
            .rule("first", vec!["A"])
            .rule("second", vec!["A"])
            .build();
        let first_lhs = *g.rules.keys().next().unwrap();
        let first_name = g.rule_names.get(&first_lhs).unwrap();
        prop_assert_eq!(first_name, "first");
    }

    #[test]
    fn test_start_can_have_alternatives(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("A", "a")
            .token("B", "b")
            .rule("s", vec!["A"])
            .rule("s", vec!["B"])
            .start("s")
            .build();
        let first_lhs = *g.rules.keys().next().unwrap();
        let alts = &g.rules[&first_lhs];
        prop_assert_eq!(alts.len(), 2);
    }

    #[test]
    fn test_start_empty_rhs(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .rule("nullable", vec![])
            .rule("nullable", vec!["nullable"])
            .start("nullable")
            .build();
        let first_lhs = *g.rules.keys().next().unwrap();
        let alts = &g.rules[&first_lhs];
        prop_assert_eq!(alts.len(), 2);
        prop_assert!(alts.iter().any(|r| matches!(r.rhs[0], Symbol::Epsilon)));
    }
}

// ---------------------------------------------------------------------------
// 6. Builder determinism (5 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_determinism_name(name in grammar_name_strategy()) {
        let g1 = GrammarBuilder::new(&name).build();
        let g2 = GrammarBuilder::new(&name).build();
        prop_assert_eq!(g1.name, g2.name);
    }

    #[test]
    fn test_determinism_tokens(name in grammar_name_strategy()) {
        let build = || {
            GrammarBuilder::new(&name)
                .token("A", "a")
                .token("B", "b")
                .build()
        };
        let g1 = build();
        let g2 = build();
        prop_assert_eq!(g1.tokens.len(), g2.tokens.len());
        for (k, v) in &g1.tokens {
            prop_assert_eq!(g2.tokens.get(k).map(|t| &t.name), Some(&v.name));
        }
    }

    #[test]
    fn test_determinism_rules(name in grammar_name_strategy()) {
        let build = || {
            GrammarBuilder::new(&name)
                .token("A", "a")
                .rule("s", vec!["A"])
                .start("s")
                .build()
        };
        let g1 = build();
        let g2 = build();
        let count = |g: &Grammar| -> usize {
            g.rules.values().map(|v| v.len()).sum()
        };
        prop_assert_eq!(count(&g1), count(&g2));
    }

    #[test]
    fn test_determinism_rule_order(name in grammar_name_strategy()) {
        let build = || {
            GrammarBuilder::new(&name)
                .token("A", "a")
                .token("B", "b")
                .rule("x", vec!["A"])
                .rule("y", vec!["B"])
                .start("x")
                .build()
        };
        let g1 = build();
        let g2 = build();
        let keys1: Vec<_> = g1.rules.keys().collect();
        let keys2: Vec<_> = g2.rules.keys().collect();
        prop_assert_eq!(keys1, keys2);
    }

    #[test]
    fn test_determinism_complex_grammar(name in grammar_name_strategy()) {
        let build = || {
            GrammarBuilder::new(&name)
                .token("NUM", r"\d+")
                .token("PLUS", "+")
                .token("STAR", "*")
                .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
                .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
                .rule("expr", vec!["NUM"])
                .start("expr")
                .build()
        };
        let g1 = build();
        let g2 = build();
        prop_assert_eq!(g1, g2);
    }
}

// ---------------------------------------------------------------------------
// 7. Precedence handling (5 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_precedence_stored_on_rule(
        prec in -100i16..100i16,
        assoc in assoc_strategy(),
    ) {
        let g = GrammarBuilder::new("prec")
            .token("A", "a")
            .rule_with_precedence("s", vec!["A"], prec, assoc)
            .build();
        let rule = g.all_rules().next().unwrap();
        prop_assert_eq!(rule.precedence, Some(PrecedenceKind::Static(prec)));
        prop_assert_eq!(rule.associativity, Some(assoc));
    }

    #[test]
    fn test_precedence_absent_on_plain_rule(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("A", "a")
            .rule("s", vec!["A"])
            .build();
        let rule = g.all_rules().next().unwrap();
        prop_assert_eq!(rule.precedence, None);
        prop_assert_eq!(rule.associativity, None);
    }

    #[test]
    fn test_precedence_multiple_levels(
        lo in -50i16..0i16,
        hi in 1i16..50i16,
    ) {
        let g = GrammarBuilder::new("multi_prec")
            .token("A", "a")
            .token("B", "b")
            .rule_with_precedence("s", vec!["A"], lo, Associativity::Left)
            .rule_with_precedence("s", vec!["B"], hi, Associativity::Right)
            .build();
        let rules: Vec<_> = g.all_rules().collect();
        prop_assert_eq!(rules.len(), 2);
        let precs: Vec<i16> = rules.iter().filter_map(|r| match r.precedence {
            Some(PrecedenceKind::Static(p)) => Some(p),
            _ => None,
        }).collect();
        prop_assert!(precs.contains(&lo));
        prop_assert!(precs.contains(&hi));
    }

    #[test]
    fn test_precedence_mixed_with_plain(
        prec in -100i16..100i16,
    ) {
        let g = GrammarBuilder::new("mixed")
            .token("A", "a")
            .token("B", "b")
            .rule_with_precedence("s", vec!["A"], prec, Associativity::Left)
            .rule("s", vec!["B"])
            .build();
        let rules: Vec<_> = g.all_rules().collect();
        let with_prec = rules.iter().filter(|r| r.precedence.is_some()).count();
        let without_prec = rules.iter().filter(|r| r.precedence.is_none()).count();
        prop_assert_eq!(with_prec, 1);
        prop_assert_eq!(without_prec, 1);
    }

    #[test]
    fn test_precedence_all_associativities(prec in -50i16..50i16) {
        for assoc in [Associativity::Left, Associativity::Right, Associativity::None] {
            let g = GrammarBuilder::new("assoc_test")
                .token("X", "x")
                .rule_with_precedence("s", vec!["X"], prec, assoc)
                .build();
            let rule = g.all_rules().next().unwrap();
            prop_assert_eq!(rule.associativity, Some(assoc));
        }
    }
}

// ---------------------------------------------------------------------------
// 8. Various builder patterns (6 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_pattern_chained_tokens(
        name in grammar_name_strategy(),
        n in 2usize..=6,
    ) {
        let mut b = GrammarBuilder::new(&name);
        for i in 0..n {
            b = b.token(&format!("T{i}"), &format!("t{i}"));
        }
        let g = b.build();
        prop_assert_eq!(g.tokens.len(), n);
    }

    #[test]
    fn test_pattern_token_then_rules(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("A", "a")
            .token("B", "b")
            .rule("s", vec!["A", "B"])
            .start("s")
            .build();
        let rule = g.all_rules().next().unwrap();
        prop_assert_eq!(rule.rhs.len(), 2);
    }

    #[test]
    fn test_pattern_extras(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("WS", r"\s+")
            .extra("WS")
            .build();
        prop_assert_eq!(g.extras.len(), 1);
    }

    #[test]
    fn test_pattern_externals(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("INDENT", "INDENT")
            .external("INDENT")
            .build();
        prop_assert_eq!(g.externals.len(), 1);
    }

    #[test]
    fn test_pattern_fragile_token(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .fragile_token("ERR", "err")
            .build();
        prop_assert_eq!(g.tokens.len(), 1);
        let tok = g.tokens.values().next().unwrap();
        prop_assert!(tok.fragile);
    }

    #[test]
    fn test_pattern_long_rhs(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("A", "a")
            .token("B", "b")
            .token("C", "c")
            .token("D", "d")
            .rule("s", vec!["A", "B", "C", "D"])
            .start("s")
            .build();
        let rule = g.all_rules().next().unwrap();
        prop_assert_eq!(rule.rhs.len(), 4);
    }
}

// ---------------------------------------------------------------------------
// 9. Edge cases (5 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_edge_epsilon_sentinel_content(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .rule("eps", vec![])
            .build();
        let rule = g.all_rules().next().unwrap();
        prop_assert_eq!(rule.rhs.len(), 1);
        prop_assert!(matches!(rule.rhs[0], Symbol::Epsilon));
    }

    #[test]
    fn test_edge_production_ids_unique(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("A", "a")
            .rule("s", vec!["A"])
            .rule("s", vec!["A"])
            .rule("t", vec!["A"])
            .build();
        let ids: Vec<ProductionId> = g.all_rules().map(|r| r.production_id).collect();
        for (i, id) in ids.iter().enumerate() {
            for (j, other) in ids.iter().enumerate() {
                if i != j {
                    prop_assert_ne!(id, other);
                }
            }
        }
    }

    #[test]
    fn test_edge_rule_names_populated(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("A", "a")
            .rule("alpha", vec!["A"])
            .rule("beta", vec!["A"])
            .build();
        let names: Vec<&str> = g.rule_names.values().map(|s| s.as_str()).collect();
        prop_assert!(names.contains(&"alpha"));
        prop_assert!(names.contains(&"beta"));
    }

    #[test]
    fn test_edge_build_consumes_builder(name in grammar_name_strategy()) {
        // This is a compile-time check; at runtime just verify it works
        let builder = GrammarBuilder::new(&name);
        let _g = builder.build();
        prop_assert!(true);
    }

    #[test]
    fn test_edge_same_symbol_as_token_and_rule_ref(name in grammar_name_strategy()) {
        // A token "A" referenced in a rule should resolve to Terminal
        let g = GrammarBuilder::new(&name)
            .token("A", "a")
            .rule("s", vec!["A", "s"])
            .start("s")
            .build();
        let rule = g.all_rules().next().unwrap();
        // First symbol should be Terminal (token "A")
        prop_assert!(matches!(rule.rhs[0], Symbol::Terminal(_)));
        // Second symbol should be NonTerminal (rule "s")
        prop_assert!(matches!(rule.rhs[1], Symbol::NonTerminal(_)));
    }
}
