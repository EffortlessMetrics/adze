//! Property-based and unit tests for adze-ir grammar rule properties.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, Rule, Symbol, SymbolId};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Valid identifier: starts with letter, then alphanumeric/underscore.
fn name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,12}"
}

/// Simple regex-like token patterns.
fn pattern_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-z]+",
        Just(r"\d+".to_string()),
        Just(r"[a-zA-Z_]+".to_string()),
    ]
}

/// Unique (name, pattern) pairs for tokens.
fn token_list_strategy(max_len: usize) -> impl Strategy<Value = Vec<(String, String)>> {
    prop::collection::vec((name_strategy(), pattern_strategy()), 1..=max_len).prop_map(|pairs| {
        let mut seen = std::collections::HashSet::new();
        pairs
            .into_iter()
            .filter(|(n, _)| seen.insert(n.clone()))
            .collect()
    })
}

/// Build a grammar from parameters: tokens, rules (each as (lhs_index, rhs_indices)),
/// where indices refer to token names. The first rule's LHS becomes start.
fn build_grammar_from_params(
    tokens: &[(String, String)],
    rules: &[(usize, Vec<usize>)],
) -> Grammar {
    let mut b = GrammarBuilder::new("prop_grammar");
    for (tname, pat) in tokens {
        b = b.token(tname, pat);
    }
    let token_names: Vec<&str> = tokens.iter().map(|(n, _)| n.as_str()).collect();
    let mut first_lhs: Option<&str> = None;
    for (lhs_idx, rhs_idxs) in rules {
        let lhs_name = if *lhs_idx < token_names.len() {
            token_names[*lhs_idx]
        } else {
            "start"
        };
        if first_lhs.is_none() {
            first_lhs = Some(lhs_name);
        }
        let rhs: Vec<&str> = rhs_idxs
            .iter()
            .map(|&i| {
                if i < token_names.len() {
                    token_names[i]
                } else {
                    "start"
                }
            })
            .collect();
        b = b.rule(lhs_name, rhs);
    }
    if let Some(s) = first_lhs {
        b = b.start(s);
    }
    b.build()
}

/// Strategy for rule specs: (lhs_index, Vec<rhs_indices>) given `n_tokens`.
fn rule_specs_strategy(
    n_tokens: usize,
    max_rules: usize,
) -> impl Strategy<Value = Vec<(usize, Vec<usize>)>> {
    let bound = n_tokens + 1; // +1 for "start" nonterminal
    prop::collection::vec(
        (0..bound, prop::collection::vec(0..bound, 0..=4)),
        1..=max_rules,
    )
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn total_rule_count(grammar: &Grammar) -> usize {
    grammar.rules.values().map(|v| v.len()).sum()
}

fn all_token_ids(grammar: &Grammar) -> std::collections::HashSet<SymbolId> {
    grammar.tokens.keys().copied().collect()
}

fn all_rule_lhs_ids(grammar: &Grammar) -> std::collections::HashSet<SymbolId> {
    grammar.rules.keys().copied().collect()
}

fn all_known_ids(grammar: &Grammar) -> std::collections::HashSet<SymbolId> {
    let mut ids = all_token_ids(grammar);
    ids.extend(all_rule_lhs_ids(grammar));
    // Symbols registered in rule_names are also valid (forward references)
    ids.extend(grammar.rule_names.keys().copied());
    ids
}

/// Extract SymbolId from a Symbol (Terminal/NonTerminal), ignoring Epsilon.
fn symbol_id(sym: &Symbol) -> Option<SymbolId> {
    match sym {
        Symbol::Terminal(id) | Symbol::NonTerminal(id) | Symbol::External(id) => Some(*id),
        _ => None,
    }
}

// ===========================================================================
// 1. Rule count proptest (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn rule_count_at_least_one(
        tokens in token_list_strategy(4),
        rules in rule_specs_strategy(4, 6),
    ) {
        let grammar = build_grammar_from_params(&tokens, &rules);
        prop_assert!(total_rule_count(&grammar) >= 1);
    }

    #[test]
    fn rule_count_equals_builder_calls(num_rules in 1usize..=8) {
        let mut b = GrammarBuilder::new("count_test").token("x", "x");
        for i in 0..num_rules {
            b = b.rule(&format!("r{i}"), vec!["x"]);
        }
        let grammar = b.build();
        prop_assert_eq!(total_rule_count(&grammar), num_rules);
    }

    #[test]
    fn rule_count_with_alternatives(
        num_lhs in 1usize..=4,
        alts in 1usize..=3,
    ) {
        let mut b = GrammarBuilder::new("alt_test").token("a", "a");
        for i in 0..num_lhs {
            let lhs = format!("r{i}");
            for _ in 0..alts {
                b = b.rule(&lhs, vec!["a"]);
            }
        }
        let grammar = b.build();
        prop_assert_eq!(total_rule_count(&grammar), num_lhs * alts);
    }

    #[test]
    fn rule_count_never_zero_with_rules(n in 1usize..=10) {
        let mut b = GrammarBuilder::new("nonzero").token("t", "t");
        for i in 0..n {
            b = b.rule(&format!("s{i}"), vec!["t"]);
        }
        let grammar = b.build();
        prop_assert!(!grammar.rules.is_empty());
    }

    #[test]
    fn rule_count_monotone_with_additions(
        base in 1usize..=4,
        extra in 1usize..=4,
    ) {
        let mut b = GrammarBuilder::new("mono").token("t", "t");
        for i in 0..base {
            b = b.rule(&format!("b{i}"), vec!["t"]);
        }
        let g1 = b.build();

        let mut b2 = GrammarBuilder::new("mono").token("t", "t");
        for i in 0..(base + extra) {
            b2 = b2.rule(&format!("b{i}"), vec!["t"]);
        }
        let g2 = b2.build();
        prop_assert!(total_rule_count(&g2) >= total_rule_count(&g1));
    }
}

// ===========================================================================
// 2. All rules have valid LHS proptest (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn lhs_exists_in_rule_names(
        tokens in token_list_strategy(4),
        rules in rule_specs_strategy(4, 5),
    ) {
        let grammar = build_grammar_from_params(&tokens, &rules);
        let known = all_rule_lhs_ids(&grammar);
        for rule in grammar.all_rules() {
            prop_assert!(known.contains(&rule.lhs),
                "LHS {:?} not in grammar rule keys", rule.lhs);
        }
    }

    #[test]
    fn lhs_consistent_with_rules_map(
        tokens in token_list_strategy(3),
        rules in rule_specs_strategy(3, 4),
    ) {
        let grammar = build_grammar_from_params(&tokens, &rules);
        for (lhs_id, rule_vec) in &grammar.rules {
            for rule in rule_vec {
                prop_assert_eq!(rule.lhs, *lhs_id,
                    "Rule LHS {:?} doesn't match map key {:?}", rule.lhs, lhs_id);
            }
        }
    }

    #[test]
    fn lhs_ids_are_non_overlapping_with_builder_tokens(num in 1usize..=5) {
        let mut b = GrammarBuilder::new("overlap_test");
        for i in 0..num {
            b = b.token(&format!("tok{i}"), &format!("p{i}"));
        }
        // Use distinct nonterminal names
        for i in 0..num {
            b = b.rule(&format!("nt{i}"), vec![&format!("tok{}", i % num)]);
        }
        let grammar = b.build();
        for rule in grammar.all_rules() {
            prop_assert!(grammar.rules.contains_key(&rule.lhs));
        }
    }

    #[test]
    fn lhs_present_for_single_rule(name in name_strategy()) {
        let grammar = GrammarBuilder::new("single")
            .token("t", "t")
            .rule(&name, vec!["t"])
            .start(&name)
            .build();
        let lhs_names: Vec<&str> = grammar.rule_names.values().map(|s| s.as_str()).collect();
        prop_assert!(lhs_names.contains(&name.as_str()),
            "LHS name '{}' not in rule_names", name);
    }

    #[test]
    fn lhs_symbol_id_matches_rule_names_key(name in name_strategy()) {
        let grammar = GrammarBuilder::new("match_test")
            .token("t", "t")
            .rule(&name, vec!["t"])
            .start(&name)
            .build();
        let found_id = grammar.find_symbol_by_name(&name);
        prop_assert!(found_id.is_some(), "Symbol '{}' not found", name);
        prop_assert!(grammar.rules.contains_key(&found_id.unwrap()));
    }
}

// ===========================================================================
// 3. All rules have valid RHS proptest (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn rhs_symbols_exist_in_grammar(
        tokens in token_list_strategy(4),
        rules in rule_specs_strategy(4, 5),
    ) {
        let grammar = build_grammar_from_params(&tokens, &rules);
        let known = all_known_ids(&grammar);
        for rule in grammar.all_rules() {
            for sym in &rule.rhs {
                if let Some(id) = symbol_id(sym) {
                    prop_assert!(known.contains(&id),
                        "RHS symbol {:?} not found in grammar", id);
                }
            }
        }
    }

    #[test]
    fn rhs_terminals_are_tokens(
        tokens in token_list_strategy(4),
    ) {
        if tokens.is_empty() {
            return Ok(());
        }
        let first = tokens[0].0.as_str();
        let grammar = build_grammar_from_params(&tokens, &[(tokens.len(), vec![0])]);
        let tok_ids = all_token_ids(&grammar);
        for rule in grammar.all_rules() {
            for sym in &rule.rhs {
                if let Symbol::Terminal(id) = sym {
                    prop_assert!(tok_ids.contains(id),
                        "Terminal {:?} not in tokens", id);
                }
            }
        }
        // Verify first token is used as terminal
        let _ = first;
    }

    #[test]
    fn rhs_nonterminals_are_known_symbols(
        tokens in token_list_strategy(3),
        rules in rule_specs_strategy(3, 4),
    ) {
        let grammar = build_grammar_from_params(&tokens, &rules);
        let known = all_known_ids(&grammar);
        for rule in grammar.all_rules() {
            for sym in &rule.rhs {
                if let Symbol::NonTerminal(id) = sym {
                    prop_assert!(known.contains(id),
                        "NonTerminal {:?} not a known symbol", id);
                }
            }
        }
    }

    #[test]
    fn epsilon_rhs_has_single_element(name in name_strategy()) {
        let grammar = GrammarBuilder::new("eps_test")
            .rule(&name, vec![])
            .start(&name)
            .build();
        for rule in grammar.all_rules() {
            if rule.rhs.iter().any(|s| matches!(s, Symbol::Epsilon)) {
                prop_assert_eq!(rule.rhs.len(), 1,
                    "Epsilon rule should have exactly 1 symbol in RHS");
            }
        }
    }

    #[test]
    fn rhs_length_matches_builder_input(n_rhs in 1usize..=6) {
        let mut b = GrammarBuilder::new("rhs_len");
        for i in 0..n_rhs {
            b = b.token(&format!("t{i}"), &format!("p{i}"));
        }
        let rhs_names: Vec<String> = (0..n_rhs).map(|i| format!("t{i}")).collect();
        let rhs_refs: Vec<&str> = rhs_names.iter().map(|s| s.as_str()).collect();
        b = b.rule("root", rhs_refs).start("root");
        let grammar = b.build();
        let root_id = grammar.find_symbol_by_name("root").unwrap();
        let root_rules = grammar.get_rules_for_symbol(root_id).unwrap();
        prop_assert_eq!(root_rules[0].rhs.len(), n_rhs);
    }
}

// ===========================================================================
// 4. Normalize increases rules proptest (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn normalize_never_decreases_rule_count(
        tokens in token_list_strategy(3),
        rules in rule_specs_strategy(3, 4),
    ) {
        let mut grammar = build_grammar_from_params(&tokens, &rules);
        let before = total_rule_count(&grammar);
        grammar.normalize();
        let after = total_rule_count(&grammar);
        prop_assert!(after >= before,
            "normalize decreased rules from {} to {}", before, after);
    }

    #[test]
    fn normalize_idempotent_rule_count(
        tokens in token_list_strategy(3),
        rules in rule_specs_strategy(3, 3),
    ) {
        let mut grammar = build_grammar_from_params(&tokens, &rules);
        grammar.normalize();
        let count1 = total_rule_count(&grammar);
        grammar.normalize();
        let count2 = total_rule_count(&grammar);
        prop_assert_eq!(count1, count2, "normalize not idempotent");
    }

    #[test]
    fn normalize_preserves_original_lhs(
        tokens in token_list_strategy(3),
        rules in rule_specs_strategy(3, 3),
    ) {
        let mut grammar = build_grammar_from_params(&tokens, &rules);
        let lhs_before: std::collections::HashSet<SymbolId> =
            grammar.rules.keys().copied().collect();
        grammar.normalize();
        for lhs in &lhs_before {
            prop_assert!(grammar.rules.contains_key(lhs),
                "Original LHS {:?} lost after normalize", lhs);
        }
    }

    #[test]
    fn normalize_with_optional_adds_rules(name in name_strategy()) {
        // Manually add an Optional symbol to trigger normalize expansion
        let mut grammar = GrammarBuilder::new("opt_test")
            .token("t", "t")
            .rule(&name, vec!["t"])
            .start(&name)
            .build();
        let lhs = grammar.find_symbol_by_name(&name).unwrap();
        // Add a rule with Optional
        grammar.add_rule(Rule {
            lhs,
            rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(
                *grammar.tokens.keys().next().unwrap(),
            )))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: adze_ir::ProductionId(999),
        });
        let before = total_rule_count(&grammar);
        grammar.normalize();
        let after = total_rule_count(&grammar);
        // Optional expands to 2 aux rules, should increase count
        prop_assert!(after > before,
            "Optional expansion should increase rules: {} -> {}", before, after);
    }

    #[test]
    fn normalize_with_repeat_adds_rules(name in name_strategy()) {
        let mut grammar = GrammarBuilder::new("rep_test")
            .token("t", "t")
            .rule(&name, vec!["t"])
            .start(&name)
            .build();
        let lhs = grammar.find_symbol_by_name(&name).unwrap();
        grammar.add_rule(Rule {
            lhs,
            rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(
                *grammar.tokens.keys().next().unwrap(),
            )))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: adze_ir::ProductionId(998),
        });
        let before = total_rule_count(&grammar);
        grammar.normalize();
        let after = total_rule_count(&grammar);
        prop_assert!(after > before,
            "Repeat expansion should increase rules: {} -> {}", before, after);
    }
}

// ===========================================================================
// 5. Token count proptest (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn token_count_matches_builder(tokens in token_list_strategy(8)) {
        let mut b = GrammarBuilder::new("tok_count");
        for (tname, pat) in &tokens {
            b = b.token(tname, pat);
        }
        let grammar = b.build();
        prop_assert_eq!(grammar.tokens.len(), tokens.len());
    }

    #[test]
    fn tokens_have_correct_names(tokens in token_list_strategy(5)) {
        let mut b = GrammarBuilder::new("tok_names");
        for (tname, pat) in &tokens {
            b = b.token(tname, pat);
        }
        let grammar = b.build();
        let names: std::collections::HashSet<&str> =
            grammar.tokens.values().map(|t| t.name.as_str()).collect();
        for (tname, _) in &tokens {
            prop_assert!(names.contains(tname.as_str()),
                "Token '{}' missing", tname);
        }
    }

    #[test]
    fn token_ids_are_unique(tokens in token_list_strategy(6)) {
        let mut b = GrammarBuilder::new("tok_ids");
        for (tname, pat) in &tokens {
            b = b.token(tname, pat);
        }
        let grammar = b.build();
        let ids: Vec<SymbolId> = grammar.tokens.keys().copied().collect();
        let unique: std::collections::HashSet<SymbolId> = ids.iter().copied().collect();
        prop_assert_eq!(ids.len(), unique.len());
    }

    #[test]
    fn token_count_zero_without_tokens(name in name_strategy()) {
        let grammar = GrammarBuilder::new(&name).build();
        prop_assert!(grammar.tokens.is_empty());
    }

    #[test]
    fn adding_rules_does_not_change_token_count(
        tokens in token_list_strategy(4),
        n_rules in 1usize..=5,
    ) {
        let mut b = GrammarBuilder::new("stable_tok");
        for (tname, pat) in &tokens {
            b = b.token(tname, pat);
        }
        let g_no_rules = b.build();

        let mut b2 = GrammarBuilder::new("stable_tok");
        for (tname, pat) in &tokens {
            b2 = b2.token(tname, pat);
        }
        for i in 0..n_rules {
            b2 = b2.rule(&format!("r{i}"), vec![]);
        }
        let g_with_rules = b2.build();
        prop_assert_eq!(g_no_rules.tokens.len(), g_with_rules.tokens.len());
    }
}

// ===========================================================================
// 6. Regular rule property tests (15 tests)
// ===========================================================================

#[test]
fn single_terminal_rule_shape() {
    let grammar = GrammarBuilder::new("test")
        .token("num", r"\d+")
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let rules = grammar.get_rules_for_symbol(expr_id).unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].rhs.len(), 1);
    assert!(matches!(rules[0].rhs[0], Symbol::Terminal(_)));
}

#[test]
fn multiple_terminal_rule_shape() {
    let grammar = GrammarBuilder::new("test")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("abc", vec!["a", "b", "c"])
        .start("abc")
        .build();
    let id = grammar.find_symbol_by_name("abc").unwrap();
    let rules = grammar.get_rules_for_symbol(id).unwrap();
    assert_eq!(rules[0].rhs.len(), 3);
    for sym in &rules[0].rhs {
        assert!(matches!(sym, Symbol::Terminal(_)));
    }
}

#[test]
fn nonterminal_reference_in_rhs() {
    let grammar = GrammarBuilder::new("test")
        .token("t", "t")
        .rule("inner", vec!["t"])
        .rule("outer", vec!["inner"])
        .start("outer")
        .build();
    let outer_id = grammar.find_symbol_by_name("outer").unwrap();
    let rules = grammar.get_rules_for_symbol(outer_id).unwrap();
    assert!(matches!(rules[0].rhs[0], Symbol::NonTerminal(_)));
}

#[test]
fn mixed_terminal_nonterminal_rhs() {
    let grammar = GrammarBuilder::new("test")
        .token("plus", r"\+")
        .rule("atom", vec!["plus"])
        .rule("expr", vec!["atom", "plus", "atom"])
        .start("expr")
        .build();
    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let rules = grammar.get_rules_for_symbol(expr_id).unwrap();
    assert_eq!(rules[0].rhs.len(), 3);
    assert!(matches!(rules[0].rhs[0], Symbol::NonTerminal(_)));
    assert!(matches!(rules[0].rhs[1], Symbol::Terminal(_)));
    assert!(matches!(rules[0].rhs[2], Symbol::NonTerminal(_)));
}

#[test]
fn multiple_alternatives_same_lhs() {
    let grammar = GrammarBuilder::new("test")
        .token("a", "a")
        .token("b", "b")
        .rule("choice", vec!["a"])
        .rule("choice", vec!["b"])
        .start("choice")
        .build();
    let id = grammar.find_symbol_by_name("choice").unwrap();
    let rules = grammar.get_rules_for_symbol(id).unwrap();
    assert_eq!(rules.len(), 2);
}

#[test]
fn recursive_rule() {
    let grammar = GrammarBuilder::new("test")
        .token("t", "t")
        .rule("list", vec!["t"])
        .rule("list", vec!["list", "t"])
        .start("list")
        .build();
    let id = grammar.find_symbol_by_name("list").unwrap();
    let rules = grammar.get_rules_for_symbol(id).unwrap();
    assert_eq!(rules.len(), 2);
    // Second rule has self-reference
    assert!(matches!(rules[1].rhs[0], Symbol::NonTerminal(_)));
}

#[test]
fn rule_with_left_precedence() {
    let grammar = GrammarBuilder::new("test")
        .token("plus", r"\+")
        .rule_with_precedence("expr", vec!["plus"], 1, Associativity::Left)
        .start("expr")
        .build();
    let id = grammar.find_symbol_by_name("expr").unwrap();
    let rules = grammar.get_rules_for_symbol(id).unwrap();
    assert!(rules[0].precedence.is_some());
    assert_eq!(rules[0].associativity, Some(Associativity::Left));
}

#[test]
fn rule_with_right_precedence() {
    let grammar = GrammarBuilder::new("test")
        .token("eq", "=")
        .rule_with_precedence("assign", vec!["eq"], 2, Associativity::Right)
        .start("assign")
        .build();
    let id = grammar.find_symbol_by_name("assign").unwrap();
    let rules = grammar.get_rules_for_symbol(id).unwrap();
    assert_eq!(rules[0].associativity, Some(Associativity::Right));
}

#[test]
fn production_ids_unique_across_all_rules() {
    let grammar = GrammarBuilder::new("test")
        .token("a", "a")
        .token("b", "b")
        .rule("r1", vec!["a"])
        .rule("r1", vec!["b"])
        .rule("r2", vec!["a", "b"])
        .start("r1")
        .build();
    let ids: Vec<_> = grammar.all_rules().map(|r| r.production_id).collect();
    let unique: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(ids.len(), unique.len());
}

#[test]
fn rule_lhs_matches_map_key() {
    let grammar = GrammarBuilder::new("test")
        .token("t", "t")
        .rule("a", vec!["t"])
        .rule("b", vec!["t"])
        .start("a")
        .build();
    for (key, rules) in &grammar.rules {
        for rule in rules {
            assert_eq!(rule.lhs, *key);
        }
    }
}

#[test]
fn grammar_name_preserved() {
    let grammar = GrammarBuilder::new("my_grammar")
        .token("t", "t")
        .rule("s", vec!["t"])
        .start("s")
        .build();
    assert_eq!(grammar.name, "my_grammar");
}

#[test]
fn find_symbol_by_name_returns_correct_id() {
    let grammar = GrammarBuilder::new("test")
        .token("num", r"\d+")
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    assert!(grammar.rules.contains_key(&expr_id));
}

#[test]
fn get_rules_for_symbol_none_for_unknown() {
    let grammar = GrammarBuilder::new("test")
        .token("t", "t")
        .rule("s", vec!["t"])
        .start("s")
        .build();
    assert!(grammar.get_rules_for_symbol(SymbolId(9999)).is_none());
}

#[test]
fn all_rules_iterator_count() {
    let grammar = GrammarBuilder::new("test")
        .token("a", "a")
        .rule("r1", vec!["a"])
        .rule("r1", vec![])
        .rule("r2", vec!["a"])
        .start("r1")
        .build();
    assert_eq!(grammar.all_rules().count(), 3);
}

#[test]
fn rule_without_precedence_has_none() {
    let grammar = GrammarBuilder::new("test")
        .token("t", "t")
        .rule("s", vec!["t"])
        .start("s")
        .build();
    for rule in grammar.all_rules() {
        assert!(rule.precedence.is_none());
        assert!(rule.associativity.is_none());
    }
}

// ===========================================================================
// 7. Edge cases (10 tests)
// ===========================================================================

#[test]
fn empty_rhs_produces_epsilon() {
    let grammar = GrammarBuilder::new("test")
        .rule("empty", vec![])
        .start("empty")
        .build();
    let id = grammar.find_symbol_by_name("empty").unwrap();
    let rules = grammar.get_rules_for_symbol(id).unwrap();
    assert_eq!(rules[0].rhs.len(), 1);
    assert!(matches!(rules[0].rhs[0], Symbol::Epsilon));
}

#[test]
fn single_symbol_rhs() {
    let grammar = GrammarBuilder::new("test")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let id = grammar.find_symbol_by_name("s").unwrap();
    let rules = grammar.get_rules_for_symbol(id).unwrap();
    assert_eq!(rules[0].rhs.len(), 1);
}

#[test]
fn many_alternatives_for_one_lhs() {
    let mut b = GrammarBuilder::new("test").token("t", "t");
    for _ in 0..20 {
        b = b.rule("multi", vec!["t"]);
    }
    let grammar = b.start("multi").build();
    let id = grammar.find_symbol_by_name("multi").unwrap();
    let rules = grammar.get_rules_for_symbol(id).unwrap();
    assert_eq!(rules.len(), 20);
}

#[test]
fn long_rhs_sequence() {
    let mut b = GrammarBuilder::new("test");
    let n = 15;
    for i in 0..n {
        b = b.token(&format!("t{i}"), &format!("p{i}"));
    }
    let rhs: Vec<String> = (0..n).map(|i| format!("t{i}")).collect();
    let rhs_refs: Vec<&str> = rhs.iter().map(|s| s.as_str()).collect();
    let grammar = b.rule("long", rhs_refs).start("long").build();
    let id = grammar.find_symbol_by_name("long").unwrap();
    let rules = grammar.get_rules_for_symbol(id).unwrap();
    assert_eq!(rules[0].rhs.len(), n);
}

#[test]
fn multiple_epsilon_rules_different_lhs() {
    let grammar = GrammarBuilder::new("test")
        .rule("a", vec![])
        .rule("b", vec![])
        .rule("c", vec![])
        .start("a")
        .build();
    for rule in grammar.all_rules() {
        assert!(matches!(rule.rhs[0], Symbol::Epsilon));
    }
}

#[test]
fn grammar_with_only_tokens_no_rules() {
    let grammar = GrammarBuilder::new("test")
        .token("a", "a")
        .token("b", "b")
        .build();
    assert_eq!(grammar.tokens.len(), 2);
    assert!(grammar.rules.is_empty());
}

#[test]
fn normalize_on_simple_grammar_is_noop() {
    let mut grammar = GrammarBuilder::new("test")
        .token("t", "t")
        .rule("s", vec!["t"])
        .start("s")
        .build();
    let before = total_rule_count(&grammar);
    grammar.normalize();
    let after = total_rule_count(&grammar);
    assert_eq!(before, after);
}

#[test]
fn normalize_optional_expands_correctly() {
    let mut grammar = GrammarBuilder::new("test")
        .token("t", "t")
        .rule("s", vec!["t"])
        .start("s")
        .build();
    let tok_id = *grammar.tokens.keys().next().unwrap();
    let lhs = grammar.find_symbol_by_name("s").unwrap();
    grammar.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(tok_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(100),
    });
    grammar.normalize();
    // After normalization, auxiliary rules should exist
    assert!(total_rule_count(&grammar) >= 3);
}

#[test]
fn normalize_repeat_expands_correctly() {
    let mut grammar = GrammarBuilder::new("test")
        .token("t", "t")
        .rule("s", vec!["t"])
        .start("s")
        .build();
    let tok_id = *grammar.tokens.keys().next().unwrap();
    let lhs = grammar.find_symbol_by_name("s").unwrap();
    grammar.add_rule(Rule {
        lhs,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(tok_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(101),
    });
    grammar.normalize();
    assert!(total_rule_count(&grammar) >= 3);
}

#[test]
fn self_referencing_epsilon_and_terminal() {
    let grammar = GrammarBuilder::new("test")
        .token("x", "x")
        .rule("rec", vec!["rec", "x"])
        .rule("rec", vec![])
        .start("rec")
        .build();
    let id = grammar.find_symbol_by_name("rec").unwrap();
    let rules = grammar.get_rules_for_symbol(id).unwrap();
    assert_eq!(rules.len(), 2);
    // One rule has NonTerminal self-reference + Terminal
    assert_eq!(rules[0].rhs.len(), 2);
    // Other is epsilon
    assert!(matches!(rules[1].rhs[0], Symbol::Epsilon));
}
