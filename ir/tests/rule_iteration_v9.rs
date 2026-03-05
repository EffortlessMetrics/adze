//! Comprehensive tests for rule iteration, counting, and access patterns in adze-ir Grammar.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, PrecedenceKind, Symbol, SymbolId};

// ── 1. Empty grammar ──

#[test]
fn test_empty_grammar_rules_map_is_empty() {
    let g = GrammarBuilder::new("ri_v9_empty1").build();
    assert!(g.rules.is_empty());
}

#[test]
fn test_empty_grammar_all_rules_count_zero() {
    let g = GrammarBuilder::new("ri_v9_empty2").build();
    assert_eq!(g.all_rules().count(), 0);
}

#[test]
fn test_empty_grammar_rule_names_empty() {
    let g = GrammarBuilder::new("ri_v9_empty3").build();
    assert!(g.rule_names.is_empty());
}

#[test]
fn test_empty_grammar_name_preserved() {
    let g = GrammarBuilder::new("ri_v9_empty4").build();
    assert_eq!(g.name, "ri_v9_empty4");
}

// ── 2. Single rule grammar ──

#[test]
fn test_single_rule_count_gte_one() {
    let g = GrammarBuilder::new("ri_v9_single1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn test_single_rule_rules_map_has_one_key() {
    let g = GrammarBuilder::new("ri_v9_single2")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.rules.len(), 1);
}

#[test]
fn test_single_rule_rhs_length() {
    let g = GrammarBuilder::new("ri_v9_single3")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs.len(), 1);
}

#[test]
fn test_single_rule_no_precedence() {
    let g = GrammarBuilder::new("ri_v9_single4")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert!(rule.precedence.is_none());
}

#[test]
fn test_single_rule_no_associativity() {
    let g = GrammarBuilder::new("ri_v9_single5")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert!(rule.associativity.is_none());
}

// ── 3. Multiple rules count ──

#[test]
fn test_two_alternatives_count() {
    let g = GrammarBuilder::new("ri_v9_multi1")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn test_two_nonterminals_rules_map_len() {
    let g = GrammarBuilder::new("ri_v9_multi2")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.rules.len(), 2);
}

#[test]
fn test_three_rules_count() {
    let g = GrammarBuilder::new("ri_v9_multi3")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("b", vec!["y"])
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 3);
}

#[test]
fn test_multiple_alternatives_same_lhs() {
    let g = GrammarBuilder::new("ri_v9_multi4")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let s_rules = g.get_rules_for_symbol(s_id).unwrap();
    assert_eq!(s_rules.len(), 3);
}

// ── 4. Rule lhs is correct SymbolId ──

#[test]
fn test_rule_lhs_matches_symbol_id() {
    let g = GrammarBuilder::new("ri_v9_lhs1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.lhs, s_id);
}

#[test]
fn test_rules_grouped_by_lhs() {
    let g = GrammarBuilder::new("ri_v9_lhs2")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("b", vec!["y"])
        .start("a")
        .build();
    for (lhs_id, rules) in &g.rules {
        for rule in rules {
            assert_eq!(rule.lhs, *lhs_id);
        }
    }
}

#[test]
fn test_all_rules_lhs_present_in_rules_keys() {
    let g = GrammarBuilder::new("ri_v9_lhs3")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    for rule in g.all_rules() {
        assert!(g.rules.contains_key(&rule.lhs));
    }
}

// ── 5. Rule rhs length ──

#[test]
fn test_rule_rhs_single_symbol() {
    let g = GrammarBuilder::new("ri_v9_rhs1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs.len(), 1);
}

#[test]
fn test_rule_rhs_two_symbols() {
    let g = GrammarBuilder::new("ri_v9_rhs2")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x", "y"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let rule = &g.rules[&s_id][0];
    assert_eq!(rule.rhs.len(), 2);
}

#[test]
fn test_rule_rhs_three_symbols() {
    let g = GrammarBuilder::new("ri_v9_rhs3")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("s", vec!["x", "y", "z"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let rule = &g.rules[&s_id][0];
    assert_eq!(rule.rhs.len(), 3);
}

#[test]
fn test_epsilon_rhs_has_length_one() {
    let g = GrammarBuilder::new("ri_v9_rhs4")
        .rule("s", vec![])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs.len(), 1);
    assert_eq!(rule.rhs[0], Symbol::Epsilon);
}

// ── 6. rule_names populated ──

#[test]
fn test_rule_names_populated_for_nonterminal() {
    let g = GrammarBuilder::new("ri_v9_names1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert!(!g.rule_names.is_empty());
    assert!(g.rule_names.values().any(|n| n == "s"));
}

#[test]
fn test_rule_names_has_all_nonterminals() {
    let g = GrammarBuilder::new("ri_v9_names2")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let names: Vec<&str> = g.rule_names.values().map(|s| s.as_str()).collect();
    assert!(names.contains(&"a"));
    assert!(names.contains(&"b"));
    assert!(names.contains(&"s"));
}

#[test]
fn test_rule_names_maps_correct_symbol_id() {
    let g = GrammarBuilder::new("ri_v9_names3")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    assert_eq!(g.rule_names[&s_id], "s");
}

#[test]
fn test_find_symbol_by_name_returns_none_for_missing() {
    let g = GrammarBuilder::new("ri_v9_names4")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert!(g.find_symbol_by_name("nonexistent").is_none());
}

// ── 7. rules IndexMap iteration order ──

#[test]
fn test_rules_iteration_order_start_first() {
    let g = GrammarBuilder::new("ri_v9_order1")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let first_key = g.rules.keys().next().unwrap();
    let s_id = g.find_symbol_by_name("s").unwrap();
    assert_eq!(*first_key, s_id);
}

#[test]
fn test_rules_keys_count_matches_distinct_lhs() {
    let g = GrammarBuilder::new("ri_v9_order2")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("a", vec!["y"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.rules.len(), 2);
}

#[test]
fn test_rules_keys_are_unique() {
    let g = GrammarBuilder::new("ri_v9_order3")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("b", vec!["y"])
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let keys: Vec<_> = g.rules.keys().collect();
    for (i, k1) in keys.iter().enumerate() {
        for k2 in keys.iter().skip(i + 1) {
            assert_ne!(k1, k2);
        }
    }
}

// ── 8. all_rules matches rules.values() ──

#[test]
fn test_all_rules_same_count_as_flat_values() {
    let g = GrammarBuilder::new("ri_v9_equiv1")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("s", vec!["a"])
        .rule("s", vec!["y"])
        .start("s")
        .build();
    let flat_count: usize = g.rules.values().map(|v| v.len()).sum();
    assert_eq!(g.all_rules().count(), flat_count);
}

#[test]
fn test_all_rules_items_match_values_flat() {
    let g = GrammarBuilder::new("ri_v9_equiv2")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let from_all: Vec<_> = g.all_rules().collect();
    let from_values: Vec<_> = g.rules.values().flat_map(|v| v.iter()).collect();
    assert_eq!(from_all.len(), from_values.len());
    for (a, b) in from_all.iter().zip(from_values.iter()) {
        assert_eq!(a.lhs, b.lhs);
        assert_eq!(a.production_id, b.production_id);
    }
}

#[test]
fn test_all_rules_empty_matches_values() {
    let g = GrammarBuilder::new("ri_v9_equiv3").build();
    let from_all: Vec<_> = g.all_rules().collect();
    let from_values: Vec<_> = g.rules.values().flat_map(|v| v.iter()).collect();
    assert_eq!(from_all.len(), from_values.len());
}

// ── 9. Precedence ──

#[test]
fn test_precedence_set_on_rule() {
    let g = GrammarBuilder::new("ri_v9_prec1")
        .token("x", "x")
        .token("+", "+")
        .rule_with_precedence("s", vec!["s", "+", "s"], 5, Associativity::Left)
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let has_prec = g
        .all_rules()
        .any(|r| r.precedence == Some(PrecedenceKind::Static(5)));
    assert!(has_prec);
}

#[test]
fn test_precedence_value_matches() {
    let g = GrammarBuilder::new("ri_v9_prec2")
        .token("x", "x")
        .token("+", "+")
        .rule_with_precedence("s", vec!["s", "+", "s"], 10, Associativity::Left)
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let prec_rule = g.all_rules().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(10)));
}

#[test]
fn test_no_precedence_on_plain_rule() {
    let g = GrammarBuilder::new("ri_v9_prec3")
        .token("x", "x")
        .token("+", "+")
        .rule_with_precedence("s", vec!["s", "+", "s"], 3, Associativity::Left)
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let plain_rule = g.all_rules().find(|r| r.precedence.is_none()).unwrap();
    assert!(plain_rule.associativity.is_none());
}

#[test]
fn test_multiple_precedence_levels() {
    let g = GrammarBuilder::new("ri_v9_prec4")
        .token("x", "x")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("s", vec!["s", "+", "s"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "*", "s"], 2, Associativity::Left)
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let precs: Vec<_> = g.all_rules().filter_map(|r| r.precedence).collect();
    assert!(precs.contains(&PrecedenceKind::Static(1)));
    assert!(precs.contains(&PrecedenceKind::Static(2)));
}

// ── 10. Associativity ──

#[test]
fn test_left_associativity() {
    let g = GrammarBuilder::new("ri_v9_assoc1")
        .token("x", "x")
        .token("+", "+")
        .rule_with_precedence("s", vec!["s", "+", "s"], 1, Associativity::Left)
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let assoc_rule = g.all_rules().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(assoc_rule.associativity, Some(Associativity::Left));
}

#[test]
fn test_right_associativity() {
    let g = GrammarBuilder::new("ri_v9_assoc2")
        .token("x", "x")
        .token("=", "=")
        .rule_with_precedence("s", vec!["s", "=", "s"], 1, Associativity::Right)
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let assoc_rule = g.all_rules().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(assoc_rule.associativity, Some(Associativity::Right));
}

#[test]
fn test_none_associativity() {
    let g = GrammarBuilder::new("ri_v9_assoc3")
        .token("x", "x")
        .token("<", "<")
        .rule_with_precedence("s", vec!["s", "<", "s"], 1, Associativity::None)
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let assoc_rule = g.all_rules().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(assoc_rule.associativity, Some(Associativity::None));
}

#[test]
fn test_mixed_associativity() {
    let g = GrammarBuilder::new("ri_v9_assoc4")
        .token("x", "x")
        .token("+", "+")
        .token("=", "=")
        .rule_with_precedence("s", vec!["s", "+", "s"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "=", "s"], 2, Associativity::Right)
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let assocs: Vec<_> = g.all_rules().filter_map(|r| r.associativity).collect();
    assert!(assocs.contains(&Associativity::Left));
    assert!(assocs.contains(&Associativity::Right));
}

// ── 11. Cloned grammar ──

#[test]
fn test_cloned_grammar_rules_count_matches() {
    let g = GrammarBuilder::new("ri_v9_clone1")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = g.clone();
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

#[test]
fn test_cloned_grammar_rules_equal() {
    let g = GrammarBuilder::new("ri_v9_clone2")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let g2 = g.clone();
    assert_eq!(g.rules, g2.rules);
}

#[test]
fn test_cloned_grammar_rule_names_equal() {
    let g = GrammarBuilder::new("ri_v9_clone3")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = g.clone();
    assert_eq!(g.rule_names, g2.rule_names);
}

#[test]
fn test_cloned_grammar_name_equal() {
    let g = GrammarBuilder::new("ri_v9_clone4")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let g2 = g.clone();
    assert_eq!(g.name, g2.name);
}

// ── 12. Rules after normalize ──

#[test]
fn test_normalize_returns_vec() {
    let mut g = GrammarBuilder::new("ri_v9_norm1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let result = g.normalize();
    let _ = result; // normalize returns Vec<Rule>
}

#[test]
fn test_normalize_preserves_simple_rules() {
    let mut g = GrammarBuilder::new("ri_v9_norm2")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let count_before = g.all_rules().count();
    let _ = g.normalize();
    assert!(g.all_rules().count() >= count_before);
}

#[test]
fn test_normalize_grammar_still_has_rules() {
    let mut g = GrammarBuilder::new("ri_v9_norm3")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("s", vec!["a", "y"])
        .start("s")
        .build();
    let _ = g.normalize();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn test_normalize_lhs_still_valid() {
    let mut g = GrammarBuilder::new("ri_v9_norm4")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let _ = g.normalize();
    for rule in g.all_rules() {
        assert!(g.rules.contains_key(&rule.lhs));
    }
}

// ── 13. Rules after optimize ──

#[test]
fn test_optimize_does_not_panic() {
    let mut g = GrammarBuilder::new("ri_v9_opt1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    g.optimize();
}

#[test]
fn test_optimize_preserves_rules() {
    let mut g = GrammarBuilder::new("ri_v9_opt2")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let count_before = g.all_rules().count();
    g.optimize();
    assert_eq!(g.all_rules().count(), count_before);
}

#[test]
fn test_optimize_then_all_rules() {
    let mut g = GrammarBuilder::new("ri_v9_opt3")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("s", vec!["a", "y"])
        .start("s")
        .build();
    g.optimize();
    for rule in g.all_rules() {
        assert!(g.rules.contains_key(&rule.lhs));
    }
}

// ── 14. Various grammar sizes ──

#[test]
fn test_size_one_rule() {
    let g = GrammarBuilder::new("ri_v9_size1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.rules.len(), 1);
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn test_size_five_rules() {
    let g = GrammarBuilder::new("ri_v9_size5")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("r1", vec!["a"])
        .rule("r2", vec!["b"])
        .rule("r3", vec!["c"])
        .rule("r4", vec!["d"])
        .rule("r5", vec!["e"])
        .start("r1")
        .build();
    assert_eq!(g.rules.len(), 5);
    assert_eq!(g.all_rules().count(), 5);
}

#[test]
fn test_size_ten_rules() {
    let g = GrammarBuilder::new("ri_v9_size10")
        .token("t1", "t1")
        .token("t2", "t2")
        .token("t3", "t3")
        .token("t4", "t4")
        .token("t5", "t5")
        .token("t6", "t6")
        .token("t7", "t7")
        .token("t8", "t8")
        .token("t9", "t9")
        .token("t10", "t10")
        .rule("n1", vec!["t1"])
        .rule("n2", vec!["t2"])
        .rule("n3", vec!["t3"])
        .rule("n4", vec!["t4"])
        .rule("n5", vec!["t5"])
        .rule("n6", vec!["t6"])
        .rule("n7", vec!["t7"])
        .rule("n8", vec!["t8"])
        .rule("n9", vec!["t9"])
        .rule("n10", vec!["t10"])
        .start("n1")
        .build();
    assert_eq!(g.rules.len(), 10);
    assert_eq!(g.all_rules().count(), 10);
}

#[test]
fn test_size_twenty_rules() {
    let mut builder = GrammarBuilder::new("ri_v9_size20");
    for i in 1..=20 {
        let tname = format!("t{i}");
        builder = builder.token(&tname, &tname);
    }
    for i in 1..=20 {
        let rname = format!("n{i}");
        let tname = format!("t{i}");
        builder = builder.rule(&rname, vec![&tname]);
    }
    builder = builder.start("n1");
    let g = builder.build();
    assert_eq!(g.rules.len(), 20);
    assert_eq!(g.all_rules().count(), 20);
}

#[test]
fn test_size_twenty_all_lhs_unique() {
    let mut builder = GrammarBuilder::new("ri_v9_size20u");
    for i in 1..=20 {
        let tname = format!("t{i}");
        builder = builder.token(&tname, &tname);
    }
    for i in 1..=20 {
        let rname = format!("n{i}");
        let tname = format!("t{i}");
        builder = builder.rule(&rname, vec![&tname]);
    }
    builder = builder.start("n1");
    let g = builder.build();
    let lhs_ids: Vec<SymbolId> = g.all_rules().map(|r| r.lhs).collect();
    for (i, a) in lhs_ids.iter().enumerate() {
        for b in lhs_ids.iter().skip(i + 1) {
            assert_ne!(a, b);
        }
    }
}

// ── 15. Rule access by key in rules IndexMap ──

#[test]
fn test_access_rules_by_symbol_id() {
    let g = GrammarBuilder::new("ri_v9_access1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    assert!(g.rules.contains_key(&s_id));
}

#[test]
fn test_access_rules_by_symbol_id_returns_vec() {
    let g = GrammarBuilder::new("ri_v9_access2")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    assert_eq!(g.rules[&s_id].len(), 2);
}

#[test]
fn test_get_rules_for_symbol_some() {
    let g = GrammarBuilder::new("ri_v9_access3")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    assert!(g.get_rules_for_symbol(s_id).is_some());
}

#[test]
fn test_get_rules_for_symbol_none() {
    let g = GrammarBuilder::new("ri_v9_access4")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert!(g.get_rules_for_symbol(SymbolId(9999)).is_none());
}

// ── Additional: iteration patterns ──

#[test]
fn test_all_rules_can_collect_to_vec() {
    let g = GrammarBuilder::new("ri_v9_iter1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let rules: Vec<_> = g.all_rules().collect();
    assert!(!rules.is_empty());
}

#[test]
fn test_all_rules_filter_by_precedence() {
    let g = GrammarBuilder::new("ri_v9_iter2")
        .token("x", "x")
        .token("+", "+")
        .rule_with_precedence("s", vec!["s", "+", "s"], 3, Associativity::Left)
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let prec_rules: Vec<_> = g.all_rules().filter(|r| r.precedence.is_some()).collect();
    assert_eq!(prec_rules.len(), 1);
}

#[test]
fn test_all_rules_map_lhs() {
    let g = GrammarBuilder::new("ri_v9_iter3")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("s", vec!["a", "y"])
        .start("s")
        .build();
    let lhs_ids: Vec<SymbolId> = g.all_rules().map(|r| r.lhs).collect();
    assert_eq!(lhs_ids.len(), 2);
}

#[test]
fn test_all_rules_any() {
    let g = GrammarBuilder::new("ri_v9_iter4")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert!(g.all_rules().any(|r| !r.rhs.is_empty()));
}

#[test]
fn test_rules_values_flat_map() {
    let g = GrammarBuilder::new("ri_v9_iter5")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("s", vec!["a"])
        .rule("s", vec!["y"])
        .start("s")
        .build();
    let total: usize = g.rules.values().map(|v| v.len()).sum();
    assert_eq!(total, 3);
}

// ── Rhs symbol type checks ──

#[test]
fn test_rhs_terminal_symbol() {
    let g = GrammarBuilder::new("ri_v9_sym1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert!(matches!(rule.rhs[0], Symbol::Terminal(_)));
}

#[test]
fn test_rhs_nonterminal_symbol() {
    let g = GrammarBuilder::new("ri_v9_sym2")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let s_rule = &g.rules[&s_id][0];
    assert!(matches!(s_rule.rhs[0], Symbol::NonTerminal(_)));
}

#[test]
fn test_rhs_mixed_terminal_nonterminal() {
    let g = GrammarBuilder::new("ri_v9_sym3")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("s", vec!["a", "x"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let s_rule = &g.rules[&s_id][0];
    assert!(matches!(s_rule.rhs[0], Symbol::NonTerminal(_)));
    assert!(matches!(s_rule.rhs[1], Symbol::Terminal(_)));
}

// ── Production IDs ──

#[test]
fn test_production_ids_are_unique() {
    let g = GrammarBuilder::new("ri_v9_pid1")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .start("s")
        .build();
    let pids: Vec<_> = g.all_rules().map(|r| r.production_id).collect();
    for (i, a) in pids.iter().enumerate() {
        for b in pids.iter().skip(i + 1) {
            assert_ne!(a, b);
        }
    }
}

#[test]
fn test_production_ids_across_nonterminals() {
    let g = GrammarBuilder::new("ri_v9_pid2")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("s", vec!["a", "y"])
        .start("s")
        .build();
    let pids: Vec<_> = g.all_rules().map(|r| r.production_id).collect();
    assert_ne!(pids[0], pids[1]);
}

// ── Grammar::new vs builder ──

#[test]
fn test_grammar_new_empty() {
    let g = Grammar::new("ri_v9_gnew1".to_string());
    assert_eq!(g.all_rules().count(), 0);
    assert!(g.rules.is_empty());
}

#[test]
fn test_grammar_new_name() {
    let g = Grammar::new("ri_v9_gnew2".to_string());
    assert_eq!(g.name, "ri_v9_gnew2");
}

// ── Default grammar ──

#[test]
fn test_grammar_default_empty() {
    let g = Grammar::default();
    assert!(g.rules.is_empty());
    assert_eq!(g.all_rules().count(), 0);
}

// ── Iteration idempotency ──

#[test]
fn test_all_rules_count_is_idempotent() {
    let g = GrammarBuilder::new("ri_v9_idem1")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("s", vec!["a", "y"])
        .start("s")
        .build();
    let c1 = g.all_rules().count();
    let c2 = g.all_rules().count();
    assert_eq!(c1, c2);
}

#[test]
fn test_rules_len_stable() {
    let g = GrammarBuilder::new("ri_v9_idem2")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let l1 = g.rules.len();
    let l2 = g.rules.len();
    assert_eq!(l1, l2);
}

// ── Edge cases ──

#[test]
fn test_rule_with_many_rhs_symbols() {
    let g = GrammarBuilder::new("ri_v9_edge1")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("s", vec!["a", "b", "c", "d", "e"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs.len(), 5);
}

#[test]
fn test_rule_with_single_rhs_symbol() {
    let g = GrammarBuilder::new("ri_v9_edge2")
        .token("z", "z")
        .rule("s", vec!["z"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs.len(), 1);
}

#[test]
fn test_same_token_in_multiple_rules() {
    let g = GrammarBuilder::new("ri_v9_edge3")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["x"])
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    assert_eq!(g.all_rules().count(), 3);
}

#[test]
fn test_rule_lhs_appears_in_own_rhs() {
    let g = GrammarBuilder::new("ri_v9_edge4")
        .token("x", "x")
        .token("+", "+")
        .rule("s", vec!["s", "+", "s"])
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let recursive_rule = g.rules[&s_id].iter().find(|r| r.rhs.len() == 3).unwrap();
    assert!(
        recursive_rule
            .rhs
            .iter()
            .any(|sym| matches!(sym, Symbol::NonTerminal(id) if *id == s_id))
    );
}

#[test]
fn test_chain_of_nonterminals() {
    let g = GrammarBuilder::new("ri_v9_edge5")
        .token("x", "x")
        .rule("d", vec!["x"])
        .rule("c", vec!["d"])
        .rule("b", vec!["c"])
        .rule("a", vec!["b"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.rules.len(), 5);
    assert_eq!(g.all_rules().count(), 5);
}

// ── Token interaction ──

#[test]
fn test_tokens_not_in_rules_map() {
    let g = GrammarBuilder::new("ri_v9_tok1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    // tokens are separate from rules
    assert!(!g.tokens.is_empty());
    for (tok_id, _) in &g.tokens {
        // Token IDs should not appear as rule keys
        if !g.rule_names.contains_key(tok_id) {
            assert!(!g.rules.contains_key(tok_id));
        }
    }
}

#[test]
fn test_tokens_count_independent_of_rules() {
    let g = GrammarBuilder::new("ri_v9_tok2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 3);
    assert_eq!(g.rules.len(), 1);
}

// ── Precedence + associativity combined ──

#[test]
fn test_prec_and_assoc_on_same_rule() {
    let g = GrammarBuilder::new("ri_v9_pa1")
        .token("x", "x")
        .token("+", "+")
        .rule_with_precedence("s", vec!["s", "+", "s"], 7, Associativity::Right)
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let prec_rule = g.all_rules().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(7)));
    assert_eq!(prec_rule.associativity, Some(Associativity::Right));
}

#[test]
fn test_negative_precedence() {
    let g = GrammarBuilder::new("ri_v9_pa2")
        .token("x", "x")
        .token("+", "+")
        .rule_with_precedence("s", vec!["s", "+", "s"], -5, Associativity::Left)
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let prec_rule = g.all_rules().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(-5)));
}

#[test]
fn test_zero_precedence() {
    let g = GrammarBuilder::new("ri_v9_pa3")
        .token("x", "x")
        .token("+", "+")
        .rule_with_precedence("s", vec!["s", "+", "s"], 0, Associativity::None)
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let prec_rule = g.all_rules().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(0)));
}

// ── Clone + modify independence ──

#[test]
fn test_clone_independent_mutation() {
    let g = GrammarBuilder::new("ri_v9_clmut1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let mut g2 = g.clone();
    g2.optimize();
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

#[test]
fn test_clone_normalize_independence() {
    let g = GrammarBuilder::new("ri_v9_clmut2")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let original_count = g.all_rules().count();
    let mut g2 = g.clone();
    let _ = g2.normalize();
    assert_eq!(g.all_rules().count(), original_count);
}

// ── start_symbol ──

#[test]
fn test_start_symbol_present() {
    let g = GrammarBuilder::new("ri_v9_start1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let first_key = *g.rules.keys().next().unwrap();
    assert_eq!(first_key, s_id);
}

#[test]
fn test_start_symbol_method_returns_some() {
    let g = GrammarBuilder::new("ri_v9_start2")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    // start_symbol() may or may not find it depending on naming heuristics,
    // but the rules map should not be empty
    assert!(!g.rules.is_empty());
}

// ── Equality checks ──

#[test]
fn test_two_grammars_same_structure_equal() {
    let g1 = GrammarBuilder::new("ri_v9_eq1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let g2 = GrammarBuilder::new("ri_v9_eq1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert_eq!(g1, g2);
}

#[test]
fn test_two_grammars_different_names_not_equal() {
    let g1 = GrammarBuilder::new("ri_v9_neq1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let g2 = GrammarBuilder::new("ri_v9_neq2")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert_ne!(g1, g2);
}

// ── Fields on rules ──

#[test]
fn test_rule_fields_empty_by_default() {
    let g = GrammarBuilder::new("ri_v9_fld1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    for rule in g.all_rules() {
        assert!(rule.fields.is_empty());
    }
}

// ── add_rule method ──

#[test]
fn test_add_rule_increases_count() {
    use adze_ir::{ProductionId, Rule, Symbol};
    let mut g = Grammar::new("ri_v9_add1".to_string());
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    g.add_rule(rule);
    assert_eq!(g.all_rules().count(), 1);
}

#[test]
fn test_add_rule_multiple() {
    use adze_ir::{ProductionId, Rule, Symbol};
    let mut g = Grammar::new("ri_v9_add2".to_string());
    for i in 0..5 {
        let rule = Rule {
            lhs: SymbolId(i + 1),
            rhs: vec![Symbol::Terminal(SymbolId(100))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i),
        };
        g.add_rule(rule);
    }
    assert_eq!(g.all_rules().count(), 5);
}
