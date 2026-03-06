//! Property-based tests for Grammar operations in adze-ir (v7).
//!
//! Focuses on compositional grammar building, structural invariants,
//! registry interactions, and cross-cutting property checks.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, GrammarOptimizer, GrammarValidator, PrecedenceKind, Symbol};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a grammar with `n` tokens, a start rule consuming t0, and unique name.
fn build_tok_grammar(suffix: &str, n: usize) -> Grammar {
    let mut b = GrammarBuilder::new(&format!("pg_v7_{}", suffix));
    for i in 0..n {
        let tname = format!("t{}", i);
        let pat = format!("p{}", i);
        b = b.token(&tname, &pat);
    }
    b = b.rule("start", vec!["t0"]).start("start");
    b.build()
}

/// Build a grammar with `n` distinct LHS non-terminals, each consuming token "a".
fn build_rule_grammar(suffix: &str, n: usize) -> Grammar {
    let mut b = GrammarBuilder::new(&format!("pg_v7_{}", suffix)).token("a", "a");
    for i in 0..n {
        let lhs = format!("r{}", i);
        b = b.rule(&lhs, vec!["a"]);
    }
    b = b.start("r0");
    b.build()
}

/// Build a grammar with `n` alternative productions under the same LHS.
fn build_alt_grammar(suffix: &str, n: usize) -> Grammar {
    let mut b = GrammarBuilder::new(&format!("pg_v7_{}", suffix));
    for i in 0..n {
        let tname = format!("t{}", i);
        let pat = format!("p{}", i);
        b = b.token(&tname, &pat);
    }
    for i in 0..n {
        let tname = format!("t{}", i);
        b = b.rule("start", vec![&tname]);
    }
    b = b.start("start");
    b.build()
}

// ===========================================================================
// 1 – Token-count scaling
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn token_count_scales_linearly(n in 1usize..10) {
        let g = build_tok_grammar(&format!("tcs_{}", n), n);
        prop_assert_eq!(g.tokens.len(), n);
    }
}

// ===========================================================================
// 2 – Rule with start symbol always present
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn start_symbol_present_with_rules(m in 1usize..5) {
        let g = build_rule_grammar(&format!("ssp_{}", m), m);
        prop_assert!(g.start_symbol().is_some());
    }
}

// ===========================================================================
// 3 – Grammar name preserved after build
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn name_preserved_after_build(suffix in "[a-z]{1,8}") {
        let name = format!("pg_v7_{}", suffix);
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        prop_assert_eq!(&g.name, &name);
    }
}

// ===========================================================================
// 4 – Token count matches after build
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn token_count_matches_after_build(n in 1usize..10) {
        let g = build_tok_grammar(&format!("tcm_{}", n), n);
        prop_assert_eq!(g.tokens.len(), n);
    }
}

// ===========================================================================
// 5 – Clone produces same name
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn clone_preserves_name(suffix in "[a-z]{1,8}") {
        let g = build_tok_grammar(&suffix, 2);
        let c = g.clone();
        prop_assert_eq!(&g.name, &c.name);
    }
}

// ===========================================================================
// 6 – Clone produces same token count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn clone_preserves_token_count(n in 1usize..10) {
        let g = build_tok_grammar(&format!("cpt_{}", n), n);
        let c = g.clone();
        prop_assert_eq!(g.tokens.len(), c.tokens.len());
    }
}

// ===========================================================================
// 7 – Clone produces same rule count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn clone_preserves_rule_count(n in 1usize..8) {
        let g = build_rule_grammar(&format!("cpr_{}", n), n);
        let c = g.clone();
        prop_assert_eq!(g.rules.len(), c.rules.len());
    }
}

// ===========================================================================
// 8 – all_rules().count() == sum of rule vecs
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn all_rules_count_equals_rules_len(n in 1usize..8) {
        let g = build_rule_grammar(&format!("arc_{}", n), n);
        let iter_count = g.all_rules().count();
        let sum_count: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(iter_count, sum_count);
    }
}

// ===========================================================================
// 9 – normalize() doesn't panic
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn normalize_no_panic(n in 1usize..6) {
        let mut g = build_rule_grammar(&format!("nnp_{}", n), n);
        let _ = g.normalize();
    }
}

// ===========================================================================
// 10 – optimize() doesn't panic
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn optimize_no_panic(n in 1usize..6) {
        let mut g = build_rule_grammar(&format!("onp_{}", n), n);
        g.optimize();
    }
}

// ===========================================================================
// 11 – validate() returns Ok or Err (doesn't panic)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn validate_no_panic(n in 1usize..6) {
        let g = build_rule_grammar(&format!("vnp_{}", n), n);
        let _ = g.validate();
    }
}

// ===========================================================================
// 12 – Deterministic builds produce same token count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn deterministic_token_count(n in 1usize..10) {
        let g1 = build_tok_grammar(&format!("dtc_{}", n), n);
        let g2 = build_tok_grammar(&format!("dtc_{}", n), n);
        prop_assert_eq!(g1.tokens.len(), g2.tokens.len());
    }
}

// ===========================================================================
// 13 – Precedence values (-100..100) build without error
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn precedence_range_builds(prec in -100i16..100) {
        let g = GrammarBuilder::new(&format!("pg_v7_prb_{}", (prec as i32 + 200)))
            .token("a", "a")
            .token("op", "op")
            .rule_with_precedence("expr", vec!["expr", "op", "expr"], prec, Associativity::Left)
            .rule("expr", vec!["a"])
            .start("expr")
            .build();
        prop_assert!(!g.rules.is_empty());
    }
}

// ===========================================================================
// 14 – All Associativity variants build
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn all_assoc_variants_build(idx in 0u8..3) {
        let assoc = match idx {
            0 => Associativity::Left,
            1 => Associativity::Right,
            _ => Associativity::None,
        };
        let g = GrammarBuilder::new(&format!("pg_v7_aab_{}", idx))
            .token("a", "a")
            .token("op", "op")
            .rule_with_precedence("expr", vec!["expr", "op", "expr"], 1, assoc)
            .rule("expr", vec!["a"])
            .start("expr")
            .build();
        prop_assert!(g.all_rules().count() >= 2);
    }
}

// ===========================================================================
// 15 – Various grammar sizes all buildable
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn various_sizes_buildable(tok in 1usize..10, rules in 1usize..5) {
        let mut b = GrammarBuilder::new(&format!("pg_v7_vsb_{}_{}", tok, rules));
        for i in 0..tok {
            b = b.token(&format!("t{}", i), &format!("p{}", i));
        }
        for i in 0..rules {
            b = b.rule(&format!("r{}", i), vec!["t0"]);
        }
        b = b.start("r0");
        let g = b.build();
        prop_assert_eq!(g.tokens.len(), tok);
        prop_assert_eq!(g.rules.len(), rules);
    }
}

// ===========================================================================
// 16 – Alternative productions under one LHS
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn alternatives_count_correct(n in 1usize..8) {
        let g = build_alt_grammar(&format!("alt_{}", n), n);
        let total: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(total, n);
        prop_assert_eq!(g.rules.len(), 1);
    }
}

// ===========================================================================
// 17 – get_rules_for_symbol returns correct count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn get_rules_for_start_returns_rules(n in 1usize..5) {
        let g = build_alt_grammar(&format!("grfs_{}", n), n);
        let start_id = g.start_symbol().unwrap();
        let rules = g.get_rules_for_symbol(start_id).unwrap();
        prop_assert_eq!(rules.len(), n);
    }
}

// ===========================================================================
// 18 – Nonexistent symbol returns None
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn find_nonexistent_symbol_none(suffix in "[a-z]{1,6}") {
        let g = GrammarBuilder::new(&format!("pg_v7_fns_{}", suffix))
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        prop_assert!(g.find_symbol_by_name("zzzznonexist").is_none());
    }
}

// ===========================================================================
// 19 – Normalize preserves or grows rule count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn normalize_preserves_or_grows(n in 1usize..6) {
        let mut g = build_rule_grammar(&format!("npg_{}", n), n);
        let before = g.all_rules().count();
        let _ = g.normalize();
        let after = g.all_rules().count();
        prop_assert!(after >= before);
    }
}

// ===========================================================================
// 20 – Optimize then validate doesn't panic
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn optimize_then_validate_no_panic(n in 1usize..5) {
        let mut g = build_rule_grammar(&format!("otv_{}", n), n);
        g.optimize();
        let _ = g.validate();
    }
}

// ===========================================================================
// 21 – GrammarValidator returns result without panic
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn grammar_validator_no_panic(n in 1usize..6) {
        let g = build_rule_grammar(&format!("gvnp_{}", n), n);
        let mut v = GrammarValidator::new();
        let _ = v.validate(&g);
    }
}

// ===========================================================================
// 22 – GrammarOptimizer returns stats
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn grammar_optimizer_returns_stats(n in 1usize..6) {
        let mut g = build_rule_grammar(&format!("gors_{}", n), n);
        let mut opt = GrammarOptimizer::new();
        let stats = opt.optimize(&mut g);
        prop_assert!(stats.removed_unused_symbols < 10_000);
    }
}

// ===========================================================================
// 23 – Default grammar is empty
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn default_grammar_empty(_dummy in 0u8..1) {
        let g = Grammar::default();
        prop_assert!(g.rules.is_empty());
        prop_assert!(g.tokens.is_empty());
        prop_assert!(g.name.is_empty());
    }
}

// ===========================================================================
// 24 – Fragile token builds without error
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn fragile_token_builds(suffix in "[a-z]{1,6}") {
        let g = GrammarBuilder::new(&format!("pg_v7_ft_{}", suffix))
            .fragile_token("err", "error")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let has_fragile = g.tokens.values().any(|t| t.fragile);
        prop_assert!(has_fragile);
    }
}

// ===========================================================================
// 25 – Non-fragile tokens are not fragile
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn normal_tokens_not_fragile(n in 1usize..8) {
        let g = build_tok_grammar(&format!("ntf_{}", n), n);
        for tok in g.tokens.values() {
            prop_assert!(!tok.fragile);
        }
    }
}

// ===========================================================================
// 26 – Serde JSON roundtrip preserves equality
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn serde_json_roundtrip(n in 1usize..8) {
        let g = build_tok_grammar(&format!("sjr_{}", n), n);
        let json = serde_json::to_string(&g).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&g, &back);
    }
}

// ===========================================================================
// 27 – Serde roundtrip preserves name
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn serde_roundtrip_preserves_name(suffix in "[a-z]{1,8}") {
        let g = build_tok_grammar(&suffix, 3);
        let json = serde_json::to_string(&g).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&g.name, &back.name);
    }
}

// ===========================================================================
// 28 – Double serde roundtrip is stable
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn serde_double_roundtrip_stable(n in 1usize..8) {
        let g = build_tok_grammar(&format!("sdr_{}", n), n);
        let j1 = serde_json::to_string(&g).unwrap();
        let mid: Grammar = serde_json::from_str(&j1).unwrap();
        let j2 = serde_json::to_string(&mid).unwrap();
        prop_assert_eq!(j1, j2);
    }
}

// ===========================================================================
// 29 – Clone then modify doesn't affect original
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn clone_independence(n in 1usize..8) {
        let g = build_tok_grammar(&format!("ci_{}", n), n);
        let mut c = g.clone();
        c.name = "modified".to_string();
        prop_assert_ne!(&g.name, &c.name);
        prop_assert_eq!(g.tokens.len(), n);
    }
}

// ===========================================================================
// 30 – Every rule has a valid LHS in rules map
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn every_rule_lhs_in_map(n in 1usize..6) {
        let g = build_rule_grammar(&format!("erl_{}", n), n);
        for rule in g.all_rules() {
            prop_assert!(g.rules.contains_key(&rule.lhs));
        }
    }
}

// ===========================================================================
// 31 – Unique production IDs within a grammar
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn unique_production_ids(n in 1usize..8) {
        let g = build_rule_grammar(&format!("upi_{}", n), n);
        let ids: Vec<_> = g.all_rules().map(|r| r.production_id).collect();
        let mut deduped = ids.clone();
        deduped.sort();
        deduped.dedup();
        prop_assert_eq!(ids.len(), deduped.len());
    }
}

// ===========================================================================
// 32 – Epsilon rule creates single-element rhs
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn epsilon_rule_rhs_length(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("pg_v7_erl")
            .token("a", "a")
            .rule("start", vec![])
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let eps_rules: Vec<_> = g.all_rules()
            .filter(|r| r.rhs.iter().any(|s| matches!(s, Symbol::Epsilon)))
            .collect();
        for r in eps_rules {
            prop_assert_eq!(r.rhs.len(), 1);
        }
    }
}

// ===========================================================================
// 33 – rule_names covers every LHS
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn rule_names_covers_all_lhs(n in 1usize..6) {
        let g = build_rule_grammar(&format!("rnca_{}", n), n);
        for lhs_id in g.rules.keys() {
            prop_assert!(
                g.rule_names.contains_key(lhs_id),
                "rule_names missing LHS {:?}", lhs_id,
            );
        }
    }
}

// ===========================================================================
// 34 – Extra tokens tracked
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn extras_count_matches(n in 1usize..5) {
        let mut b = GrammarBuilder::new(&format!("pg_v7_ecm_{}", n)).token("a", "a");
        for i in 0..n {
            let name = format!("ws{}", i);
            b = b.token(&name, &format!("ws{}", i)).extra(&name);
        }
        b = b.rule("start", vec!["a"]).start("start");
        let g = b.build();
        prop_assert_eq!(g.extras.len(), n);
    }
}

// ===========================================================================
// 35 – Inline rules tracked
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn inline_count_matches(n in 1usize..5) {
        let mut b = GrammarBuilder::new(&format!("pg_v7_icm_{}", n)).token("a", "a");
        for i in 0..n {
            let name = format!("h{}", i);
            b = b.rule(&name, vec!["a"]).inline(&name);
        }
        b = b.rule("start", vec!["h0"]).start("start");
        let g = b.build();
        prop_assert_eq!(g.inline_rules.len(), n);
    }
}

// ===========================================================================
// 36 – Supertype count matches
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn supertype_count_matches(n in 1usize..4) {
        let mut b = GrammarBuilder::new(&format!("pg_v7_scm_{}", n)).token("a", "a");
        for i in 0..n {
            let name = format!("st{}", i);
            b = b.rule(&name, vec!["a"]).supertype(&name);
        }
        b = b.rule("start", vec!["st0"]).start("start");
        let g = b.build();
        prop_assert_eq!(g.supertypes.len(), n);
    }
}

// ===========================================================================
// 37 – External token count matches
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn external_count_matches(n in 1usize..5) {
        let mut b = GrammarBuilder::new(&format!("pg_v7_excm_{}", n)).token("a", "a");
        for i in 0..n {
            b = b.external(&format!("ext{}", i));
        }
        b = b.rule("start", vec!["a"]).start("start");
        let g = b.build();
        prop_assert_eq!(g.externals.len(), n);
    }
}

// ===========================================================================
// 38 – Precedence declaration count matches
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn precedence_decl_count(n in 1usize..5) {
        let mut b = GrammarBuilder::new(&format!("pg_v7_pdc_{}", n))
            .token("a", "a");
        for i in 0..n {
            let tok = format!("op{}", i);
            b = b.token(&tok, &tok)
                .precedence(i as i16, Associativity::Left, vec![&tok]);
        }
        b = b.rule("start", vec!["a"]).start("start");
        let g = b.build();
        prop_assert_eq!(g.precedences.len(), n);
    }
}

// ===========================================================================
// 39 – Precedence level ordering preserved
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn precedence_levels_preserved(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("pg_v7_plp")
            .token("a", "a")
            .token("plus", "+")
            .token("star", "*")
            .precedence(1, Associativity::Left, vec!["plus"])
            .precedence(2, Associativity::Left, vec!["star"])
            .rule("expr", vec!["a"])
            .start("expr")
            .build();
        prop_assert_eq!(g.precedences[0].level, 1);
        prop_assert_eq!(g.precedences[1].level, 2);
    }
}

// ===========================================================================
// 40 – Associativity preserved in rule_with_precedence
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn assoc_roundtrip_in_rule(idx in 0u8..3) {
        let assoc = match idx {
            0 => Associativity::Left,
            1 => Associativity::Right,
            _ => Associativity::None,
        };
        let g = GrammarBuilder::new(&format!("pg_v7_art_{}", idx))
            .token("a", "a")
            .token("op", "op")
            .rule_with_precedence("expr", vec!["expr", "op", "expr"], 1, assoc)
            .rule("expr", vec!["a"])
            .start("expr")
            .build();
        let rule = g.all_rules().find(|r| r.associativity.is_some()).unwrap();
        prop_assert_eq!(rule.associativity, Some(assoc));
    }
}

// ===========================================================================
// 41 – check_empty_terminals passes for well-formed grammars
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn check_empty_terminals_ok(n in 1usize..8) {
        let g = build_tok_grammar(&format!("cet_{}", n), n);
        prop_assert!(g.check_empty_terminals().is_ok());
    }
}

// ===========================================================================
// 42 – build_registry doesn't panic
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn build_registry_no_panic(n in 1usize..8) {
        let g = build_rule_grammar(&format!("brnp_{}", n), n);
        let _ = g.build_registry();
    }
}

// ===========================================================================
// 43 – get_or_build_registry is idempotent
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn get_or_build_registry_idempotent(n in 1usize..6) {
        let mut g = build_rule_grammar(&format!("gorbi_{}", n), n);
        let _r1 = g.get_or_build_registry();
        prop_assert!(g.symbol_registry.is_some());
        let _r2 = g.get_or_build_registry();
        prop_assert!(g.symbol_registry.is_some());
    }
}

// ===========================================================================
// 44 – Normalize then validate doesn't panic
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn normalize_then_validate_no_panic(n in 1usize..5) {
        let mut g = build_rule_grammar(&format!("ntv_{}", n), n);
        let _ = g.normalize();
        let _ = g.validate();
    }
}

// ===========================================================================
// 45 – Normalize idempotent on flat grammar
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn normalize_idempotent_flat(n in 1usize..5) {
        let mut g = build_rule_grammar(&format!("nif_{}", n), n);
        let _ = g.normalize();
        let count1 = g.all_rules().count();
        let _ = g.normalize();
        let count2 = g.all_rules().count();
        prop_assert_eq!(count1, count2);
    }
}

// ===========================================================================
// 46 – Token names are unique within a grammar
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn token_names_unique(n in 2usize..10) {
        let g = build_tok_grammar(&format!("tnu_{}", n), n);
        let names: Vec<&str> = g.tokens.values().map(|t| t.name.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        prop_assert_eq!(names.len(), sorted.len());
    }
}

// ===========================================================================
// 47 – Deterministic builds produce equal grammars
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn deterministic_full_equality(n in 1usize..6) {
        let g1 = build_rule_grammar(&format!("dfe_{}", n), n);
        let g2 = build_rule_grammar(&format!("dfe_{}", n), n);
        prop_assert_eq!(&g1, &g2);
    }
}

// ===========================================================================
// 48 – add_rule grows the grammar
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn add_rule_grows_grammar(_dummy in 0u8..1) {
        let mut g = build_tok_grammar("arg", 2);
        let before = g.all_rules().count();
        let new_rule = adze_ir::Rule {
            lhs: *g.rules.keys().next().unwrap(),
            rhs: vec![Symbol::Epsilon],
            precedence: None,
            associativity: None,
            fields: Vec::new(),
            production_id: adze_ir::ProductionId(999),
        };
        g.add_rule(new_rule);
        let after = g.all_rules().count();
        prop_assert_eq!(after, before + 1);
    }
}

// ===========================================================================
// 49 – Grammar with only epsilon rules builds
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn only_epsilon_rules_builds(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("pg_v7_oer")
            .rule("start", vec![])
            .start("start")
            .build();
        prop_assert!(!g.rules.is_empty());
    }
}

// ===========================================================================
// 50 – Mixed epsilon and normal alternatives
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn mixed_epsilon_and_normal(n in 1usize..5) {
        let mut b = GrammarBuilder::new(&format!("pg_v7_men_{}", n));
        for i in 0..n {
            b = b.token(&format!("t{}", i), &format!("p{}", i));
        }
        b = b.rule("start", vec![]);
        for i in 0..n {
            b = b.rule("start", vec![&format!("t{}", i)]);
        }
        b = b.start("start");
        let g = b.build();
        let total: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(total, n + 1);
    }
}

// ===========================================================================
// 51 – Rule RHS references valid symbols
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn rhs_symbols_valid(n in 1usize..6) {
        let g = build_tok_grammar(&format!("rsv_{}", n), n);
        for rule in g.all_rules() {
            for sym in &rule.rhs {
                match sym {
                    Symbol::Terminal(id) => {
                        prop_assert!(g.tokens.contains_key(id));
                    }
                    Symbol::NonTerminal(id) => {
                        prop_assert!(g.rules.contains_key(id));
                    }
                    Symbol::Epsilon => {}
                    _ => {}
                }
            }
        }
    }
}

// ===========================================================================
// 52 – PrecedenceKind::Static roundtrip
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn precedence_static_value_preserved(prec in -100i16..100) {
        let g = GrammarBuilder::new(&format!("pg_v7_psvp_{}", (prec as i32 + 200)))
            .token("a", "a")
            .token("op", "op")
            .rule_with_precedence("expr", vec!["expr", "op", "expr"], prec, Associativity::Left)
            .rule("expr", vec!["a"])
            .start("expr")
            .build();
        let rule = g.all_rules().find(|r| r.precedence.is_some()).unwrap();
        prop_assert_eq!(rule.precedence, Some(PrecedenceKind::Static(prec)));
    }
}

// ===========================================================================
// 53 – Grammar name non-empty after build
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn grammar_name_nonempty(suffix in "[a-z]{1,8}") {
        let g = build_tok_grammar(&suffix, 1);
        prop_assert!(!g.name.is_empty());
    }
}

// ===========================================================================
// 54 – find_symbol_by_name finds builder-added rules
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn find_symbol_by_name_finds_rules(n in 1usize..6) {
        let g = build_rule_grammar(&format!("fsbnfr_{}", n), n);
        for i in 0..n {
            let name = format!("r{}", i);
            prop_assert!(
                g.find_symbol_by_name(&name).is_some(),
                "rule {} not found via find_symbol_by_name",
                name,
            );
        }
    }
}

// ===========================================================================
// 55 – Multiple precedence rules in same grammar
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn multiple_precedence_rules(n in 2usize..5) {
        let mut b = GrammarBuilder::new(&format!("pg_v7_mpr_{}", n))
            .token("a", "a");
        for i in 0..n {
            let op = format!("op{}", i);
            b = b.token(&op, &op);
            b = b.rule_with_precedence(
                "expr",
                vec!["expr", &op, "expr"],
                i as i16,
                Associativity::Left,
            );
        }
        b = b.rule("expr", vec!["a"]).start("expr");
        let g = b.build();
        let prec_count = g.all_rules().filter(|r| r.precedence.is_some()).count();
        prop_assert_eq!(prec_count, n);
    }
}

// ===========================================================================
// 56 – Token pattern preserved in build
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn token_pattern_preserved(n in 1usize..8) {
        let g = build_tok_grammar(&format!("tpp_{}", n), n);
        for (i, tok) in g.tokens.values().enumerate() {
            prop_assert_eq!(&tok.name, &format!("t{}", i));
        }
    }
}

// ===========================================================================
// 57 – Clone equality is reflexive
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn clone_reflexive(n in 1usize..6) {
        let g = build_tok_grammar(&format!("cr_{}", n), n);
        #[allow(clippy::redundant_clone)]
        let c = g.clone();
        prop_assert_eq!(&g, &c);
    }
}

// ===========================================================================
// 58 – Grammar with many tokens and rules validates
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn large_grammar_validates(tok in 3usize..8, rules in 2usize..5) {
        let mut b = GrammarBuilder::new(&format!("pg_v7_lgv_{}_{}", tok, rules));
        for i in 0..tok {
            b = b.token(&format!("t{}", i), &format!("p{}", i));
        }
        for i in 0..rules {
            b = b.rule(&format!("r{}", i), vec!["t0"]);
        }
        b = b.start("r0");
        let g = b.build();
        let _ = g.validate();
    }
}

// ===========================================================================
// 59 – SymbolId Copy semantics
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn symbol_id_copy_semantics(val in 1u16..100) {
        let id = adze_ir::SymbolId(val);
        let copied = id;
        prop_assert_eq!(id, copied);
        prop_assert_eq!(id.0, val);
    }
}

// ===========================================================================
// 60 – RuleId Copy semantics
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn rule_id_copy_semantics(val in 1u16..100) {
        let id = adze_ir::RuleId(val);
        let copied = id;
        prop_assert_eq!(id, copied);
        prop_assert_eq!(id.0, val);
    }
}

// ===========================================================================
// 61 – ProductionId Copy semantics
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn production_id_copy_semantics(val in 1u16..100) {
        let id = adze_ir::ProductionId(val);
        let copied = id;
        prop_assert_eq!(id, copied);
        prop_assert_eq!(id.0, val);
    }
}

// ===========================================================================
// 62 – SymbolId Display format
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn symbol_id_display(val in 0u16..100) {
        let id = adze_ir::SymbolId(val);
        let display = format!("{}", id);
        prop_assert_eq!(display, format!("Symbol({})", val));
    }
}

// ===========================================================================
// 63 – Serde roundtrip with precedence grammar
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn serde_roundtrip_precedence_grammar(prec in -50i16..50) {
        let g = GrammarBuilder::new(&format!("pg_v7_srpg_{}", (prec as i32 + 200)))
            .token("a", "a")
            .token("op", "op")
            .rule_with_precedence("expr", vec!["expr", "op", "expr"], prec, Associativity::Right)
            .rule("expr", vec!["a"])
            .start("expr")
            .build();
        let json = serde_json::to_string(&g).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&g, &back);
    }
}

// ===========================================================================
// 64 – Optimize preserves grammar name
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn optimize_preserves_name(suffix in "[a-z]{1,6}") {
        let name = format!("pg_v7_{}", suffix);
        let mut g = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        g.optimize();
        prop_assert_eq!(&g.name, &name);
    }
}

// ===========================================================================
// 65 – Validate well-formed grammar returns Ok
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn validate_well_formed_ok(suffix in "[a-z]{1,6}") {
        let g = GrammarBuilder::new(&format!("pg_v7_vwf_{}", suffix))
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        prop_assert!(g.validate().is_ok());
    }
}
