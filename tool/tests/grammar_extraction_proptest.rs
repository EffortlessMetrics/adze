//! Property-based tests for the grammar extraction pipeline.
//!
//! Tests invariants of `Grammar`, `GrammarConverter`, `GrammarVisualizer`,
//! and `GrammarValidator` using `proptest`.

use adze_ir::validation::GrammarValidator;
use adze_ir::{
    Associativity, ExternalToken, FieldId, Grammar, PrecedenceKind, ProductionId, Rule, Symbol,
    SymbolId, Token, TokenPattern,
};
use adze_tool::{GrammarConverter, GrammarVisualizer};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate a valid grammar name (non-empty, identifier-like).
fn grammar_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,15}"
}

/// Generate a `SymbolId` in a small range.
fn symbol_id_strategy() -> impl Strategy<Value = SymbolId> {
    (1u16..100).prop_map(SymbolId)
}

/// Generate a `TokenPattern`.
fn token_pattern_strategy() -> impl Strategy<Value = TokenPattern> {
    prop_oneof![
        "[a-zA-Z0-9+\\-*/]{1,10}".prop_map(TokenPattern::String),
        prop_oneof![
            Just(r"[a-z]+".to_string()),
            Just(r"\d+".to_string()),
            Just(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
        ]
        .prop_map(TokenPattern::Regex),
    ]
}

/// Generate a `Token`.
fn token_strategy() -> impl Strategy<Value = (String, TokenPattern, bool)> {
    (
        "[a-z][a-z0-9_]{0,10}",
        token_pattern_strategy(),
        any::<bool>(),
    )
}

/// Generate a leaf `Symbol` (no recursion).
fn leaf_symbol_strategy() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        symbol_id_strategy().prop_map(Symbol::Terminal),
        symbol_id_strategy().prop_map(Symbol::NonTerminal),
        Just(Symbol::Epsilon),
    ]
}

/// Generate an `Associativity`.
fn assoc_strategy() -> impl Strategy<Value = Associativity> {
    prop_oneof![
        Just(Associativity::Left),
        Just(Associativity::Right),
        Just(Associativity::None),
    ]
}

/// Generate a `PrecedenceKind`.
fn prec_kind_strategy() -> impl Strategy<Value = PrecedenceKind> {
    prop_oneof![
        (-10i16..10).prop_map(PrecedenceKind::Static),
        (-10i16..10).prop_map(PrecedenceKind::Dynamic),
    ]
}

/// Build a self-consistent `Grammar` with the given name and token/rule counts.
fn grammar_strategy() -> impl Strategy<Value = Grammar> {
    (
        grammar_name_strategy(),
        prop::collection::vec(token_strategy(), 1..=5),
        prop::collection::vec(
            (
                prop::option::of(prec_kind_strategy()),
                prop::option::of(assoc_strategy()),
            ),
            1..=4,
        ),
    )
        .prop_map(|(name, tokens, rule_attrs)| {
            let mut grammar = Grammar::new(name);

            // Insert tokens with sequential IDs starting at 1.
            let mut token_ids = Vec::new();
            for (i, (tok_name, pattern, fragile)) in tokens.into_iter().enumerate() {
                let id = SymbolId((i + 1) as u16);
                token_ids.push(id);
                grammar.tokens.insert(
                    id,
                    Token {
                        name: tok_name,
                        pattern,
                        fragile,
                    },
                );
            }

            // Create a non-terminal symbol above all tokens.
            let nt_id = SymbolId((token_ids.len() + 1) as u16);
            grammar.rule_names.insert(nt_id, "start".to_string());

            // Add one rule per attribute set, each referencing a random token.
            for (prod_idx, (prec, assoc)) in rule_attrs.into_iter().enumerate() {
                let tok = token_ids[prod_idx % token_ids.len()];
                grammar.rules.entry(nt_id).or_default().push(Rule {
                    lhs: nt_id,
                    rhs: vec![Symbol::Terminal(tok)],
                    precedence: prec,
                    associativity: assoc,
                    fields: vec![],
                    production_id: ProductionId(prod_idx as u16),
                });
            }

            grammar
        })
}

// ---------------------------------------------------------------------------
// 1. Grammar name preserved through new()
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn grammar_name_preserved_through_new(name in grammar_name_strategy()) {
        let g = Grammar::new(name.clone());
        prop_assert_eq!(&g.name, &name);
    }
}

// ---------------------------------------------------------------------------
// 2. add_rule preserves LHS grouping
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn add_rule_groups_by_lhs(lhs_id in 1u16..50, count in 1usize..=6) {
        let mut g = Grammar::new("test".into());
        let lhs = SymbolId(lhs_id);
        // Also register token so validation can pass later
        g.tokens.insert(SymbolId(100), Token {
            name: "tok".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        });
        for i in 0..count {
            g.add_rule(Rule {
                lhs,
                rhs: vec![Symbol::Terminal(SymbolId(100))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(i as u16),
            });
        }
        let rules = g.get_rules_for_symbol(lhs).unwrap();
        prop_assert_eq!(rules.len(), count);
        for r in rules {
            prop_assert_eq!(r.lhs, lhs);
        }
    }
}

// ---------------------------------------------------------------------------
// 3. all_rules yields correct total count
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn all_rules_count_matches(g in grammar_strategy()) {
        let expected: usize = g.rules.values().map(|rs| rs.len()).sum();
        prop_assert_eq!(g.all_rules().count(), expected);
    }
}

// ---------------------------------------------------------------------------
// 4. Token patterns are never empty in generated grammars
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn tokens_have_nonempty_patterns(g in grammar_strategy()) {
        for (_, token) in &g.tokens {
            match &token.pattern {
                TokenPattern::String(s) => prop_assert!(!s.is_empty()),
                TokenPattern::Regex(r) => prop_assert!(!r.is_empty()),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 5. check_empty_terminals accepts valid grammars
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn check_empty_terminals_accepts_valid(g in grammar_strategy()) {
        // Our strategy never generates empty patterns, so this should pass.
        prop_assert!(g.check_empty_terminals().is_ok());
    }
}

// ---------------------------------------------------------------------------
// 6. check_empty_terminals detects empty string tokens
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn check_empty_terminals_rejects_empty_string(name in grammar_name_strategy()) {
        let mut g = Grammar::new(name);
        g.tokens.insert(SymbolId(1), Token {
            name: "empty".into(),
            pattern: TokenPattern::String(String::new()),
            fragile: false,
        });
        prop_assert!(g.check_empty_terminals().is_err());
    }
}

// ---------------------------------------------------------------------------
// 7. check_empty_terminals detects empty regex tokens
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn check_empty_terminals_rejects_empty_regex(name in grammar_name_strategy()) {
        let mut g = Grammar::new(name);
        g.tokens.insert(SymbolId(1), Token {
            name: "empty_re".into(),
            pattern: TokenPattern::Regex(String::new()),
            fragile: false,
        });
        prop_assert!(g.check_empty_terminals().is_err());
    }
}

// ---------------------------------------------------------------------------
// 8. Normalize eliminates Optional symbols
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn normalize_eliminates_optional(tok_id in 1u16..50) {
        let mut g = Grammar::new("norm_test".into());
        let tok = SymbolId(tok_id);
        let nt = SymbolId(tok_id + 100);
        g.tokens.insert(tok, Token {
            name: "t".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        });
        g.add_rule(Rule {
            lhs: nt,
            rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(tok)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        g.normalize();
        // After normalization no rule RHS should contain Optional.
        for rule in g.all_rules() {
            for sym in &rule.rhs {
                prop_assert!(!matches!(sym, Symbol::Optional(_)));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 9. Normalize eliminates Repeat symbols
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn normalize_eliminates_repeat(tok_id in 1u16..50) {
        let mut g = Grammar::new("norm_test".into());
        let tok = SymbolId(tok_id);
        let nt = SymbolId(tok_id + 100);
        g.tokens.insert(tok, Token {
            name: "t".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        });
        g.add_rule(Rule {
            lhs: nt,
            rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(tok)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        g.normalize();
        for rule in g.all_rules() {
            for sym in &rule.rhs {
                prop_assert!(!matches!(sym, Symbol::Repeat(_)));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 10. Normalize eliminates Choice symbols
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn normalize_eliminates_choice(tok_a in 1u16..50, tok_b in 51u16..100) {
        let mut g = Grammar::new("norm_test".into());
        let a = SymbolId(tok_a);
        let b = SymbolId(tok_b);
        let nt = SymbolId(200);
        g.tokens.insert(a, Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        });
        g.tokens.insert(b, Token {
            name: "b".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        });
        g.add_rule(Rule {
            lhs: nt,
            rhs: vec![Symbol::Choice(vec![Symbol::Terminal(a), Symbol::Terminal(b)])],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        g.normalize();
        for rule in g.all_rules() {
            for sym in &rule.rhs {
                prop_assert!(!matches!(sym, Symbol::Choice(_)));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 11. Normalize eliminates Sequence symbols
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn normalize_eliminates_sequence(tok_a in 1u16..50, tok_b in 51u16..100) {
        let mut g = Grammar::new("norm_test".into());
        let a = SymbolId(tok_a);
        let b = SymbolId(tok_b);
        let nt = SymbolId(200);
        g.tokens.insert(a, Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        });
        g.tokens.insert(b, Token {
            name: "b".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        });
        g.add_rule(Rule {
            lhs: nt,
            rhs: vec![Symbol::Sequence(vec![Symbol::Terminal(a), Symbol::Terminal(b)])],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        g.normalize();
        for rule in g.all_rules() {
            for sym in &rule.rhs {
                prop_assert!(!matches!(sym, Symbol::Sequence(_)));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 12. Normalize preserves grammar name
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn normalize_preserves_grammar_name(g in grammar_strategy()) {
        let name = g.name.clone();
        let mut g2 = g;
        g2.normalize();
        prop_assert_eq!(&g2.name, &name);
    }
}

// ---------------------------------------------------------------------------
// 13. Normalize preserves token set
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn normalize_preserves_tokens(g in grammar_strategy()) {
        let token_ids: Vec<_> = g.tokens.keys().copied().collect();
        let mut g2 = g;
        g2.normalize();
        for id in &token_ids {
            prop_assert!(g2.tokens.contains_key(id));
        }
    }
}

// ---------------------------------------------------------------------------
// 14. GrammarConverter sample grammar is self-consistent
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn sample_grammar_has_nonempty_rules(_dummy in 0..1i32) {
        let g = GrammarConverter::create_sample_grammar();
        prop_assert!(g.all_rules().count() > 0);
        prop_assert!(!g.tokens.is_empty());
        prop_assert_eq!(&g.name, "sample");
    }
}

// ---------------------------------------------------------------------------
// 15. GrammarConverter sample grammar passes empty-terminal check
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn sample_grammar_passes_empty_check(_dummy in 0..1i32) {
        let g = GrammarConverter::create_sample_grammar();
        prop_assert!(g.check_empty_terminals().is_ok());
    }
}

// ---------------------------------------------------------------------------
// 16. build_registry produces correct terminal count
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn build_registry_registers_tokens(g in grammar_strategy()) {
        let registry = g.build_registry();
        // Every token should be registered.
        for (_, token) in &g.tokens {
            let id = registry.get_id(&token.name);
            prop_assert!(id.is_some(), "Token '{}' not in registry", token.name);
        }
    }
}

// ---------------------------------------------------------------------------
// 17. build_registry terminals are marked as terminal
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn registry_terminals_flagged(g in grammar_strategy()) {
        let registry = g.build_registry();
        for (_, token) in &g.tokens {
            if let Some(id) = registry.get_id(&token.name) {
                let meta = registry.get_metadata(id).unwrap();
                prop_assert!(meta.terminal,
                    "Token '{}' should be terminal", token.name);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 18. Visualizer DOT output is non-empty for any grammar
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn visualizer_dot_nonempty(g in grammar_strategy()) {
        let viz = GrammarVisualizer::new(g);
        let dot = viz.to_dot();
        prop_assert!(!dot.is_empty());
        prop_assert!(dot.contains("digraph"));
    }
}

// ---------------------------------------------------------------------------
// 19. Visualizer text output is non-empty
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn visualizer_text_nonempty(g in grammar_strategy()) {
        let viz = GrammarVisualizer::new(g);
        let text = viz.to_text();
        prop_assert!(!text.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 20. Visualizer dependency graph is non-empty
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn visualizer_dep_graph_nonempty(g in grammar_strategy()) {
        let viz = GrammarVisualizer::new(g);
        let deps = viz.dependency_graph();
        prop_assert!(!deps.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 21. GrammarValidator stats match grammar shape
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn validator_stats_match(g in grammar_strategy()) {
        let mut validator = GrammarValidator::new();
        let result = validator.validate(&g);
        prop_assert_eq!(result.stats.total_tokens, g.tokens.len());
        prop_assert_eq!(result.stats.total_rules, g.all_rules().count());
    }
}

// ---------------------------------------------------------------------------
// 22. find_symbol_by_name roundtrips with rule_names
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn find_symbol_by_name_roundtrips(g in grammar_strategy()) {
        for (id, name) in &g.rule_names {
            let found = g.find_symbol_by_name(name);
            prop_assert_eq!(found, Some(*id));
        }
    }
}

// ---------------------------------------------------------------------------
// 23. Symbol ordering is deterministic (Ord impl)
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn symbol_ord_deterministic(a in leaf_symbol_strategy(), b in leaf_symbol_strategy()) {
        let cmp1 = a.cmp(&b);
        let cmp2 = a.cmp(&b);
        prop_assert_eq!(cmp1, cmp2);
    }
}

// ---------------------------------------------------------------------------
// 24. Normalize is idempotent (running twice has same effect)
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn normalize_idempotent(g in grammar_strategy()) {
        let mut g1 = g.clone();
        g1.normalize();
        let rules_after_first: Vec<_> = g1.all_rules().cloned().collect();

        let mut g2 = g;
        g2.normalize();
        g2.normalize();
        let rules_after_second: Vec<_> = g2.all_rules().cloned().collect();

        prop_assert_eq!(rules_after_first.len(), rules_after_second.len());
    }
}

// ---------------------------------------------------------------------------
// 25. Externals survive normalize
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn normalize_preserves_externals(
        name in grammar_name_strategy(),
        ext_name in "[a-z]{2,6}",
    ) {
        let mut g = Grammar::new(name);
        g.externals.push(ExternalToken {
            name: ext_name.clone(),
            symbol_id: SymbolId(999),
        });
        // Add a trivial rule so normalize has something to process.
        g.tokens.insert(SymbolId(1), Token {
            name: "t".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        });
        g.add_rule(Rule {
            lhs: SymbolId(2),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        g.normalize();
        prop_assert_eq!(g.externals.len(), 1);
        prop_assert_eq!(&g.externals[0].name, &ext_name);
    }
}

// ---------------------------------------------------------------------------
// 26. Grammar validate rejects dangling symbol references
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn validate_rejects_dangling_refs(bad_id in 200u16..300) {
        let mut g = Grammar::new("dangling".into());
        // Rule references a terminal that does not exist.
        g.add_rule(Rule {
            lhs: SymbolId(1),
            rhs: vec![Symbol::Terminal(SymbolId(bad_id))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        prop_assert!(g.validate().is_err());
    }
}

// ---------------------------------------------------------------------------
// 27. Grammar validate accepts self-consistent grammar
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn validate_accepts_consistent(g in grammar_strategy()) {
        // Our grammar_strategy builds consistent grammars.
        prop_assert!(g.validate().is_ok());
    }
}

// ---------------------------------------------------------------------------
// 28. Fields with lexicographic ordering pass validate
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn validate_accepts_sorted_fields(
        a in "[a-c][a-z]{0,4}",
        b in "[d-f][a-z]{0,4}",
    ) {
        let mut g = Grammar::new("fields_test".into());
        g.tokens.insert(SymbolId(1), Token {
            name: "t".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        });
        g.add_rule(Rule {
            lhs: SymbolId(2),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        // Insert fields in sorted order.
        let mut names = vec![a, b];
        names.sort();
        for (i, name) in names.into_iter().enumerate() {
            g.fields.insert(FieldId(i as u16), name);
        }
        prop_assert!(g.validate().is_ok());
    }
}
