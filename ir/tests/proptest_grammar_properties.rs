//! Property-based tests for adze-ir grammar properties.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Symbol, SymbolId};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn token_name_strategy() -> impl Strategy<Value = String> {
    "[a-z]{1,8}"
}

fn rule_name_strategy() -> impl Strategy<Value = String> {
    "[a-z]{1,8}"
}

fn token_pattern_strategy() -> impl Strategy<Value = String> {
    "[a-z]{1,5}"
}

/// Generate a vec of (name, pattern) pairs for tokens, with unique names.
fn token_list_strategy(max: usize) -> impl Strategy<Value = Vec<(String, String)>> {
    prop::collection::vec((token_name_strategy(), token_pattern_strategy()), 1..=max).prop_map(
        |v| {
            let mut seen = std::collections::HashSet::new();
            v.into_iter()
                .filter(|(n, _)| seen.insert(n.clone()))
                .collect()
        },
    )
}

/// Generate a vec of unique rule-name strings.
fn rule_name_list_strategy(max: usize) -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(rule_name_strategy(), 1..=max).prop_map(|v| {
        let mut seen = std::collections::HashSet::new();
        v.into_iter().filter(|n| seen.insert(n.clone())).collect()
    })
}

// ---------------------------------------------------------------------------
// Property 1: Any grammar built with GrammarBuilder has non-empty name
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_grammar_has_nonempty_name(name in "[a-z]{1,12}") {
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build();
        prop_assert!(!g.name.is_empty());
        prop_assert_eq!(g.name, name);
    }
}

// ---------------------------------------------------------------------------
// Property 2: Grammars with N tokens have at least N entries in tokens map
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_n_tokens_in_map(tokens in token_list_strategy(10)) {
        let n = tokens.len();
        let mut b = GrammarBuilder::new("g");
        for (name, pat) in &tokens {
            b = b.token(name, pat);
        }
        // Need at least one rule to build a valid grammar
        let first = &tokens[0].0;
        b = b.rule("start", vec![first]).start("start");
        let g = b.build();
        prop_assert!(g.tokens.len() >= n);
    }
}

// ---------------------------------------------------------------------------
// Property 3: Grammars with rules have at least those rules in rules map
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_rules_present(
        tok in token_list_strategy(5),
        rule_count in 1usize..6,
    ) {
        let mut b = GrammarBuilder::new("g");
        for (name, pat) in &tok {
            b = b.token(name, pat);
        }
        let first_tok = &tok[0].0;
        for i in 0..rule_count {
            let lhs = format!("r{}", i);
            b = b.rule(&lhs, vec![first_tok]);
        }
        b = b.start("r0");
        let g = b.build();
        // Each distinct LHS should be present
        prop_assert!(g.rules.len() >= rule_count);
    }
}

// ---------------------------------------------------------------------------
// Property 4: normalize() is idempotent
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_normalize_idempotent(tok_count in 1usize..4, rule_count in 1usize..4) {
        let mut b = GrammarBuilder::new("idem");
        for i in 0..tok_count {
            let name = format!("t{}", i);
            b = b.token(&name, &name);
        }
        for i in 0..rule_count {
            let lhs = format!("r{}", i);
            b = b.rule(&lhs, vec!["t0"]);
        }
        b = b.start("r0");
        let mut g1 = b.build();
        g1.normalize();
        let snap1 = format!("{:?}", g1.rules);

        g1.normalize();
        let snap2 = format!("{:?}", g1.rules);
        prop_assert_eq!(snap1, snap2, "normalize must be idempotent");
    }
}

// ---------------------------------------------------------------------------
// Property 5: Grammar name is preserved through normalize()
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_name_preserved_through_normalize(name in "[a-z]{1,10}") {
        let mut g = GrammarBuilder::new(&name)
            .token("x", "x")
            .rule("s", vec!["x"])
            .start("s")
            .build();
        let before = g.name.clone();
        g.normalize();
        prop_assert_eq!(before, g.name);
    }
}

// ---------------------------------------------------------------------------
// Property 6: start_symbol() returns Some when .start() was called
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_start_symbol_some(name in rule_name_strategy()) {
        let g = GrammarBuilder::new("g")
            .token("tok", "tok")
            .rule(&name, vec!["tok"])
            .start(&name)
            .build();
        // start_symbol uses heuristics; the first rule's LHS is the start symbol.
        // Since we called .start(), the start rule is placed first.
        prop_assert!(g.start_symbol().is_some());
    }
}

// ---------------------------------------------------------------------------
// Property 7: Token count is preserved through normalize()
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_token_count_preserved_through_normalize(tokens in token_list_strategy(8)) {
        let n = tokens.len();
        let mut b = GrammarBuilder::new("g");
        for (name, pat) in &tokens {
            b = b.token(name, pat);
        }
        let first = &tokens[0].0;
        b = b.rule("s", vec![first]).start("s");
        let mut g = b.build();
        let before = g.tokens.len();
        g.normalize();
        prop_assert_eq!(before, g.tokens.len(), "tokens must not change on normalize");
        prop_assert!(g.tokens.len() >= n);
    }
}

// ---------------------------------------------------------------------------
// Property 8: All RHS symbols in rules reference existing symbol IDs
// ---------------------------------------------------------------------------

fn collect_symbol_ids(sym: &Symbol) -> Vec<SymbolId> {
    match sym {
        Symbol::Terminal(id) | Symbol::NonTerminal(id) | Symbol::External(id) => vec![*id],
        Symbol::Optional(inner) | Symbol::Repeat(inner) | Symbol::RepeatOne(inner) => {
            collect_symbol_ids(inner)
        }
        Symbol::Choice(v) | Symbol::Sequence(v) => v.iter().flat_map(collect_symbol_ids).collect(),
        Symbol::Epsilon => vec![],
    }
}

proptest! {
    #[test]
    fn prop_rhs_ids_exist(tok_count in 1usize..5, rule_count in 1usize..5) {
        let mut b = GrammarBuilder::new("g");
        for i in 0..tok_count {
            let name = format!("t{}", i);
            b = b.token(&name, &name);
        }
        for i in 0..rule_count {
            let lhs = format!("r{}", i);
            b = b.rule(&lhs, vec!["t0"]);
        }
        b = b.start("r0");
        let g = b.build();

        let all_known: std::collections::HashSet<SymbolId> = g
            .tokens
            .keys()
            .copied()
            .chain(g.rules.keys().copied())
            .collect();

        for rule in g.all_rules() {
            for sym in &rule.rhs {
                for id in collect_symbol_ids(sym) {
                    prop_assert!(
                        all_known.contains(&id),
                        "RHS id {:?} not in known ids",
                        id
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Property 9: Randomly generated simple grammars don't panic on construction
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_no_panic_on_construction(
        name in "[a-z]{1,8}",
        tok_count in 1usize..8,
        rule_count in 1usize..8,
    ) {
        let mut b = GrammarBuilder::new(&name);
        for i in 0..tok_count {
            let tname = format!("t{}", i);
            b = b.token(&tname, &tname);
        }
        for i in 0..rule_count {
            let lhs = format!("r{}", i);
            b = b.rule(&lhs, vec!["t0"]);
        }
        b = b.start("r0");
        let _g = b.build(); // must not panic
    }
}

// ---------------------------------------------------------------------------
// Property 10: Grammar with K alternatives for same LHS has K rules
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_k_alternatives(k in 1usize..10) {
        let mut b = GrammarBuilder::new("g");
        // Create k distinct tokens to use as separate alternatives
        for i in 0..k {
            let tname = format!("t{}", i);
            b = b.token(&tname, &tname);
        }
        for i in 0..k {
            let tname = format!("t{}", i);
            b = b.rule("expr", vec![&tname]);
        }
        b = b.start("expr");
        let g = b.build();

        let expr_id = g.find_symbol_by_name("expr").unwrap();
        let rules = g.get_rules_for_symbol(expr_id).unwrap();
        prop_assert_eq!(rules.len(), k, "expected {} alternatives, got {}", k, rules.len());
    }
}

// ===========================================================================
// Regular #[test] cases
// ===========================================================================

#[test]
fn test_empty_grammar_name_is_stored() {
    // GrammarBuilder::new("") gives an empty name, but property 1 says non-empty
    // when name strategy is [a-z]{1,12}. Verify "" is stored faithfully.
    let g = GrammarBuilder::new("")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name, "");
}

#[test]
fn test_single_token_grammar() {
    let g = GrammarBuilder::new("one")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 1);
    assert_eq!(g.rules.len(), 1);
}

#[test]
fn test_multiple_tokens() {
    let g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn test_duplicate_token_name_overwrites() {
    let g = GrammarBuilder::new("dup")
        .token("a", "first")
        .token("a", "second")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    // Same name → same SymbolId → overwrite in IndexMap
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn test_start_symbol_present() {
    let g = GrammarBuilder::new("g")
        .token("x", "x")
        .rule("main", vec!["x"])
        .start("main")
        .build();
    assert!(g.start_symbol().is_some());
}

#[test]
fn test_normalize_simple_grammar_unchanged() {
    let mut g = GrammarBuilder::new("g")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let before = g.rules.clone();
    g.normalize();
    // Simple grammar has no complex symbols, rules should stay the same structure-wise
    assert_eq!(g.rules.len(), before.len());
}

#[test]
fn test_normalize_preserves_tokens() {
    let mut g = GrammarBuilder::new("g")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let tok_count = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), tok_count);
}

#[test]
fn test_name_survives_normalize() {
    let mut g = GrammarBuilder::new("mygrammar")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    g.normalize();
    assert_eq!(g.name, "mygrammar");
}

#[test]
fn test_rule_alternatives_counted() {
    let g = GrammarBuilder::new("g")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let sid = g.find_symbol_by_name("s").unwrap();
    assert_eq!(g.get_rules_for_symbol(sid).unwrap().len(), 3);
}

#[test]
fn test_rhs_terminal_ids_in_tokens() {
    let g = GrammarBuilder::new("g")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x", "y"])
        .start("s")
        .build();
    for rule in g.all_rules() {
        for sym in &rule.rhs {
            if let Symbol::Terminal(id) = sym {
                assert!(g.tokens.contains_key(id));
            }
        }
    }
}

#[test]
fn test_rhs_nonterminal_ids_in_rules() {
    let g = GrammarBuilder::new("g")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let rule_lhs_ids: std::collections::HashSet<_> = g.rules.keys().copied().collect();
    for rule in g.all_rules() {
        for sym in &rule.rhs {
            if let Symbol::NonTerminal(id) = sym {
                assert!(
                    rule_lhs_ids.contains(id),
                    "NonTerminal {:?} not in rules",
                    id
                );
            }
        }
    }
}

#[test]
fn test_no_panic_large_grammar() {
    let mut b = GrammarBuilder::new("big");
    for i in 0..50 {
        let tname = format!("t{}", i);
        b = b.token(&tname, &tname);
    }
    for i in 0..20 {
        let rname = format!("r{}", i);
        b = b.rule(&rname, vec!["t0", "t1"]);
    }
    b = b.start("r0");
    let _g = b.build();
}

#[test]
fn test_normalize_idempotent_simple() {
    let mut g = GrammarBuilder::new("g")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    let snap1 = format!("{:?}", g.rules);
    g.normalize();
    let snap2 = format!("{:?}", g.rules);
    assert_eq!(snap1, snap2);
}

#[test]
fn test_find_symbol_by_name() {
    let g = GrammarBuilder::new("g")
        .token("x", "x")
        .rule("alpha", vec!["x"])
        .start("alpha")
        .build();
    assert!(g.find_symbol_by_name("alpha").is_some());
}

#[test]
fn test_find_symbol_by_name_missing() {
    let g = GrammarBuilder::new("g")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert!(g.find_symbol_by_name("nonexistent").is_none());
}

#[test]
fn test_all_rules_iterator() {
    let g = GrammarBuilder::new("g")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("t", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.all_rules().count(), 3);
}

#[test]
fn test_epsilon_rule() {
    let g = GrammarBuilder::new("g")
        .token("x", "x")
        .rule("s", vec![])
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let sid = g.find_symbol_by_name("s").unwrap();
    let rules = g.get_rules_for_symbol(sid).unwrap();
    assert!(rules.iter().any(|r| r.rhs.contains(&Symbol::Epsilon)));
}

#[test]
fn test_grammar_clone_equality() {
    let g = GrammarBuilder::new("g")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let g2 = g.clone();
    assert_eq!(g, g2);
}

#[test]
fn test_start_reorders_rules() {
    let g = GrammarBuilder::new("g")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["x"])
        .start("b")
        .build();
    // "b" should be first key in rules because .start("b") was called
    let first_key = g.rules.keys().next().unwrap();
    let name = g.rule_names.get(first_key).unwrap();
    assert_eq!(name, "b");
}

#[test]
fn test_fragile_token() {
    let g = GrammarBuilder::new("g")
        .fragile_token("err", "err")
        .rule("s", vec!["err"])
        .start("s")
        .build();
    let tok = g.tokens.values().next().unwrap();
    assert!(tok.fragile);
}

#[test]
fn test_extra_token() {
    let g = GrammarBuilder::new("g")
        .token("ws", "ws")
        .token("x", "x")
        .extra("ws")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert!(!g.extras.is_empty());
}

#[test]
fn test_external_token() {
    let g = GrammarBuilder::new("g")
        .token("x", "x")
        .external("indent")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.externals[0].name, "indent");
}

#[test]
fn test_python_like_preset() {
    let g = GrammarBuilder::python_like();
    assert_eq!(g.name, "python_like");
    assert!(g.start_symbol().is_some());
    assert!(!g.tokens.is_empty());
    assert!(!g.rules.is_empty());
}

#[test]
fn test_javascript_like_preset() {
    let g = GrammarBuilder::javascript_like();
    assert_eq!(g.name, "javascript_like");
    assert!(g.start_symbol().is_some());
    assert!(!g.tokens.is_empty());
    assert!(!g.rules.is_empty());
}

#[test]
fn test_rule_with_precedence() {
    let g = GrammarBuilder::new("g")
        .token("x", "x")
        .token("p", "p")
        .rule_with_precedence("e", vec!["e", "p", "e"], 5, adze_ir::Associativity::Left)
        .rule("e", vec!["x"])
        .start("e")
        .build();
    let eid = g.find_symbol_by_name("e").unwrap();
    let rules = g.get_rules_for_symbol(eid).unwrap();
    assert!(rules.iter().any(|r| r.precedence.is_some()));
}

#[test]
fn test_production_ids_unique() {
    let g = GrammarBuilder::new("g")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("t", vec!["a"])
        .start("s")
        .build();
    let ids: Vec<_> = g.all_rules().map(|r| r.production_id).collect();
    let unique: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(ids.len(), unique.len(), "production IDs must be unique");
}

#[test]
fn test_symbol_id_is_u16() {
    let id = SymbolId(65535u16);
    assert_eq!(id.0, 65535);
}

#[test]
fn test_add_rule_post_build() {
    let mut g = GrammarBuilder::new("g")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let new_rule = adze_ir::Rule {
        lhs: SymbolId(999),
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(100),
    };
    g.add_rule(new_rule);
    assert!(g.rules.contains_key(&SymbolId(999)));
}

// ---------------------------------------------------------------------------
// Additional proptest blocks for broader coverage
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn prop_grammar_name_matches_input(name in "[a-z]{1,15}") {
        let g = GrammarBuilder::new(&name)
            .token("t", "t")
            .rule("s", vec!["t"])
            .start("s")
            .build();
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn prop_all_rules_count(
        alts_a in 1usize..5,
        alts_b in 1usize..5,
    ) {
        let mut b = GrammarBuilder::new("g");
        for i in 0..(alts_a + alts_b) {
            let tname = format!("t{}", i);
            b = b.token(&tname, &tname);
        }
        for i in 0..alts_a {
            let tname = format!("t{}", i);
            b = b.rule("a", vec![&tname]);
        }
        for i in 0..alts_b {
            let tname = format!("t{}", alts_a + i);
            b = b.rule("b", vec![&tname]);
        }
        b = b.start("a");
        let g = b.build();
        prop_assert_eq!(g.all_rules().count(), alts_a + alts_b);
    }

    #[test]
    fn prop_clone_equals_original(tok_count in 1usize..5) {
        let mut b = GrammarBuilder::new("g");
        for i in 0..tok_count {
            let t = format!("t{}", i);
            b = b.token(&t, &t);
        }
        b = b.rule("s", vec!["t0"]).start("s");
        let g = b.build();
        let g2 = g.clone();
        prop_assert_eq!(g, g2);
    }

    #[test]
    fn prop_normalize_does_not_lose_rules(rule_count in 1usize..6) {
        let mut b = GrammarBuilder::new("g").token("a", "a");
        for i in 0..rule_count {
            let lhs = format!("r{}", i);
            b = b.rule(&lhs, vec!["a"]);
        }
        b = b.start("r0");
        let mut g = b.build();
        let before = g.all_rules().count();
        g.normalize();
        prop_assert!(g.all_rules().count() >= before, "normalize must not drop rules");
    }

    #[test]
    fn prop_find_symbol_round_trip(name in "[a-z]{2,8}") {
        let g = GrammarBuilder::new("g")
            .token("t", "t")
            .rule(&name, vec!["t"])
            .start(&name)
            .build();
        let id = g.find_symbol_by_name(&name);
        prop_assert!(id.is_some(), "symbol '{}' must be findable", name);
        let found_name = g.rule_names.get(&id.unwrap());
        prop_assert_eq!(found_name.map(|s| s.as_str()), Some(name.as_str()));
    }

    #[test]
    fn prop_token_names_preserved(tokens in token_list_strategy(6)) {
        let mut b = GrammarBuilder::new("g");
        for (name, pat) in &tokens {
            b = b.token(name, pat);
        }
        let first = &tokens[0].0;
        b = b.rule("s", vec![first]).start("s");
        let g = b.build();
        let tok_names: std::collections::HashSet<_> =
            g.tokens.values().map(|t| t.name.clone()).collect();
        for (name, _) in &tokens {
            prop_assert!(tok_names.contains(name), "token '{}' missing", name);
        }
    }

    #[test]
    fn prop_multiple_lhs_distinct_keys(lhs_count in 2usize..8) {
        let mut b = GrammarBuilder::new("g").token("a", "a");
        for i in 0..lhs_count {
            let lhs = format!("lhs{}", i);
            b = b.rule(&lhs, vec!["a"]);
        }
        b = b.start("lhs0");
        let g = b.build();
        prop_assert!(g.rules.len() >= lhs_count);
    }

    #[test]
    fn prop_normalize_preserves_name_twice(name in "[a-z]{1,10}") {
        let mut g = GrammarBuilder::new(&name)
            .token("x", "x")
            .rule("s", vec!["x"])
            .start("s")
            .build();
        g.normalize();
        g.normalize();
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn prop_no_panic_many_alternatives(k in 1usize..15) {
        let mut b = GrammarBuilder::new("g");
        for i in 0..k {
            let t = format!("t{}", i);
            b = b.token(&t, &t);
        }
        for i in 0..k {
            let t = format!("t{}", i);
            b = b.rule("s", vec![&t]);
        }
        b = b.start("s");
        let _g = b.build(); // must not panic
    }

    #[test]
    fn prop_extras_preserved(extra_count in 1usize..5) {
        let mut b = GrammarBuilder::new("g").token("x", "x");
        for i in 0..extra_count {
            let name = format!("e{}", i);
            b = b.token(&name, &name).extra(&name);
        }
        b = b.rule("s", vec!["x"]).start("s");
        let g = b.build();
        prop_assert_eq!(g.extras.len(), extra_count);
    }
}
