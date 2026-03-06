//! Property-based tests for Grammar operations in adze-ir (v10).
//!
//! Covers building, cloning, normalization, optimization, validation,
//! precedence, associativity, Debug output, serde roundtrip, and more.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, SymbolId};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_token_count() -> impl Strategy<Value = usize> {
    1usize..10
}

fn arb_rule_count() -> impl Strategy<Value = usize> {
    1usize..8
}

fn arb_precedence() -> impl Strategy<Value = i16> {
    -10i16..10
}

fn arb_associativity() -> impl Strategy<Value = Associativity> {
    prop_oneof![
        Just(Associativity::Left),
        Just(Associativity::Right),
        Just(Associativity::None),
    ]
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a grammar with `n` tokens, a start rule consuming t0, and unique name.
fn build_tok_grammar(idx: usize, n: usize) -> Grammar {
    let mut b = GrammarBuilder::new(&format!("v10_tok_{idx}_{n}"));
    for i in 0..n {
        let tname = format!("t{i}");
        let pat = format!("p{i}");
        b = b.token(&tname, &pat);
    }
    b = b.rule("start", vec!["t0"]).start("start");
    b.build()
}

/// Build a grammar with `n` distinct LHS non-terminals, each consuming token "a".
fn build_rule_grammar(idx: usize, n: usize) -> Grammar {
    let mut b = GrammarBuilder::new(&format!("v10_rule_{idx}_{n}")).token("a", "a");
    for i in 0..n {
        let lhs = format!("r{i}");
        b = b.rule(&lhs, vec!["a"]);
    }
    b = b.start("r0");
    b.build()
}

/// Build a grammar with `n` alternative productions under the same LHS.
fn build_alt_grammar(idx: usize, n: usize) -> Grammar {
    let mut b = GrammarBuilder::new(&format!("v10_alt_{idx}_{n}"));
    for i in 0..n {
        let tname = format!("t{i}");
        let pat = format!("p{i}");
        b = b.token(&tname, &pat);
    }
    for i in 0..n {
        let tname = format!("t{i}");
        b = b.rule("start", vec![&tname]);
    }
    b = b.start("start");
    b.build()
}

/// Build a grammar with a rule that has the given precedence and associativity.
fn build_prec_grammar(idx: usize, prec: i16, assoc: Associativity) -> Grammar {
    GrammarBuilder::new(&format!("v10_prec_{idx}"))
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUM"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], prec, assoc)
        .start("expr")
        .build()
}

// ===========================================================================
// 1 – Grammar with N tokens always builds
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn grammar_with_n_tokens_builds(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        prop_assert!(!g.name.is_empty());
    }
}

// ===========================================================================
// 2 – Grammar with N rules always builds
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn grammar_with_n_rules_builds(n in arb_rule_count()) {
        let g = build_rule_grammar(n, n);
        prop_assert!(!g.name.is_empty());
    }
}

// ===========================================================================
// 3 – Built grammar has start_symbol
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn built_grammar_has_start_symbol(n in arb_rule_count()) {
        let g = build_rule_grammar(n, n);
        prop_assert!(g.start_symbol().is_some());
    }
}

// ===========================================================================
// 4 – Built grammar name matches
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn built_grammar_name_matches(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        prop_assert_eq!(&g.name, &format!("v10_tok_{n}_{n}"));
    }
}

// ===========================================================================
// 5 – Token count matches
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn token_count_matches(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        prop_assert_eq!(g.tokens.len(), n);
    }
}

// ===========================================================================
// 6 – Clone equals original
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn clone_name_equals_original(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        let g2 = g.clone();
        prop_assert_eq!(&g.name, &g2.name);
    }

    #[test]
    fn clone_start_symbol_equals_original(n in arb_rule_count()) {
        let g = build_rule_grammar(n, n);
        let g2 = g.clone();
        prop_assert_eq!(g.start_symbol(), g2.start_symbol());
    }

    #[test]
    fn clone_rule_names_equal_original(n in arb_rule_count()) {
        let g = build_rule_grammar(n, n);
        let g2 = g.clone();
        prop_assert_eq!(g.rule_names.len(), g2.rule_names.len());
        for (k, v) in &g.rule_names {
            prop_assert_eq!(g2.rule_names.get(k), Some(v));
        }
    }

    #[test]
    fn clone_token_count_equals_original(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        let g2 = g.clone();
        prop_assert_eq!(g.tokens.len(), g2.tokens.len());
    }
}

// ===========================================================================
// 7 – normalize is idempotent
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn normalize_idempotent(n in arb_token_count()) {
        let mut g1 = build_alt_grammar(n, n);
        g1.normalize();
        let snap1 = g1.clone();
        g1.normalize();
        prop_assert_eq!(g1.rules.len(), snap1.rules.len());
        prop_assert_eq!(g1.tokens.len(), snap1.tokens.len());
        prop_assert_eq!(g1.name, snap1.name);
    }
}

// ===========================================================================
// 8 – optimize is idempotent
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn optimize_idempotent(n in arb_token_count()) {
        let mut g1 = build_alt_grammar(n, n);
        g1.optimize();
        let snap1 = g1.clone();
        g1.optimize();
        prop_assert_eq!(g1.rules.len(), snap1.rules.len());
        prop_assert_eq!(g1.tokens.len(), snap1.tokens.len());
        prop_assert_eq!(g1.name, snap1.name);
    }
}

// ===========================================================================
// 9 – validate after build → Ok
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn validate_after_build_ok(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        prop_assert!(g.validate().is_ok());
    }
}

// ===========================================================================
// 10 – validate after normalize → Ok
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn validate_after_normalize_ok(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        g.normalize();
        prop_assert!(g.validate().is_ok());
    }
}

// ===========================================================================
// 11 – validate after optimize → Ok
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn validate_after_optimize_ok(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        g.optimize();
        prop_assert!(g.validate().is_ok());
    }
}

// ===========================================================================
// 12 – Grammar with different precedences
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn grammar_with_precedence_builds(prec in arb_precedence()) {
        let g = build_prec_grammar(prec as usize, prec, Associativity::Left);
        prop_assert!(!g.name.is_empty());
        prop_assert!(g.all_rules().count() >= 2);
    }
}

// ===========================================================================
// 13 – Grammar with various associativities
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn grammar_with_associativity_builds(assoc in arb_associativity()) {
        let g = build_prec_grammar(0, 1, assoc);
        prop_assert!(g.validate().is_ok());
    }

    #[test]
    fn prec_grammar_has_start(assoc in arb_associativity(), prec in arb_precedence()) {
        let g = build_prec_grammar(prec as usize, prec, assoc);
        prop_assert!(g.start_symbol().is_some());
    }
}

// ===========================================================================
// 14 – Grammar Debug non-empty
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn grammar_debug_non_empty(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        let debug = format!("{g:?}");
        prop_assert!(!debug.is_empty());
        prop_assert!(debug.contains("Grammar"));
    }
}

// ===========================================================================
// 15 – Grammar serde roundtrip
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn grammar_serde_json_roundtrip(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        let json = serde_json::to_string(&g).expect("serialize");
        let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(&g.name, &g2.name);
        prop_assert_eq!(g.tokens.len(), g2.tokens.len());
        prop_assert_eq!(g.rules.len(), g2.rules.len());
    }

    #[test]
    fn grammar_serde_roundtrip_preserves_rules(n in arb_rule_count()) {
        let g = build_rule_grammar(n, n);
        let json = serde_json::to_string(&g).expect("serialize");
        let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(g.all_rules().count(), g2.all_rules().count());
    }

    #[test]
    fn grammar_serde_roundtrip_preserves_start(n in arb_rule_count()) {
        let g = build_rule_grammar(n, n);
        let json = serde_json::to_string(&g).expect("serialize");
        let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(g.start_symbol(), g2.start_symbol());
    }
}

// ===========================================================================
// 16 – all_rules count matches expected
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn all_rules_count_single_rule_per_nonterminal(n in arb_rule_count()) {
        let g = build_rule_grammar(n, n);
        prop_assert_eq!(g.all_rules().count(), n);
    }

    #[test]
    fn all_rules_count_alt_grammar(n in arb_token_count()) {
        let g = build_alt_grammar(n, n);
        prop_assert_eq!(g.all_rules().count(), n);
    }
}

// ===========================================================================
// 17 – token IDs are distinct
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn token_ids_are_distinct(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        let ids: Vec<SymbolId> = g.tokens.keys().copied().collect();
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                prop_assert_ne!(ids[i], ids[j]);
            }
        }
    }
}

// ===========================================================================
// 18 – rule_names contains expected names
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn rule_names_contains_start(n in arb_rule_count()) {
        let g = build_rule_grammar(n, n);
        let names: Vec<&String> = g.rule_names.values().collect();
        prop_assert!(names.iter().any(|name| *name == "r0"));
    }

    #[test]
    fn rule_names_contains_all_nonterminals(n in arb_rule_count()) {
        let g = build_rule_grammar(n, n);
        let names: Vec<&str> = g.rule_names.values().map(|s| s.as_str()).collect();
        for i in 0..n {
            let expected = format!("r{i}");
            prop_assert!(
                names.contains(&expected.as_str()),
                "missing rule name: {expected}"
            );
        }
    }
}

// ===========================================================================
// 19 – normalize doesn't panic
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn normalize_does_not_panic_tok(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        g.normalize();
    }

    #[test]
    fn normalize_does_not_panic_rule(n in arb_rule_count()) {
        let mut g = build_rule_grammar(n, n);
        g.normalize();
    }

    #[test]
    fn normalize_does_not_panic_alt(n in arb_token_count()) {
        let mut g = build_alt_grammar(n, n);
        g.normalize();
    }
}

// ===========================================================================
// 20 – optimize doesn't panic
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn optimize_does_not_panic_tok(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        g.optimize();
    }

    #[test]
    fn optimize_does_not_panic_rule(n in arb_rule_count()) {
        let mut g = build_rule_grammar(n, n);
        g.optimize();
    }

    #[test]
    fn optimize_does_not_panic_alt(n in arb_token_count()) {
        let mut g = build_alt_grammar(n, n);
        g.optimize();
    }
}

// ===========================================================================
// 21 – normalize preserves name
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn normalize_preserves_name(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        let name_before = g.name.clone();
        g.normalize();
        prop_assert_eq!(&g.name, &name_before);
    }
}

// ===========================================================================
// 22 – optimize preserves name
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn optimize_preserves_name(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        let name_before = g.name.clone();
        g.optimize();
        prop_assert_eq!(&g.name, &name_before);
    }
}

// ===========================================================================
// 23 – normalize preserves tokens
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn normalize_preserves_tokens(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        let count_before = g.tokens.len();
        g.normalize();
        prop_assert_eq!(g.tokens.len(), count_before);
    }
}

// ===========================================================================
// 24 – optimize preserves tokens
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn optimize_preserves_tokens(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        let count_before = g.tokens.len();
        g.optimize();
        prop_assert_eq!(g.tokens.len(), count_before);
    }
}

// ===========================================================================
// 25 – validate after normalize+optimize → Ok
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn validate_after_normalize_then_optimize(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        g.normalize();
        g.optimize();
        prop_assert!(g.validate().is_ok());
    }

    #[test]
    fn validate_after_optimize_then_normalize(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        g.optimize();
        g.normalize();
        prop_assert!(g.validate().is_ok());
    }
}

// ===========================================================================
// 26 – serde roundtrip after normalize
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn serde_roundtrip_after_normalize(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        g.normalize();
        let json = serde_json::to_string(&g).expect("serialize");
        let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(&g.name, &g2.name);
        prop_assert_eq!(g.tokens.len(), g2.tokens.len());
    }
}

// ===========================================================================
// 27 – serde roundtrip after optimize
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn serde_roundtrip_after_optimize(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        g.optimize();
        let json = serde_json::to_string(&g).expect("serialize");
        let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(&g.name, &g2.name);
        prop_assert_eq!(g.tokens.len(), g2.tokens.len());
    }
}

// ===========================================================================
// 28 – prec grammar validate ok
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prec_grammar_validates(prec in arb_precedence(), assoc in arb_associativity()) {
        let g = build_prec_grammar(prec as usize, prec, assoc);
        prop_assert!(g.validate().is_ok());
    }
}

// ===========================================================================
// 29 – prec grammar normalize doesn't panic
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prec_grammar_normalize_no_panic(prec in arb_precedence(), assoc in arb_associativity()) {
        let mut g = build_prec_grammar(prec as usize, prec, assoc);
        g.normalize();
    }
}

// ===========================================================================
// 30 – prec grammar optimize doesn't panic
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prec_grammar_optimize_no_panic(prec in arb_precedence(), assoc in arb_associativity()) {
        let mut g = build_prec_grammar(prec as usize, prec, assoc);
        g.optimize();
    }
}

// ===========================================================================
// 31 – alt grammar token count preserved
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn alt_grammar_token_count(n in arb_token_count()) {
        let g = build_alt_grammar(n, n);
        prop_assert_eq!(g.tokens.len(), n);
    }
}

// ===========================================================================
// 32 – alt grammar has start symbol
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn alt_grammar_has_start(n in arb_token_count()) {
        let g = build_alt_grammar(n, n);
        prop_assert!(g.start_symbol().is_some());
    }
}

// ===========================================================================
// 33 – Debug output contains grammar name
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn debug_contains_grammar_name(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        let debug = format!("{g:?}");
        prop_assert!(debug.contains(&g.name));
    }
}

// ===========================================================================
// 34 – serde preserves precedence info
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn serde_preserves_precedences(prec in arb_precedence(), assoc in arb_associativity()) {
        let g = build_prec_grammar(prec as usize, prec, assoc);
        let json = serde_json::to_string(&g).expect("serialize");
        let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(g.precedences.len(), g2.precedences.len());
    }
}

// ===========================================================================
// 35 – Default grammar is empty
// ===========================================================================

#[test]
fn default_grammar_is_empty() {
    let g = Grammar::default();
    assert!(g.name.is_empty());
    assert!(g.rules.is_empty());
    assert!(g.tokens.is_empty());
    assert!(g.rule_names.is_empty());
    assert!(g.start_symbol().is_none());
}

// ===========================================================================
// 36 – clone of default grammar
// ===========================================================================

#[test]
fn clone_of_default_grammar() {
    let g = Grammar::default();
    let g2 = g.clone();
    assert_eq!(g.name, g2.name);
    assert_eq!(g.rules.len(), g2.rules.len());
}

// ===========================================================================
// 37 – validate default grammar → Ok (no rules, no issues)
// ===========================================================================

#[test]
fn validate_default_grammar() {
    let g = Grammar::default();
    assert!(g.validate().is_ok());
}

// ===========================================================================
// 38 – normalize default grammar doesn't panic
// ===========================================================================

#[test]
fn normalize_default_grammar() {
    let mut g = Grammar::default();
    g.normalize();
}

// ===========================================================================
// 39 – optimize default grammar doesn't panic
// ===========================================================================

#[test]
fn optimize_default_grammar() {
    let mut g = Grammar::default();
    g.optimize();
}

// ===========================================================================
// 40 – serde roundtrip for default grammar
// ===========================================================================

#[test]
fn serde_roundtrip_default_grammar() {
    let g = Grammar::default();
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(g.name, g2.name);
}

// ===========================================================================
// 41 – token-only grammar (no rules)
// ===========================================================================

#[test]
fn token_only_grammar_validates() {
    let g = GrammarBuilder::new("v10_tok_only")
        .token("A", "a")
        .token("B", "b")
        .build();
    assert!(g.validate().is_ok());
    assert_eq!(g.tokens.len(), 2);
    assert_eq!(g.all_rules().count(), 0);
}

// ===========================================================================
// 42 – rule grammar has correct rule names count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn rule_grammar_rule_names_count(n in arb_rule_count()) {
        let g = build_rule_grammar(n, n);
        // Each unique LHS nonterminal should appear in rule_names
        prop_assert!(g.rule_names.len() >= n);
    }
}

// ===========================================================================
// 43 – normalize then validate for rule grammars
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn normalize_then_validate_rule_grammar(n in arb_rule_count()) {
        let mut g = build_rule_grammar(n, n);
        g.normalize();
        prop_assert!(g.validate().is_ok());
    }
}

// ===========================================================================
// 44 – optimize then validate for rule grammars
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn optimize_then_validate_rule_grammar(n in arb_rule_count()) {
        let mut g = build_rule_grammar(n, n);
        g.optimize();
        prop_assert!(g.validate().is_ok());
    }
}

// ===========================================================================
// 45 – clone after normalize preserves state
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn clone_after_normalize(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        g.normalize();
        let g2 = g.clone();
        prop_assert_eq!(&g.name, &g2.name);
        prop_assert_eq!(g.tokens.len(), g2.tokens.len());
        prop_assert_eq!(g.all_rules().count(), g2.all_rules().count());
    }
}

// ===========================================================================
// 46 – clone after optimize preserves state
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn clone_after_optimize(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        g.optimize();
        let g2 = g.clone();
        prop_assert_eq!(&g.name, &g2.name);
        prop_assert_eq!(g.tokens.len(), g2.tokens.len());
        prop_assert_eq!(g.all_rules().count(), g2.all_rules().count());
    }
}

// ===========================================================================
// 47 – each associativity variant builds successfully
// ===========================================================================

#[test]
fn associativity_left_builds() {
    let g = build_prec_grammar(100, 1, Associativity::Left);
    assert!(g.validate().is_ok());
}

#[test]
fn associativity_right_builds() {
    let g = build_prec_grammar(101, 1, Associativity::Right);
    assert!(g.validate().is_ok());
}

#[test]
fn associativity_none_builds() {
    let g = build_prec_grammar(102, 1, Associativity::None);
    assert!(g.validate().is_ok());
}

// ===========================================================================
// 48 – negative precedence values work
// ===========================================================================

#[test]
fn negative_precedence_builds() {
    let g = build_prec_grammar(103, -5, Associativity::Left);
    assert!(g.validate().is_ok());
    assert!(g.all_rules().count() >= 2);
}

// ===========================================================================
// 49 – zero precedence builds
// ===========================================================================

#[test]
fn zero_precedence_builds() {
    let g = build_prec_grammar(104, 0, Associativity::Left);
    assert!(g.validate().is_ok());
}

// ===========================================================================
// 50 – grammar with multiple precedence levels
// ===========================================================================

#[test]
fn multiple_precedence_levels() {
    let g = GrammarBuilder::new("v10_multiprec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["NUM"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .start("expr")
        .build();
    assert!(g.validate().is_ok());
    assert_eq!(g.all_rules().count(), 3);
}

// ===========================================================================
// 51 – single token grammar
// ===========================================================================

#[test]
fn single_token_grammar() {
    let g = GrammarBuilder::new("v10_single")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    assert_eq!(g.tokens.len(), 1);
    assert_eq!(g.all_rules().count(), 1);
    assert!(g.validate().is_ok());
}

// ===========================================================================
// 52 – grammar PartialEq on clones
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn grammar_partialeq_clone(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        let g2 = g.clone();
        prop_assert_eq!(&g, &g2);
    }
}

// ===========================================================================
// 53 – different grammars are not equal
// ===========================================================================

#[test]
fn different_grammars_not_equal() {
    let g1 = build_tok_grammar(200, 2);
    let g2 = build_tok_grammar(201, 3);
    assert_ne!(g1, g2);
}

// ===========================================================================
// 54 – serde preserves equality
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn serde_preserves_equality(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        let json = serde_json::to_string(&g).expect("serialize");
        let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(&g, &g2);
    }
}

// ===========================================================================
// 55 – rule_names for alt grammar contains "start"
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn alt_grammar_rule_names_has_start(n in arb_token_count()) {
        let g = build_alt_grammar(n, n);
        let names: Vec<&str> = g.rule_names.values().map(|s| s.as_str()).collect();
        prop_assert!(names.contains(&"start"));
    }
}

// ===========================================================================
// 56-60 – combined property checks
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn build_normalize_validate_pipeline(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        prop_assert!(g.validate().is_ok());
        g.normalize();
        prop_assert!(g.validate().is_ok());
    }

    #[test]
    fn build_optimize_validate_pipeline(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        prop_assert!(g.validate().is_ok());
        g.optimize();
        prop_assert!(g.validate().is_ok());
    }

    #[test]
    fn build_normalize_optimize_validate_pipeline(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        g.normalize();
        g.optimize();
        prop_assert!(g.validate().is_ok());
    }

    #[test]
    fn build_optimize_normalize_validate_pipeline(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        g.optimize();
        g.normalize();
        prop_assert!(g.validate().is_ok());
    }

    #[test]
    fn full_pipeline_preserves_name(n in arb_token_count()) {
        let mut g = build_tok_grammar(n, n);
        let name = g.name.clone();
        g.normalize();
        g.optimize();
        prop_assert_eq!(&g.name, &name);
    }
}

// ===========================================================================
// 61-65 – token properties
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn token_names_are_unique(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        let names: Vec<&str> = g.tokens.values().map(|t| t.name.as_str()).collect();
        for i in 0..names.len() {
            for j in (i + 1)..names.len() {
                prop_assert_ne!(names[i], names[j]);
            }
        }
    }

    #[test]
    fn token_patterns_are_non_empty(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        for token in g.tokens.values() {
            let pat_str = format!("{:?}", token.pattern);
            prop_assert!(!pat_str.is_empty());
        }
    }

    #[test]
    fn tokens_not_fragile_by_default(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        for token in g.tokens.values() {
            prop_assert!(!token.fragile);
        }
    }

    #[test]
    fn rule_grammar_has_single_token(n in arb_rule_count()) {
        let g = build_rule_grammar(n, n);
        prop_assert_eq!(g.tokens.len(), 1);
    }

    #[test]
    fn alt_grammar_all_rules_same_lhs(n in arb_token_count()) {
        let g = build_alt_grammar(n, n);
        let lhs_ids: Vec<SymbolId> = g.all_rules().map(|r| r.lhs).collect();
        if let Some(&first) = lhs_ids.first() {
            for &id in &lhs_ids {
                prop_assert_eq!(id, first);
            }
        }
    }
}

// ===========================================================================
// 66-70 – rule structure
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn each_rule_has_nonempty_rhs(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        for rule in g.all_rules() {
            prop_assert!(!rule.rhs.is_empty());
        }
    }

    #[test]
    fn rules_map_keys_match_lhs(n in arb_rule_count()) {
        let g = build_rule_grammar(n, n);
        for (&lhs_key, rules) in &g.rules {
            for rule in rules {
                prop_assert_eq!(lhs_key, rule.lhs);
            }
        }
    }

    #[test]
    fn rule_fields_empty_by_default(n in arb_rule_count()) {
        let g = build_rule_grammar(n, n);
        for rule in g.all_rules() {
            prop_assert!(rule.fields.is_empty());
        }
    }

    #[test]
    fn prec_rule_has_precedence_set(prec in arb_precedence(), assoc in arb_associativity()) {
        let g = build_prec_grammar(prec as usize, prec, assoc);
        let with_prec: Vec<_> = g.all_rules().filter(|r| r.precedence.is_some()).collect();
        prop_assert!(!with_prec.is_empty());
    }

    #[test]
    fn prec_rule_has_associativity_set(prec in arb_precedence(), assoc in arb_associativity()) {
        let g = build_prec_grammar(prec as usize, prec, assoc);
        let with_assoc: Vec<_> = g.all_rules().filter(|r| r.associativity.is_some()).collect();
        prop_assert!(!with_assoc.is_empty());
    }
}

// ===========================================================================
// 71-75 – serde edge cases
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn serde_roundtrip_rule_grammar(n in arb_rule_count()) {
        let g = build_rule_grammar(n, n);
        let json = serde_json::to_string(&g).expect("serialize");
        let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(&g, &g2);
    }

    #[test]
    fn serde_roundtrip_alt_grammar(n in arb_token_count()) {
        let g = build_alt_grammar(n, n);
        let json = serde_json::to_string(&g).expect("serialize");
        let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(&g, &g2);
    }

    #[test]
    fn serde_roundtrip_prec_grammar(prec in arb_precedence(), assoc in arb_associativity()) {
        let g = build_prec_grammar(prec as usize, prec, assoc);
        let json = serde_json::to_string(&g).expect("serialize");
        let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
        prop_assert_eq!(&g, &g2);
    }

    #[test]
    fn serde_json_is_valid_json(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        let json = serde_json::to_string(&g).expect("serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        prop_assert!(parsed.is_object());
    }

    #[test]
    fn serde_json_contains_name_field(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        let json = serde_json::to_string(&g).expect("serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        prop_assert_eq!(parsed["name"].as_str(), Some(g.name.as_str()));
    }
}

// ===========================================================================
// 76-80 – misc structural invariants
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn extras_empty_by_default(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        prop_assert!(g.extras.is_empty());
    }

    #[test]
    fn conflicts_empty_by_default(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        prop_assert!(g.conflicts.is_empty());
    }

    #[test]
    fn externals_empty_by_default(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        prop_assert!(g.externals.is_empty());
    }

    #[test]
    fn fields_empty_by_default(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        prop_assert!(g.fields.is_empty());
    }

    #[test]
    fn supertypes_empty_by_default(n in arb_token_count()) {
        let g = build_tok_grammar(n, n);
        prop_assert!(g.supertypes.is_empty());
    }
}

// ===========================================================================
// 81-84 – additional coverage
// ===========================================================================

#[test]
fn debug_default_grammar_non_empty() {
    let g = Grammar::default();
    let debug = format!("{g:?}");
    assert!(!debug.is_empty());
}

#[test]
fn serde_default_grammar_roundtrip() {
    let g = Grammar::default();
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(g, g2);
}

#[test]
fn grammar_with_many_alt_rules() {
    let g = build_alt_grammar(999, 9);
    assert_eq!(g.all_rules().count(), 9);
    assert_eq!(g.tokens.len(), 9);
    assert!(g.validate().is_ok());
}

#[test]
fn grammar_with_max_token_count() {
    let g = build_tok_grammar(998, 9);
    assert_eq!(g.tokens.len(), 9);
    assert!(g.validate().is_ok());
}
