//! Property-based tests for GrammarBuilder (v2).
//!
//! 40+ proptest-driven tests covering: token/rule counts, name preservation,
//! start symbol, normalize stability, clone fidelity, serde roundtrip, and
//! random grammar construction.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, Symbol, SymbolId};
use proptest::prelude::*;

// ===========================================================================
// Strategies
// ===========================================================================

fn grammar_name() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9]{0,11}"
}

fn token_name_unique(max: usize) -> impl Strategy<Value = Vec<(String, String)>> {
    prop::collection::vec(("[a-z]{1,6}", "[a-z]{1,4}"), 1..=max).prop_map(|v| {
        let mut seen = std::collections::HashSet::new();
        v.into_iter()
            .filter(|(n, _)| seen.insert(n.clone()))
            .collect()
    })
}

fn build_simple(name: &str, tok_count: usize, rule_count: usize) -> Grammar {
    let mut b = GrammarBuilder::new(name);
    for i in 0..tok_count.max(1) {
        let t = format!("tok{i}");
        b = b.token(&t, &t);
    }
    for i in 0..rule_count.max(1) {
        let lhs = format!("rule{i}");
        b = b.rule(&lhs, vec!["tok0"]);
    }
    b = b.start("rule0");
    b.build()
}

fn json_roundtrip(g: &Grammar) -> Grammar {
    let json = serde_json::to_string(g).expect("serialize");
    serde_json::from_str(&json).expect("deserialize")
}

// ===========================================================================
// Area 1 – Any number of tokens, grammar name preserved
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    // 1
    #[test]
    fn prop_name_preserved_with_varying_tokens(
        name in grammar_name(),
        tok_count in 1usize..20,
    ) {
        let g = build_simple(&name, tok_count, 1);
        prop_assert_eq!(&g.name, &name);
    }

    // 2
    #[test]
    fn prop_token_count_matches(tokens in token_name_unique(15)) {
        let n = tokens.len();
        let mut b = GrammarBuilder::new("g");
        for (name, pat) in &tokens {
            b = b.token(name, pat);
        }
        b = b.rule("s", vec![&tokens[0].0]).start("s");
        let g = b.build();
        prop_assert!(g.tokens.len() >= n);
    }

    // 3
    #[test]
    fn prop_name_unchanged_after_many_tokens(
        name in grammar_name(),
        count in 1usize..30,
    ) {
        let mut b = GrammarBuilder::new(&name);
        for i in 0..count {
            b = b.token(&format!("t{i}"), &format!("p{i}"));
        }
        b = b.rule("s", vec!["t0"]).start("s");
        let g = b.build();
        prop_assert_eq!(&g.name, &name);
    }

    // 4
    #[test]
    fn prop_zero_extra_tokens_ok(name in grammar_name()) {
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build();
        prop_assert!(g.tokens.len() >= 1);
        prop_assert_eq!(&g.name, &name);
    }
}

// ===========================================================================
// Area 2 – Any number of rules, all_rules().count() >= rule count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    // 5
    #[test]
    fn prop_all_rules_at_least_rule_count(rule_count in 1usize..15) {
        let g = build_simple("g", 1, rule_count);
        prop_assert!(g.all_rules().count() >= rule_count);
    }

    // 6
    #[test]
    fn prop_alternatives_counted(alts in 1usize..10) {
        let mut b = GrammarBuilder::new("g");
        for i in 0..alts {
            b = b.token(&format!("t{i}"), &format!("t{i}"));
        }
        for i in 0..alts {
            b = b.rule("expr", vec![&format!("t{i}")]);
        }
        b = b.start("expr");
        let g = b.build();
        let eid = g.find_symbol_by_name("expr").unwrap();
        prop_assert_eq!(g.get_rules_for_symbol(eid).unwrap().len(), alts);
    }

    // 7
    #[test]
    fn prop_distinct_lhs_counted(lhs_count in 1usize..12) {
        let mut b = GrammarBuilder::new("g").token("a", "a");
        for i in 0..lhs_count {
            b = b.rule(&format!("r{i}"), vec!["a"]);
        }
        b = b.start("r0");
        let g = b.build();
        prop_assert!(g.rules.len() >= lhs_count);
    }

    // 8
    #[test]
    fn prop_empty_rhs_counted_as_rule(extra_rules in 0usize..5) {
        let mut b = GrammarBuilder::new("g").token("a", "a");
        b = b.rule("s", vec![]);
        for i in 0..extra_rules {
            b = b.rule(&format!("r{i}"), vec!["a"]);
        }
        b = b.start("s");
        let g = b.build();
        prop_assert!(g.all_rules().count() >= 1 + extra_rules);
    }
}

// ===========================================================================
// Area 3 – Start symbol always set after .start()
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    // 9
    #[test]
    fn prop_start_symbol_is_some(name in "[a-z]{1,8}") {
        let g = GrammarBuilder::new("g")
            .token("t", "t")
            .rule(&name, vec!["t"])
            .start(&name)
            .build();
        prop_assert!(g.start_symbol().is_some());
    }

    // 10
    #[test]
    fn prop_start_is_first_rule_key(name in "[a-z]{2,8}") {
        let g = GrammarBuilder::new("g")
            .token("t", "t")
            .rule("other", vec!["t"])
            .rule(&name, vec!["t"])
            .start(&name)
            .build();
        let first_key = g.rules.keys().next().unwrap();
        let first_name = g.rule_names.get(first_key).unwrap();
        prop_assert_eq!(first_name, &name);
    }

    // 11
    #[test]
    fn prop_start_symbol_survives_normalize(name in "[a-z]{1,8}") {
        let mut g = GrammarBuilder::new("g")
            .token("t", "t")
            .rule(&name, vec!["t"])
            .start(&name)
            .build();
        g.normalize();
        prop_assert!(g.start_symbol().is_some());
    }

    // 12
    #[test]
    fn prop_start_override(
        first in "[a-z]{1,6}",
        second in "[a-z]{1,6}",
    ) {
        let g = GrammarBuilder::new("g")
            .token("t", "t")
            .rule(&first, vec!["t"])
            .rule(&second, vec!["t"])
            .start(&first)
            .start(&second)
            .build();
        let first_key = g.rules.keys().next().unwrap();
        let first_name = g.rule_names.get(first_key).unwrap();
        prop_assert_eq!(first_name, &second);
    }
}

// ===========================================================================
// Area 4 – normalize() never panics for valid grammars
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    // 13
    #[test]
    fn prop_normalize_no_panic(
        tok_count in 1usize..10,
        rule_count in 1usize..10,
    ) {
        let mut g = build_simple("norm", tok_count, rule_count);
        g.normalize();
    }

    // 14
    #[test]
    fn prop_normalize_with_empty_rhs_no_panic(extra in 0usize..5) {
        let mut b = GrammarBuilder::new("g").token("a", "a");
        b = b.rule("s", vec![]);
        for i in 0..extra {
            b = b.rule(&format!("r{i}"), vec!["a"]);
        }
        b = b.start("s");
        let mut g = b.build();
        g.normalize();
    }

    // 15
    #[test]
    fn prop_normalize_preserves_tokens(tok_count in 1usize..10) {
        let mut g = build_simple("g", tok_count, 1);
        let before = g.tokens.len();
        g.normalize();
        prop_assert_eq!(g.tokens.len(), before);
    }

    // 16
    #[test]
    fn prop_normalize_does_not_lose_rules(rule_count in 1usize..10) {
        let mut g = build_simple("g", 1, rule_count);
        let before = g.all_rules().count();
        g.normalize();
        prop_assert!(g.all_rules().count() >= before);
    }
}

// ===========================================================================
// Area 5 – Clone preserves name
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    // 17
    #[test]
    fn prop_clone_preserves_name(name in grammar_name()) {
        let g = build_simple(&name, 2, 2);
        let cloned = g.clone();
        prop_assert_eq!(&g.name, &cloned.name);
    }

    // 18
    #[test]
    fn prop_clone_preserves_name_after_normalize(name in grammar_name()) {
        let mut g = build_simple(&name, 2, 2);
        g.normalize();
        let cloned = g.clone();
        prop_assert_eq!(&g.name, &cloned.name);
    }

    // 19
    #[test]
    fn prop_clone_of_clone_preserves_name(name in grammar_name()) {
        let g = build_simple(&name, 1, 1);
        let c1 = g.clone();
        let c2 = c1.clone();
        prop_assert_eq!(&g.name, &c2.name);
    }

    // 20
    #[test]
    fn prop_clone_name_with_many_tokens(
        name in grammar_name(),
        tok_count in 1usize..20,
    ) {
        let g = build_simple(&name, tok_count, 1);
        prop_assert_eq!(&g.name, &g.clone().name);
    }
}

// ===========================================================================
// Area 6 – Clone preserves rule count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    // 21
    #[test]
    fn prop_clone_preserves_rule_count(rule_count in 1usize..12) {
        let g = build_simple("g", 1, rule_count);
        let cloned = g.clone();
        prop_assert_eq!(g.all_rules().count(), cloned.all_rules().count());
    }

    // 22
    #[test]
    fn prop_clone_preserves_token_count(tok_count in 1usize..15) {
        let g = build_simple("g", tok_count, 1);
        let cloned = g.clone();
        prop_assert_eq!(g.tokens.len(), cloned.tokens.len());
    }

    // 23
    #[test]
    fn prop_clone_full_equality(
        tok_count in 1usize..8,
        rule_count in 1usize..8,
    ) {
        let g = build_simple("g", tok_count, rule_count);
        prop_assert_eq!(&g, &g.clone());
    }

    // 24
    #[test]
    fn prop_clone_preserves_extras_count(extra_count in 1usize..5) {
        let mut b = GrammarBuilder::new("g").token("x", "x");
        for i in 0..extra_count {
            let n = format!("e{i}");
            b = b.token(&n, &n).extra(&n);
        }
        b = b.rule("s", vec!["x"]).start("s");
        let g = b.build();
        let cloned = g.clone();
        prop_assert_eq!(g.extras.len(), cloned.extras.len());
    }
}

// ===========================================================================
// Area 7 – Serde roundtrip preserves name
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    // 25
    #[test]
    fn prop_serde_preserves_name(name in grammar_name()) {
        let g = build_simple(&name, 2, 2);
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(&g.name, &g2.name);
    }

    // 26
    #[test]
    fn prop_serde_preserves_token_count(tok_count in 1usize..15) {
        let g = build_simple("g", tok_count, 1);
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(g.tokens.len(), g2.tokens.len());
    }

    // 27
    #[test]
    fn prop_serde_preserves_rule_count(rule_count in 1usize..10) {
        let g = build_simple("g", 1, rule_count);
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(g.all_rules().count(), g2.all_rules().count());
    }

    // 28
    #[test]
    fn prop_serde_full_equality(
        tok_count in 1usize..8,
        rule_count in 1usize..8,
    ) {
        let g = build_simple("g", tok_count, rule_count);
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(&g, &g2);
    }
}

// ===========================================================================
// Area 8 – Multiple normalizations are stable
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    // 29
    #[test]
    fn prop_double_normalize_stable(
        tok_count in 1usize..6,
        rule_count in 1usize..6,
    ) {
        let mut g = build_simple("g", tok_count, rule_count);
        g.normalize();
        let snap1 = format!("{:?}", g.rules);
        g.normalize();
        let snap2 = format!("{:?}", g.rules);
        prop_assert_eq!(snap1, snap2);
    }

    // 30
    #[test]
    fn prop_triple_normalize_stable(rule_count in 1usize..6) {
        let mut g = build_simple("g", 2, rule_count);
        g.normalize();
        g.normalize();
        let snap_a = format!("{:?}", g.rules);
        g.normalize();
        let snap_b = format!("{:?}", g.rules);
        prop_assert_eq!(snap_a, snap_b);
    }

    // 31
    #[test]
    fn prop_normalize_name_stable(name in grammar_name()) {
        let mut g = build_simple(&name, 2, 2);
        g.normalize();
        g.normalize();
        g.normalize();
        prop_assert_eq!(&g.name, &name);
    }

    // 32
    #[test]
    fn prop_normalize_token_count_stable(tok_count in 1usize..10) {
        let mut g = build_simple("g", tok_count, 1);
        g.normalize();
        let after_first = g.tokens.len();
        g.normalize();
        prop_assert_eq!(g.tokens.len(), after_first);
    }
}

// ===========================================================================
// Area 9 – Grammar with random token count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    // 33
    #[test]
    fn prop_random_tokens_build_succeeds(tok_count in 1usize..50) {
        let _g = build_simple("rand_tok", tok_count, 1);
    }

    // 34
    #[test]
    fn prop_random_tokens_serde_roundtrip(tok_count in 1usize..25) {
        let g = build_simple("g", tok_count, 1);
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(g.tokens.len(), g2.tokens.len());
    }

    // 35
    #[test]
    fn prop_random_tokens_clone(tok_count in 1usize..25) {
        let g = build_simple("g", tok_count, 1);
        prop_assert_eq!(&g, &g.clone());
    }

    // 36
    #[test]
    fn prop_random_tokens_normalize(tok_count in 1usize..25) {
        let mut g = build_simple("g", tok_count, 1);
        let before = g.tokens.len();
        g.normalize();
        prop_assert_eq!(g.tokens.len(), before);
    }
}

// ===========================================================================
// Area 10 – Grammar with random rule count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    // 37
    #[test]
    fn prop_random_rules_build_succeeds(rule_count in 1usize..30) {
        let _g = build_simple("rand_rule", 1, rule_count);
    }

    // 38
    #[test]
    fn prop_random_rules_serde_roundtrip(rule_count in 1usize..15) {
        let g = build_simple("g", 1, rule_count);
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(g.all_rules().count(), g2.all_rules().count());
    }

    // 39
    #[test]
    fn prop_random_rules_clone(rule_count in 1usize..15) {
        let g = build_simple("g", 1, rule_count);
        prop_assert_eq!(&g, &g.clone());
    }

    // 40
    #[test]
    fn prop_random_rules_normalize(rule_count in 1usize..15) {
        let mut g = build_simple("g", 1, rule_count);
        let before = g.all_rules().count();
        g.normalize();
        prop_assert!(g.all_rules().count() >= before);
    }
}

// ===========================================================================
// Bonus – cross-cutting property tests
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    // 41
    #[test]
    fn prop_serde_then_clone_equals_original(
        name in grammar_name(),
        tok_count in 1usize..8,
    ) {
        let g = build_simple(&name, tok_count, 2);
        let g2 = json_roundtrip(&g).clone();
        prop_assert_eq!(&g, &g2);
    }

    // 42
    #[test]
    fn prop_clone_then_serde_equals_original(
        name in grammar_name(),
        rule_count in 1usize..8,
    ) {
        let g = build_simple(&name, 2, rule_count);
        let g2 = json_roundtrip(&g.clone());
        prop_assert_eq!(&g, &g2);
    }

    // 43
    #[test]
    fn prop_normalize_then_serde_roundtrip(
        tok_count in 1usize..6,
        rule_count in 1usize..6,
    ) {
        let mut g = build_simple("g", tok_count, rule_count);
        g.normalize();
        let g2 = json_roundtrip(&g);
        prop_assert_eq!(&g, &g2);
    }

    // 44
    #[test]
    fn prop_precedence_rules_no_panic(
        prec in -50i16..50,
    ) {
        let g = GrammarBuilder::new("g")
            .token("n", r"\d+")
            .token("op", "+")
            .rule_with_precedence("e", vec!["e", "op", "e"], prec, Associativity::Left)
            .rule("e", vec!["n"])
            .start("e")
            .build();
        prop_assert!(g.all_rules().count() >= 2);
    }

    // 45
    #[test]
    fn prop_associativity_variants_build(
        assoc_idx in 0usize..3,
    ) {
        let assoc = match assoc_idx {
            0 => Associativity::Left,
            1 => Associativity::Right,
            _ => Associativity::None,
        };
        let g = GrammarBuilder::new("g")
            .token("n", "n")
            .token("op", "op")
            .rule_with_precedence("e", vec!["e", "op", "e"], 1, assoc)
            .rule("e", vec!["n"])
            .start("e")
            .build();
        prop_assert!(g.all_rules().count() >= 2);
    }

    // 46
    #[test]
    fn prop_fragile_token_preserved_in_clone(name in grammar_name()) {
        let g = GrammarBuilder::new(&name)
            .fragile_token("semi", ";")
            .token("id", "id")
            .rule("s", vec!["id", "semi"])
            .start("s")
            .build();
        let cloned = g.clone();
        let has_fragile = cloned.tokens.values().any(|t| t.fragile);
        prop_assert!(has_fragile);
    }

    // 47
    #[test]
    fn prop_external_count_preserved(ext_count in 1usize..5) {
        let mut b = GrammarBuilder::new("g").token("a", "a");
        for i in 0..ext_count {
            b = b.external(&format!("ext{i}"));
        }
        b = b.rule("s", vec!["a"]).start("s");
        let g = b.build();
        prop_assert_eq!(g.externals.len(), ext_count);
    }

    // 48
    #[test]
    fn prop_symbol_id_display(id_val in 0u16..1000) {
        let id = SymbolId(id_val);
        let display = format!("{id}");
        prop_assert!(display.contains(&id_val.to_string()));
    }

    // 49
    #[test]
    fn prop_rhs_symbols_are_valid(rule_count in 1usize..6) {
        let g = build_simple("g", 3, rule_count);
        let all_ids: std::collections::HashSet<SymbolId> = g
            .tokens.keys().copied()
            .chain(g.rules.keys().copied())
            .collect();
        for rule in g.all_rules() {
            for sym in &rule.rhs {
                match sym {
                    Symbol::Terminal(id) | Symbol::NonTerminal(id) => {
                        prop_assert!(all_ids.contains(id), "unknown id {id:?}");
                    }
                    Symbol::Epsilon => {}
                    _ => {}
                }
            }
        }
    }

    // 50
    #[test]
    fn prop_production_ids_unique(rule_count in 1usize..10) {
        let g = build_simple("g", 1, rule_count);
        let ids: Vec<_> = g.all_rules().map(|r| r.production_id).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        prop_assert_eq!(ids.len(), unique.len());
    }
}
