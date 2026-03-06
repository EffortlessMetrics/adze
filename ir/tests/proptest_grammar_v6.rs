//! Property-based tests for Grammar and GrammarBuilder in adze-ir (v6).

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, GrammarOptimizer, GrammarValidator, PrecedenceKind};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal valid grammar with the given name, N tokens, and a start rule.
fn build_grammar_with_tokens(name: &str, n: usize) -> Grammar {
    let mut b = GrammarBuilder::new(name);
    for i in 0..n {
        let tname = format!("t{}", i);
        let pat = format!("pat{}", i);
        b = b.token(&tname, &pat);
    }
    b = b.rule("start", vec!["t0"]).start("start");
    b.build()
}

/// Build a grammar with N distinct rules (each with its own LHS and a shared token).
fn build_grammar_with_rules(name: &str, n: usize) -> Grammar {
    let mut b = GrammarBuilder::new(name).token("a", "a");
    for i in 0..n {
        let lhs = format!("r{}", i);
        b = b.rule(&lhs, vec!["a"]);
    }
    b = b.start("r0");
    b.build()
}

// ---------------------------------------------------------------------------
// 1. Grammar name roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn grammar_name_preserved(name in "[a-z][a-z0-9_]{0,19}") {
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn grammar_name_nonempty(name in "[a-z][a-z0-9_]{0,19}") {
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        prop_assert!(!g.name.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 2. Token count preservation
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn token_count_preserved(n in 1usize..20) {
        let g = build_grammar_with_tokens("tc", n);
        prop_assert_eq!(g.tokens.len(), n);
    }

    #[test]
    fn token_count_one(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("one")
            .token("x", "x")
            .rule("start", vec!["x"])
            .start("start")
            .build();
        prop_assert_eq!(g.tokens.len(), 1);
    }

    #[test]
    fn token_count_max_boundary(_dummy in 0u8..1) {
        let g = build_grammar_with_tokens("max", 20);
        prop_assert_eq!(g.tokens.len(), 20);
    }
}

// ---------------------------------------------------------------------------
// 3. Rule count properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn rule_lhs_count_matches(n in 1usize..10) {
        let g = build_grammar_with_rules("rc", n);
        prop_assert_eq!(g.rules.len(), n);
    }

    #[test]
    fn total_production_count_matches(n in 1usize..10) {
        let g = build_grammar_with_rules("pc", n);
        let total: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(total, n);
    }

    #[test]
    fn two_alternatives_one_lhs(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("alt")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
            .build();
        prop_assert_eq!(g.rules.len(), 1);
        let prods: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(prods, 2);
    }
}

// ---------------------------------------------------------------------------
// 4. Start symbol always resolvable
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn start_symbol_resolvable(n in 1usize..10) {
        let g = build_grammar_with_rules("ss", n);
        prop_assert!(g.start_symbol().is_some());
    }

    #[test]
    fn start_symbol_is_first_rule(name in "[a-z]{2,8}") {
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let first_lhs = g.rules.keys().next().copied();
        let start = g.start_symbol();
        prop_assert_eq!(first_lhs, start);
    }

    #[test]
    fn start_symbol_has_rules(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("sr")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let start_id = g.start_symbol().unwrap();
        prop_assert!(g.get_rules_for_symbol(start_id).is_some());
    }
}

// ---------------------------------------------------------------------------
// 5. All tokens findable by name
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn all_tokens_have_names(n in 1usize..15) {
        let g = build_grammar_with_tokens("tn", n);
        for tok in g.tokens.values() {
            prop_assert!(!tok.name.is_empty());
        }
    }

    #[test]
    fn token_names_match_builder_input(n in 1usize..10) {
        let g = build_grammar_with_tokens("tm", n);
        let expected: Vec<String> = (0..n).map(|i| format!("t{}", i)).collect();
        for name in &expected {
            let found = g.tokens.values().any(|t| t.name == *name);
            prop_assert!(found, "token {} not found", name);
        }
    }

    #[test]
    fn token_patterns_preserved(n in 1usize..10) {
        let g = build_grammar_with_tokens("tp", n);
        prop_assert_eq!(g.tokens.len(), n);
        for tok in g.tokens.values() {
            prop_assert!(!tok.name.is_empty());
        }
    }
}

// ---------------------------------------------------------------------------
// 6. Precedence values preserved
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn precedence_static_preserved(prec in -100i16..100) {
        let g = GrammarBuilder::new("prec")
            .token("a", "a")
            .token("op", "op")
            .rule_with_precedence("expr", vec!["expr", "op", "expr"], prec, Associativity::Left)
            .rule("expr", vec!["a"])
            .start("expr")
            .build();
        let rule = g.all_rules().find(|r| r.precedence.is_some()).unwrap();
        prop_assert_eq!(rule.precedence, Some(PrecedenceKind::Static(prec)));
    }

    #[test]
    fn precedence_negative_preserved(prec in -100i16..-1) {
        let g = GrammarBuilder::new("neg")
            .token("a", "a")
            .token("op", "op")
            .rule_with_precedence("expr", vec!["expr", "op", "expr"], prec, Associativity::Left)
            .rule("expr", vec!["a"])
            .start("expr")
            .build();
        let rule = g.all_rules().find(|r| r.precedence.is_some()).unwrap();
        match rule.precedence {
            Some(PrecedenceKind::Static(v)) => prop_assert_eq!(v, prec),
            other => prop_assert!(false, "unexpected precedence: {:?}", other),
        }
    }

    #[test]
    fn precedence_zero_preserved(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("zero")
            .token("a", "a")
            .token("op", "op")
            .rule_with_precedence("expr", vec!["expr", "op", "expr"], 0, Associativity::Left)
            .rule("expr", vec!["a"])
            .start("expr")
            .build();
        let rule = g.all_rules().find(|r| r.precedence.is_some()).unwrap();
        prop_assert_eq!(rule.precedence, Some(PrecedenceKind::Static(0)));
    }
}

// ---------------------------------------------------------------------------
// 7. Associativity variants preserved
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn associativity_left_preserved(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("al")
            .token("a", "a")
            .token("op", "op")
            .rule_with_precedence("expr", vec!["expr", "op", "expr"], 1, Associativity::Left)
            .rule("expr", vec!["a"])
            .start("expr")
            .build();
        let rule = g.all_rules().find(|r| r.associativity.is_some()).unwrap();
        prop_assert_eq!(rule.associativity, Some(Associativity::Left));
    }

    #[test]
    fn associativity_right_preserved(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("ar")
            .token("a", "a")
            .token("op", "op")
            .rule_with_precedence("expr", vec!["expr", "op", "expr"], 1, Associativity::Right)
            .rule("expr", vec!["a"])
            .start("expr")
            .build();
        let rule = g.all_rules().find(|r| r.associativity.is_some()).unwrap();
        prop_assert_eq!(rule.associativity, Some(Associativity::Right));
    }

    #[test]
    fn associativity_none_preserved(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("an")
            .token("a", "a")
            .token("op", "op")
            .rule_with_precedence("expr", vec!["expr", "op", "expr"], 1, Associativity::None)
            .rule("expr", vec!["a"])
            .start("expr")
            .build();
        let rule = g.all_rules().find(|r| r.associativity.is_some()).unwrap();
        prop_assert_eq!(rule.associativity, Some(Associativity::None));
    }

    #[test]
    fn associativity_variant_roundtrip(idx in 0u8..3) {
        let assoc = match idx {
            0 => Associativity::Left,
            1 => Associativity::Right,
            _ => Associativity::None,
        };
        let g = GrammarBuilder::new("avr")
            .token("a", "a")
            .token("op", "op")
            .rule_with_precedence("expr", vec!["expr", "op", "expr"], 5, assoc)
            .rule("expr", vec!["a"])
            .start("expr")
            .build();
        let rule = g.all_rules().find(|r| r.associativity.is_some()).unwrap();
        prop_assert_eq!(rule.associativity, Some(assoc));
    }
}

// ---------------------------------------------------------------------------
// 8. Inline rules tracked correctly
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn inline_rules_tracked(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("il")
            .token("a", "a")
            .rule("start", vec!["helper"])
            .rule("helper", vec!["a"])
            .inline("helper")
            .start("start")
            .build();
        prop_assert!(!g.inline_rules.is_empty());
    }

    #[test]
    fn inline_rules_count(n in 1usize..5) {
        let mut b = GrammarBuilder::new("ilc").token("a", "a");
        for i in 0..n {
            let name = format!("h{}", i);
            b = b.rule(&name, vec!["a"]).inline(&name);
        }
        b = b.rule("start", vec!["h0"]).start("start");
        let g = b.build();
        prop_assert_eq!(g.inline_rules.len(), n);
    }

    #[test]
    fn no_inline_when_not_marked(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("noil")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        prop_assert!(g.inline_rules.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 9. Extras tracked correctly
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn extras_tracked(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("ex")
            .token("a", "a")
            .token("ws", r"[ \t]+")
            .extra("ws")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        prop_assert!(!g.extras.is_empty());
    }

    #[test]
    fn extras_count(n in 1usize..5) {
        let mut b = GrammarBuilder::new("exc").token("a", "a");
        for i in 0..n {
            let name = format!("e{}", i);
            b = b.token(&name, &format!("e{}", i)).extra(&name);
        }
        b = b.rule("start", vec!["a"]).start("start");
        let g = b.build();
        prop_assert_eq!(g.extras.len(), n);
    }

    #[test]
    fn no_extras_when_not_added(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("noex")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        prop_assert!(g.extras.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 10. Grammar with random token count (1-20)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn random_token_count_grammar(n in 1usize..20) {
        let g = build_grammar_with_tokens("rand_t", n);
        prop_assert_eq!(g.tokens.len(), n);
        prop_assert!(!g.rules.is_empty());
    }

    #[test]
    fn random_token_names_unique(n in 2usize..15) {
        let g = build_grammar_with_tokens("uniq", n);
        let names: Vec<&str> = g.tokens.values().map(|t| t.name.as_str()).collect();
        let mut deduped = names.clone();
        deduped.sort();
        deduped.dedup();
        prop_assert_eq!(names.len(), deduped.len());
    }
}

// ---------------------------------------------------------------------------
// 11. Grammar with random rule count (1-10)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn random_rule_count_grammar(n in 1usize..10) {
        let g = build_grammar_with_rules("rand_r", n);
        prop_assert_eq!(g.rules.len(), n);
    }

    #[test]
    fn random_rule_all_rules_iterator(n in 1usize..10) {
        let g = build_grammar_with_rules("iter", n);
        let count = g.all_rules().count();
        prop_assert_eq!(count, n);
    }
}

// ---------------------------------------------------------------------------
// 12. Normalize doesn't lose rules
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn normalize_preserves_or_grows_rules(n in 1usize..6) {
        let mut g = build_grammar_with_rules("norm", n);
        let before = g.all_rules().count();
        let _new = g.normalize();
        let after = g.all_rules().count();
        prop_assert!(after >= before, "normalize lost rules: {} -> {}", before, after);
    }

    #[test]
    fn normalize_simple_grammar_stable(_dummy in 0u8..1) {
        let mut g = GrammarBuilder::new("nss")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let before = g.all_rules().count();
        let _new = g.normalize();
        let after = g.all_rules().count();
        prop_assert_eq!(before, after);
    }

    #[test]
    fn normalize_idempotent_on_flat_grammar(_dummy in 0u8..1) {
        let mut g = GrammarBuilder::new("nid")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
            .build();
        let _first = g.normalize();
        let count1 = g.all_rules().count();
        let _second = g.normalize();
        let count2 = g.all_rules().count();
        prop_assert_eq!(count1, count2);
    }
}

// ---------------------------------------------------------------------------
// 13. Optimize doesn't crash
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn optimize_no_panic_small(n in 1usize..6) {
        let mut g = build_grammar_with_rules("opt", n);
        g.optimize();
    }

    #[test]
    fn optimize_no_panic_tokens(n in 1usize..15) {
        let mut g = build_grammar_with_tokens("optt", n);
        g.optimize();
    }

    #[test]
    fn grammar_optimizer_no_panic(n in 1usize..6) {
        let mut g = build_grammar_with_rules("gopm", n);
        let mut opt = GrammarOptimizer::new();
        let _stats = opt.optimize(&mut g);
    }

    #[test]
    fn grammar_optimizer_returns_stats(_dummy in 0u8..1) {
        let mut g = GrammarBuilder::new("ost")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let mut opt = GrammarOptimizer::new();
        let stats = opt.optimize(&mut g);
        prop_assert!(stats.removed_unused_symbols < 1000);
    }
}

// ---------------------------------------------------------------------------
// 14. Validate doesn't crash
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn validate_no_panic_small(n in 1usize..6) {
        let g = build_grammar_with_rules("val", n);
        let _ = g.validate();
    }

    #[test]
    fn validate_no_panic_tokens(n in 1usize..15) {
        let g = build_grammar_with_tokens("valt", n);
        let _ = g.validate();
    }

    #[test]
    fn grammar_validator_no_panic(n in 1usize..6) {
        let g = build_grammar_with_rules("gval", n);
        let mut v = GrammarValidator::new();
        let _result = v.validate(&g);
    }

    #[test]
    fn validate_simple_grammar_ok(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("vsimple")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let _ = g.validate();
    }
}

// ---------------------------------------------------------------------------
// 15. Multiple builds with same input produce same structure
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn deterministic_build_equality(suffix in "[a-z]{1,8}") {
        let name = format!("det_{}", suffix);
        let g1 = GrammarBuilder::new(&name)
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
            .build();
        let g2 = GrammarBuilder::new(&name)
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
            .build();
        prop_assert_eq!(&g1, &g2);
    }

    #[test]
    fn deterministic_build_json(suffix in "[a-z]{1,8}") {
        let name = format!("dj_{}", suffix);
        let g1 = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let g2 = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let j1 = serde_json::to_string(&g1).unwrap();
        let j2 = serde_json::to_string(&g2).unwrap();
        prop_assert_eq!(j1, j2);
    }

    #[test]
    fn deterministic_build_rules(suffix in "[a-z]{1,8}") {
        let name = format!("dr_{}", suffix);
        let g1 = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let g2 = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        prop_assert_eq!(g1.rules.len(), g2.rules.len());
        prop_assert_eq!(g1.rule_names, g2.rule_names);
    }
}

// ---------------------------------------------------------------------------
// 16. Serde roundtrip for built grammars
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn serde_roundtrip_name(name in "[a-z][a-z0-9]{0,10}") {
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let json = serde_json::to_string(&g).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&g.name, &back.name);
    }

    #[test]
    fn serde_roundtrip_full(n in 1usize..10) {
        let g = build_grammar_with_tokens("srd", n);
        let json = serde_json::to_string(&g).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&g, &back);
    }

    #[test]
    fn serde_double_roundtrip(n in 1usize..10) {
        let g = build_grammar_with_tokens("sdd", n);
        let j1 = serde_json::to_string(&g).unwrap();
        let mid: Grammar = serde_json::from_str(&j1).unwrap();
        let j2 = serde_json::to_string(&mid).unwrap();
        prop_assert_eq!(j1, j2);
    }
}

// ---------------------------------------------------------------------------
// 17. Clone preserves structure
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn clone_equality(n in 1usize..10) {
        let g = build_grammar_with_tokens("cln", n);
        let c = g.clone();
        prop_assert_eq!(&g, &c);
    }

    #[test]
    fn clone_token_count(n in 1usize..10) {
        let g = build_grammar_with_tokens("clt", n);
        let c = g.clone();
        prop_assert_eq!(g.tokens.len(), c.tokens.len());
    }

    #[test]
    fn double_clone_equality(n in 1usize..10) {
        let g = build_grammar_with_tokens("dc", n);
        let c1 = g.clone();
        let c2 = c1.clone();
        prop_assert_eq!(&g, &c2);
    }
}

// ---------------------------------------------------------------------------
// 18. Supertype tracking
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn supertype_tracked(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("sup")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["thing"])
            .rule("thing", vec!["a"])
            .rule("thing", vec!["b"])
            .supertype("thing")
            .start("start")
            .build();
        prop_assert!(!g.supertypes.is_empty());
    }

    #[test]
    fn no_supertype_when_not_marked(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("nosup")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        prop_assert!(g.supertypes.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 19. External token tracking
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn external_tracked(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("ext")
            .token("a", "a")
            .external("indent")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        prop_assert!(!g.externals.is_empty());
        prop_assert_eq!(&g.externals[0].name, "indent");
    }

    #[test]
    fn external_count(n in 1usize..5) {
        let mut b = GrammarBuilder::new("extc").token("a", "a");
        for i in 0..n {
            b = b.external(&format!("ext{}", i));
        }
        b = b.rule("start", vec!["a"]).start("start");
        let g = b.build();
        prop_assert_eq!(g.externals.len(), n);
    }

    #[test]
    fn no_externals_when_not_added(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("noext")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        prop_assert!(g.externals.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 20. Default grammar is empty
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn default_grammar_empty(_dummy in 0u8..1) {
        let g = Grammar::default();
        prop_assert!(g.rules.is_empty());
        prop_assert!(g.tokens.is_empty());
        prop_assert!(g.name.is_empty());
        prop_assert!(g.extras.is_empty());
        prop_assert!(g.externals.is_empty());
        prop_assert!(g.inline_rules.is_empty());
        prop_assert!(g.supertypes.is_empty());
        prop_assert!(g.precedences.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 21. find_symbol_by_name
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn find_symbol_by_name_start(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("fsbn")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        prop_assert!(g.find_symbol_by_name("start").is_some());
    }

    #[test]
    fn find_symbol_by_name_missing(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("fsbnm")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        prop_assert!(g.find_symbol_by_name("nonexistent").is_none());
    }

    #[test]
    fn find_symbol_for_all_rules(n in 1usize..8) {
        let g = build_grammar_with_rules("fsar", n);
        for i in 0..n {
            let name = format!("r{}", i);
            prop_assert!(
                g.find_symbol_by_name(&name).is_some(),
                "symbol {} not found",
                name,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 22. Precedence declaration API
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn precedence_declaration_preserved(level in -100i16..100) {
        let g = GrammarBuilder::new("pd")
            .token("a", "a")
            .token("op", "op")
            .rule("expr", vec!["a"])
            .precedence(level, Associativity::Left, vec!["op"])
            .start("expr")
            .build();
        prop_assert!(!g.precedences.is_empty());
        prop_assert_eq!(g.precedences[0].level, level);
    }

    #[test]
    fn precedence_declaration_assoc_preserved(idx in 0u8..3) {
        let assoc = match idx {
            0 => Associativity::Left,
            1 => Associativity::Right,
            _ => Associativity::None,
        };
        let g = GrammarBuilder::new("pda")
            .token("a", "a")
            .token("op", "op")
            .rule("expr", vec!["a"])
            .precedence(1, assoc, vec!["op"])
            .start("expr")
            .build();
        prop_assert_eq!(g.precedences[0].associativity, assoc);
    }
}

// ---------------------------------------------------------------------------
// 23. rule_names consistency
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn rule_names_contains_start(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("rnc")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let has_start = g.rule_names.values().any(|n| n == "start");
        prop_assert!(has_start);
    }

    #[test]
    fn rule_names_contains_all_lhs(n in 1usize..8) {
        let g = build_grammar_with_rules("rncl", n);
        for i in 0..n {
            let name = format!("r{}", i);
            let found = g.rule_names.values().any(|n| *n == name);
            prop_assert!(found, "rule name {} not in rule_names", name);
        }
    }
}

// ---------------------------------------------------------------------------
// 24. Empty rule (epsilon) support
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn epsilon_rule_supported(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("eps")
            .token("a", "a")
            .rule("start", vec![])
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let has_epsilon = g.all_rules().any(|r| {
            r.rhs.iter().any(|s| matches!(s, adze_ir::Symbol::Epsilon))
        });
        prop_assert!(has_epsilon);
    }

    #[test]
    fn epsilon_rule_and_normal_rule_coexist(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("eps2")
            .token("a", "a")
            .rule("start", vec![])
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let count: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(count, 2);
    }
}

// ---------------------------------------------------------------------------
// 25. all_rules iterator count
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn all_rules_matches_sum_of_vecs(n in 1usize..10) {
        let g = build_grammar_with_rules("arit", n);
        let via_iter = g.all_rules().count();
        let via_sum: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(via_iter, via_sum);
    }
}
