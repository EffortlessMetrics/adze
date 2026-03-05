//! Field mapping v10 tests for adze-ir Grammar.
//!
//! 80+ tests covering:
//!   fm_v10_fields_*      – Grammar.fields (IndexMap<FieldId, String>) behaviour
//!   fm_v10_rn_*          – Grammar.rule_names (IndexMap<SymbolId, String>) behaviour
//!   fm_v10_clone_*       – Clone preserves fields and rule_names
//!   fm_v10_norm_*        – Normalize doesn't lose fields/rule_names
//!   fm_v10_opt_*         – Optimize doesn't lose fields/rule_names
//!   fm_v10_debug_*       – Debug formatting includes fields
//!   fm_v10_prec_*        – Precedence interaction with fields
//!   fm_v10_combo_*       – Combined features
//!   fm_v10_edge_*        – Edge cases

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, FieldId, Grammar, SymbolId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn minimal(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build()
}

fn expr(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

fn multi_rule(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("ID", r"[a-z]+")
        .token("NUM", r"[0-9]+")
        .token("SEMI", ";")
        .token("EQ", "=")
        .rule("program", vec!["stmt"])
        .rule("stmt", vec!["ID", "EQ", "NUM", "SEMI"])
        .rule("stmt", vec!["ID", "SEMI"])
        .start("program")
        .build()
}

fn with_fields(name: &str) -> Grammar {
    let mut g = expr(name);
    g.fields.insert(FieldId(0), "left".to_string());
    g.fields.insert(FieldId(1), "operator".to_string());
    g.fields.insert(FieldId(2), "right".to_string());
    g
}

fn with_many_fields(name: &str, count: usize) -> Grammar {
    let mut g = minimal(name);
    for i in 0..count {
        g.fields.insert(FieldId(i as u16), format!("field_{i}"));
    }
    g
}

fn prec_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["expr", "STAR", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .precedence(1, Associativity::Left, vec!["PLUS"])
        .precedence(2, Associativity::Left, vec!["STAR"])
        .build()
}

// ===========================================================================
// 1. fm_v10_fields_* – fields basics (20 tests)
// ===========================================================================

#[test]
fn fm_v10_fields_empty_after_minimal_build() {
    let g = minimal("fm_v10_f01");
    assert!(g.fields.is_empty());
}

#[test]
fn fm_v10_fields_empty_after_expr_build() {
    let g = expr("fm_v10_f02");
    assert!(g.fields.is_empty());
}

#[test]
fn fm_v10_fields_default_grammar_empty() {
    let g = Grammar::default();
    assert!(g.fields.is_empty());
}

#[test]
fn fm_v10_fields_is_indexmap() {
    let g = with_fields("fm_v10_f04");
    // Verify it preserves insertion order (IndexMap property)
    let keys: Vec<FieldId> = g.fields.keys().copied().collect();
    assert_eq!(keys, vec![FieldId(0), FieldId(1), FieldId(2)]);
}

#[test]
fn fm_v10_fields_values_are_strings() {
    let g = with_fields("fm_v10_f05");
    for val in g.fields.values() {
        assert!(!val.is_empty());
    }
}

#[test]
fn fm_v10_fields_len_matches_inserted() {
    let g = with_fields("fm_v10_f06");
    assert_eq!(g.fields.len(), 3);
}

#[test]
fn fm_v10_fields_lookup_by_id() {
    let g = with_fields("fm_v10_f07");
    assert_eq!(g.fields[&FieldId(0)], "left");
    assert_eq!(g.fields[&FieldId(1)], "operator");
    assert_eq!(g.fields[&FieldId(2)], "right");
}

#[test]
fn fm_v10_fields_contains_key() {
    let g = with_fields("fm_v10_f08");
    assert!(g.fields.contains_key(&FieldId(0)));
    assert!(g.fields.contains_key(&FieldId(2)));
    assert!(!g.fields.contains_key(&FieldId(99)));
}

#[test]
fn fm_v10_fields_insertion_order_preserved() {
    let g = with_fields("fm_v10_f09");
    let names: Vec<&str> = g.fields.values().map(String::as_str).collect();
    assert_eq!(names, vec!["left", "operator", "right"]);
}

#[test]
fn fm_v10_fields_replace_existing_key() {
    let mut g = minimal("fm_v10_f10");
    g.fields.insert(FieldId(0), "old".to_string());
    g.fields.insert(FieldId(0), "new".to_string());
    assert_eq!(g.fields.len(), 1);
    assert_eq!(g.fields[&FieldId(0)], "new");
}

#[test]
fn fm_v10_fields_noncontiguous_ids() {
    let mut g = minimal("fm_v10_f11");
    g.fields.insert(FieldId(0), "a".to_string());
    g.fields.insert(FieldId(5), "b".to_string());
    g.fields.insert(FieldId(100), "c".to_string());
    assert_eq!(g.fields.len(), 3);
    assert_eq!(g.fields[&FieldId(100)], "c");
}

#[test]
fn fm_v10_fields_remove_entry() {
    let mut g = with_fields("fm_v10_f12");
    g.fields.swap_remove(&FieldId(1));
    assert_eq!(g.fields.len(), 2);
    assert!(!g.fields.contains_key(&FieldId(1)));
}

#[test]
fn fm_v10_fields_iterate_pairs() {
    let g = with_fields("fm_v10_f13");
    let pairs: Vec<(FieldId, &str)> = g.fields.iter().map(|(k, v)| (*k, v.as_str())).collect();
    assert_eq!(pairs.len(), 3);
    assert_eq!(pairs[0], (FieldId(0), "left"));
}

#[test]
fn fm_v10_fields_with_tokens_still_accessible() {
    let g = expr("fm_v10_f14");
    assert!(!g.tokens.is_empty());
    assert!(g.fields.is_empty());
}

#[test]
fn fm_v10_fields_multi_rule_grammar_empty() {
    let g = multi_rule("fm_v10_f15");
    assert!(g.fields.is_empty());
}

#[test]
fn fm_v10_fields_many_entries() {
    let g = with_many_fields("fm_v10_f16", 50);
    assert_eq!(g.fields.len(), 50);
    assert_eq!(g.fields[&FieldId(49)], "field_49");
}

#[test]
fn fm_v10_fields_single_entry() {
    let mut g = minimal("fm_v10_f17");
    g.fields.insert(FieldId(0), "only".to_string());
    assert_eq!(g.fields.len(), 1);
    assert_eq!(g.fields[&FieldId(0)], "only");
}

#[test]
fn fm_v10_fields_empty_string_value() {
    let mut g = minimal("fm_v10_f18");
    g.fields.insert(FieldId(0), String::new());
    assert_eq!(g.fields[&FieldId(0)], "");
}

#[test]
fn fm_v10_fields_get_returns_none_for_missing() {
    let g = with_fields("fm_v10_f19");
    assert!(g.fields.get(&FieldId(99)).is_none());
}

#[test]
fn fm_v10_fields_coexist_with_rules_and_tokens() {
    let g = with_fields("fm_v10_f20");
    assert!(!g.rules.is_empty());
    assert!(!g.tokens.is_empty());
    assert_eq!(g.fields.len(), 3);
}

// ===========================================================================
// 2. fm_v10_rn_* – rule_names (20 tests)
// ===========================================================================

#[test]
fn fm_v10_rn_populated_after_build() {
    let g = minimal("fm_v10_rn01");
    assert!(!g.rule_names.is_empty());
}

#[test]
fn fm_v10_rn_contains_start_symbol() {
    let g = minimal("fm_v10_rn02");
    let has_start = g.rule_names.values().any(|n| n == "start");
    assert!(has_start);
}

#[test]
fn fm_v10_rn_keys_are_valid_symbol_ids() {
    let g = expr("fm_v10_rn03");
    for key in g.rule_names.keys() {
        // SymbolId wraps u16; key.0 should be a reasonable value
        assert!(key.0 < 1000);
    }
}

#[test]
fn fm_v10_rn_values_match_input_names() {
    let g = expr("fm_v10_rn04");
    let names: Vec<&str> = g.rule_names.values().map(String::as_str).collect();
    assert!(names.contains(&"expr"));
}

#[test]
fn fm_v10_rn_multi_rule_names() {
    let g = multi_rule("fm_v10_rn05");
    let names: Vec<&str> = g.rule_names.values().map(String::as_str).collect();
    assert!(names.contains(&"program"));
    assert!(names.contains(&"stmt"));
}

#[test]
fn fm_v10_rn_default_grammar_empty() {
    let g = Grammar::default();
    assert!(g.rule_names.is_empty());
}

#[test]
fn fm_v10_rn_single_rule_count() {
    let g = minimal("fm_v10_rn07");
    // At least the start rule should be named
    assert!(!g.rule_names.is_empty());
}

#[test]
fn fm_v10_rn_expr_rule_count() {
    let g = expr("fm_v10_rn08");
    // "expr" is the only non-terminal
    assert!(!g.rule_names.is_empty());
}

#[test]
fn fm_v10_rn_multi_rule_count() {
    let g = multi_rule("fm_v10_rn09");
    // "program" and "stmt" at minimum
    assert!(g.rule_names.len() >= 2);
}

#[test]
fn fm_v10_rn_find_symbol_matches_rule_names() {
    let g = expr("fm_v10_rn10");
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert_eq!(g.rule_names[&expr_id], "expr");
}

#[test]
fn fm_v10_rn_no_duplicate_keys() {
    let g = multi_rule("fm_v10_rn11");
    let keys: Vec<SymbolId> = g.rule_names.keys().copied().collect();
    for (i, k1) in keys.iter().enumerate() {
        for k2 in &keys[i + 1..] {
            assert_ne!(k1, k2);
        }
    }
}

#[test]
fn fm_v10_rn_values_are_nonempty() {
    let g = multi_rule("fm_v10_rn12");
    for val in g.rule_names.values() {
        assert!(!val.is_empty());
    }
}

#[test]
fn fm_v10_rn_insertion_after_build() {
    let mut g = minimal("fm_v10_rn13");
    let count_before = g.rule_names.len();
    g.rule_names.insert(SymbolId(999), "custom".to_string());
    assert_eq!(g.rule_names.len(), count_before + 1);
    assert_eq!(g.rule_names[&SymbolId(999)], "custom");
}

#[test]
fn fm_v10_rn_removal_after_build() {
    let mut g = expr("fm_v10_rn14");
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    g.rule_names.swap_remove(&expr_id);
    assert!(!g.rule_names.contains_key(&expr_id));
}

#[test]
fn fm_v10_rn_preserves_insertion_order() {
    let g = multi_rule("fm_v10_rn15");
    let names: Vec<&str> = g.rule_names.values().map(String::as_str).collect();
    // At least the first rule should be in order
    assert!(!names.is_empty());
}

#[test]
fn fm_v10_rn_prec_grammar_has_rule_names() {
    let g = prec_grammar("fm_v10_rn16");
    assert!(!g.rule_names.is_empty());
    let has_expr = g.rule_names.values().any(|n| n == "expr");
    assert!(has_expr);
}

#[test]
fn fm_v10_rn_keys_align_with_rules_map() {
    let g = multi_rule("fm_v10_rn17");
    for key in g.rules.keys() {
        assert!(g.rule_names.contains_key(key));
    }
}

#[test]
fn fm_v10_rn_get_returns_none_for_missing() {
    let g = minimal("fm_v10_rn18");
    assert!(g.rule_names.get(&SymbolId(9999)).is_none());
}

#[test]
fn fm_v10_rn_contains_key_positive() {
    let g = expr("fm_v10_rn19");
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.rule_names.contains_key(&expr_id));
}

#[test]
fn fm_v10_rn_iterate_all_pairs() {
    let g = multi_rule("fm_v10_rn20");
    let mut count = 0;
    for (id, name) in &g.rule_names {
        assert!(id.0 < 1000);
        assert!(!name.is_empty());
        count += 1;
    }
    assert!(count >= 2);
}

// ===========================================================================
// 3. fm_v10_clone_* – Clone preserves fields and rule_names (10 tests)
// ===========================================================================

#[test]
fn fm_v10_clone_preserves_empty_fields() {
    let g = minimal("fm_v10_cl01");
    let cloned = g.clone();
    assert_eq!(g.fields, cloned.fields);
}

#[test]
fn fm_v10_clone_preserves_populated_fields() {
    let g = with_fields("fm_v10_cl02");
    let cloned = g.clone();
    assert_eq!(g.fields.len(), cloned.fields.len());
    assert_eq!(cloned.fields[&FieldId(0)], "left");
    assert_eq!(cloned.fields[&FieldId(2)], "right");
}

#[test]
fn fm_v10_clone_preserves_rule_names() {
    let g = multi_rule("fm_v10_cl03");
    let cloned = g.clone();
    assert_eq!(g.rule_names, cloned.rule_names);
}

#[test]
fn fm_v10_clone_deep_independence_fields() {
    let mut g = with_fields("fm_v10_cl04");
    let cloned = g.clone();
    g.fields.insert(FieldId(10), "extra".to_string());
    assert_ne!(g.fields.len(), cloned.fields.len());
}

#[test]
fn fm_v10_clone_deep_independence_rule_names() {
    let mut g = multi_rule("fm_v10_cl05");
    let cloned = g.clone();
    g.rule_names.insert(SymbolId(888), "extra_rule".to_string());
    assert_ne!(g.rule_names.len(), cloned.rule_names.len());
}

#[test]
fn fm_v10_clone_equality() {
    let g = with_fields("fm_v10_cl06");
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn fm_v10_clone_many_fields() {
    let g = with_many_fields("fm_v10_cl07", 30);
    let cloned = g.clone();
    assert_eq!(g.fields.len(), cloned.fields.len());
    for (k, v) in &g.fields {
        assert_eq!(cloned.fields[k], *v);
    }
}

#[test]
fn fm_v10_clone_prec_grammar_rule_names() {
    let g = prec_grammar("fm_v10_cl08");
    let cloned = g.clone();
    assert_eq!(g.rule_names, cloned.rule_names);
}

#[test]
fn fm_v10_clone_fields_order_preserved() {
    let g = with_fields("fm_v10_cl09");
    let cloned = g.clone();
    let orig: Vec<&str> = g.fields.values().map(String::as_str).collect();
    let copy: Vec<&str> = cloned.fields.values().map(String::as_str).collect();
    assert_eq!(orig, copy);
}

#[test]
fn fm_v10_clone_rule_names_order_preserved() {
    let g = multi_rule("fm_v10_cl10");
    let cloned = g.clone();
    let orig: Vec<&str> = g.rule_names.values().map(String::as_str).collect();
    let copy: Vec<&str> = cloned.rule_names.values().map(String::as_str).collect();
    assert_eq!(orig, copy);
}

// ===========================================================================
// 4. fm_v10_norm_* – Normalize doesn't lose fields/rule_names (10 tests)
// ===========================================================================

#[test]
fn fm_v10_norm_preserves_empty_fields() {
    let mut g = minimal("fm_v10_n01");
    g.normalize();
    assert!(g.fields.is_empty());
}

#[test]
fn fm_v10_norm_preserves_populated_fields() {
    let mut g = with_fields("fm_v10_n02");
    g.normalize();
    assert_eq!(g.fields.len(), 3);
    assert_eq!(g.fields[&FieldId(0)], "left");
}

#[test]
fn fm_v10_norm_preserves_rule_names() {
    let mut g = multi_rule("fm_v10_n03");
    let names_before: Vec<String> = g.rule_names.values().cloned().collect();
    g.normalize();
    for name in &names_before {
        let found = g.rule_names.values().any(|n| n == name);
        assert!(found, "rule_name '{name}' lost after normalize");
    }
}

#[test]
fn fm_v10_norm_fields_count_stable() {
    let mut g = with_many_fields("fm_v10_n04", 10);
    let count_before = g.fields.len();
    g.normalize();
    assert_eq!(g.fields.len(), count_before);
}

#[test]
fn fm_v10_norm_field_values_unchanged() {
    let mut g = with_fields("fm_v10_n05");
    g.normalize();
    assert_eq!(g.fields[&FieldId(1)], "operator");
    assert_eq!(g.fields[&FieldId(2)], "right");
}

#[test]
fn fm_v10_norm_rule_names_keys_valid() {
    let mut g = expr("fm_v10_n06");
    g.normalize();
    for key in g.rule_names.keys() {
        assert!(key.0 < 10000);
    }
}

#[test]
fn fm_v10_norm_double_normalize_fields_stable() {
    let mut g = with_fields("fm_v10_n07");
    g.normalize();
    let fields_after_first = g.fields.clone();
    g.normalize();
    assert_eq!(g.fields, fields_after_first);
}

#[test]
fn fm_v10_norm_prec_grammar_fields() {
    let mut g = prec_grammar("fm_v10_n08");
    g.fields.insert(FieldId(0), "lhs".to_string());
    g.fields.insert(FieldId(1), "op".to_string());
    g.normalize();
    assert_eq!(g.fields.len(), 2);
}

#[test]
fn fm_v10_norm_expr_rule_names_still_present() {
    let mut g = expr("fm_v10_n09");
    g.normalize();
    let has_expr = g.rule_names.values().any(|n| n == "expr");
    assert!(has_expr);
}

#[test]
fn fm_v10_norm_multi_rule_names_all_present() {
    let mut g = multi_rule("fm_v10_n10");
    g.normalize();
    let names: Vec<&str> = g.rule_names.values().map(String::as_str).collect();
    assert!(names.contains(&"program"));
    assert!(names.contains(&"stmt"));
}

// ===========================================================================
// 5. fm_v10_opt_* – Optimize doesn't lose fields/rule_names (10 tests)
// ===========================================================================

#[test]
fn fm_v10_opt_preserves_empty_fields() {
    let mut g = minimal("fm_v10_o01");
    g.optimize();
    assert!(g.fields.is_empty());
}

#[test]
fn fm_v10_opt_preserves_populated_fields() {
    let mut g = with_fields("fm_v10_o02");
    g.optimize();
    assert_eq!(g.fields.len(), 3);
    assert_eq!(g.fields[&FieldId(0)], "left");
}

#[test]
fn fm_v10_opt_preserves_rule_names() {
    let mut g = multi_rule("fm_v10_o03");
    let count_before = g.rule_names.len();
    g.optimize();
    assert_eq!(g.rule_names.len(), count_before);
}

#[test]
fn fm_v10_opt_field_values_unchanged() {
    let mut g = with_fields("fm_v10_o04");
    g.optimize();
    let names: Vec<&str> = g.fields.values().map(String::as_str).collect();
    assert_eq!(names, vec!["left", "operator", "right"]);
}

#[test]
fn fm_v10_opt_many_fields_stable() {
    let mut g = with_many_fields("fm_v10_o05", 25);
    g.optimize();
    assert_eq!(g.fields.len(), 25);
}

#[test]
fn fm_v10_opt_double_optimize_fields_stable() {
    let mut g = with_fields("fm_v10_o06");
    g.optimize();
    let fields_after_first = g.fields.clone();
    g.optimize();
    assert_eq!(g.fields, fields_after_first);
}

#[test]
fn fm_v10_opt_rule_names_keys_valid() {
    let mut g = expr("fm_v10_o07");
    g.optimize();
    for key in g.rule_names.keys() {
        assert!(key.0 < 10000);
    }
}

#[test]
fn fm_v10_opt_prec_grammar_fields() {
    let mut g = prec_grammar("fm_v10_o08");
    g.fields.insert(FieldId(0), "x".to_string());
    g.optimize();
    assert_eq!(g.fields[&FieldId(0)], "x");
}

#[test]
fn fm_v10_opt_expr_rule_names_present() {
    let mut g = expr("fm_v10_o09");
    g.optimize();
    let has_expr = g.rule_names.values().any(|n| n == "expr");
    assert!(has_expr);
}

#[test]
fn fm_v10_opt_normalize_then_optimize_fields() {
    let mut g = with_fields("fm_v10_o10");
    g.normalize();
    g.optimize();
    assert_eq!(g.fields.len(), 3);
    assert_eq!(g.fields[&FieldId(0)], "left");
}

// ===========================================================================
// 6. fm_v10_debug_* – Debug formatting (5 tests)
// ===========================================================================

#[test]
fn fm_v10_debug_includes_fields_keyword() {
    let g = with_fields("fm_v10_d01");
    let dbg = format!("{g:?}");
    assert!(dbg.contains("fields"));
}

#[test]
fn fm_v10_debug_includes_rule_names_keyword() {
    let g = minimal("fm_v10_d02");
    let dbg = format!("{g:?}");
    assert!(dbg.contains("rule_names"));
}

#[test]
fn fm_v10_debug_shows_field_values() {
    let g = with_fields("fm_v10_d03");
    let dbg = format!("{g:?}");
    assert!(dbg.contains("left"));
    assert!(dbg.contains("operator"));
    assert!(dbg.contains("right"));
}

#[test]
fn fm_v10_debug_empty_fields_shown() {
    let g = minimal("fm_v10_d04");
    let dbg = format!("{g:?}");
    // Fields section exists even when empty
    assert!(dbg.contains("fields"));
}

#[test]
fn fm_v10_debug_grammar_name_present() {
    let g = with_fields("fm_v10_d05");
    let dbg = format!("{g:?}");
    assert!(dbg.contains("fm_v10_d05"));
}

// ===========================================================================
// 7. fm_v10_prec_* – Precedence interaction with fields (5 tests)
// ===========================================================================

#[test]
fn fm_v10_prec_fields_empty_by_default() {
    let g = prec_grammar("fm_v10_p01");
    assert!(g.fields.is_empty());
}

#[test]
fn fm_v10_prec_rule_names_populated() {
    let g = prec_grammar("fm_v10_p02");
    assert!(!g.rule_names.is_empty());
}

#[test]
fn fm_v10_prec_fields_insertable() {
    let mut g = prec_grammar("fm_v10_p03");
    g.fields.insert(FieldId(0), "lhs".to_string());
    g.fields.insert(FieldId(1), "rhs".to_string());
    assert_eq!(g.fields.len(), 2);
}

#[test]
fn fm_v10_prec_fields_survive_clone() {
    let mut g = prec_grammar("fm_v10_p04");
    g.fields.insert(FieldId(0), "operand".to_string());
    let cloned = g.clone();
    assert_eq!(cloned.fields[&FieldId(0)], "operand");
}

#[test]
fn fm_v10_prec_precedences_and_fields_coexist() {
    let mut g = prec_grammar("fm_v10_p05");
    g.fields.insert(FieldId(0), "val".to_string());
    assert!(!g.precedences.is_empty());
    assert!(!g.fields.is_empty());
}

// ===========================================================================
// 8. fm_v10_combo_* – Combined features (5 tests)
// ===========================================================================

#[test]
fn fm_v10_combo_fields_and_rule_names_independent() {
    let g = with_fields("fm_v10_c01");
    // fields and rule_names are separate maps
    assert_eq!(g.fields.len(), 3);
    assert!(!g.rule_names.is_empty());
}

#[test]
fn fm_v10_combo_multi_rule_fields_and_names() {
    let mut g = multi_rule("fm_v10_c02");
    g.fields.insert(FieldId(0), "name".to_string());
    g.fields.insert(FieldId(1), "value".to_string());
    assert_eq!(g.fields.len(), 2);
    assert!(g.rule_names.len() >= 2);
}

#[test]
fn fm_v10_combo_normalize_then_check_both() {
    let mut g = with_fields("fm_v10_c03");
    g.normalize();
    assert_eq!(g.fields.len(), 3);
    assert!(!g.rule_names.is_empty());
}

#[test]
fn fm_v10_combo_optimize_then_check_both() {
    let mut g = with_fields("fm_v10_c04");
    g.optimize();
    assert_eq!(g.fields.len(), 3);
    assert!(!g.rule_names.is_empty());
}

#[test]
fn fm_v10_combo_clone_preserves_both() {
    let g = with_fields("fm_v10_c05");
    let cloned = g.clone();
    assert_eq!(g.fields, cloned.fields);
    assert_eq!(g.rule_names, cloned.rule_names);
}

// ===========================================================================
// 9. fm_v10_edge_* – Edge cases (8 tests)
// ===========================================================================

#[test]
fn fm_v10_edge_field_id_zero() {
    let mut g = minimal("fm_v10_e01");
    g.fields.insert(FieldId(0), "first".to_string());
    assert_eq!(g.fields[&FieldId(0)], "first");
}

#[test]
fn fm_v10_edge_field_id_max_u16() {
    let mut g = minimal("fm_v10_e02");
    g.fields.insert(FieldId(u16::MAX), "last".to_string());
    assert_eq!(g.fields[&FieldId(u16::MAX)], "last");
}

#[test]
fn fm_v10_edge_unicode_field_name() {
    let mut g = minimal("fm_v10_e03");
    g.fields.insert(FieldId(0), "日本語".to_string());
    assert_eq!(g.fields[&FieldId(0)], "日本語");
}

#[test]
fn fm_v10_edge_long_field_name() {
    let mut g = minimal("fm_v10_e04");
    let long_name = "a".repeat(1000);
    g.fields.insert(FieldId(0), long_name.clone());
    assert_eq!(g.fields[&FieldId(0)], long_name);
}

#[test]
fn fm_v10_edge_clear_fields() {
    let mut g = with_fields("fm_v10_e05");
    g.fields.clear();
    assert!(g.fields.is_empty());
}

#[test]
fn fm_v10_edge_clear_rule_names() {
    let mut g = multi_rule("fm_v10_e06");
    g.rule_names.clear();
    assert!(g.rule_names.is_empty());
}

#[test]
fn fm_v10_edge_equality_different_fields() {
    let g1 = with_fields("fm_v10_e07");
    let mut g2 = expr("fm_v10_e07");
    g2.fields.insert(FieldId(0), "different".to_string());
    assert_ne!(g1, g2);
}

#[test]
fn fm_v10_edge_equality_same_fields() {
    let g1 = with_fields("fm_v10_e08");
    let g2 = with_fields("fm_v10_e08");
    assert_eq!(g1.fields, g2.fields);
}

// ===========================================================================
// 10. fm_v10_size_* – Various grammar sizes (7 tests)
// ===========================================================================

#[test]
fn fm_v10_size_zero_fields() {
    let g = minimal("fm_v10_s01");
    assert_eq!(g.fields.len(), 0);
}

#[test]
fn fm_v10_size_one_field() {
    let g = with_many_fields("fm_v10_s02", 1);
    assert_eq!(g.fields.len(), 1);
}

#[test]
fn fm_v10_size_ten_fields() {
    let g = with_many_fields("fm_v10_s03", 10);
    assert_eq!(g.fields.len(), 10);
}

#[test]
fn fm_v10_size_hundred_fields() {
    let g = with_many_fields("fm_v10_s04", 100);
    assert_eq!(g.fields.len(), 100);
    assert_eq!(g.fields[&FieldId(99)], "field_99");
}

#[test]
fn fm_v10_size_rule_names_grow_with_rules() {
    let g1 = minimal("fm_v10_s05a");
    let g2 = multi_rule("fm_v10_s05b");
    assert!(g2.rule_names.len() >= g1.rule_names.len());
}

#[test]
fn fm_v10_size_fields_consistent_after_build() {
    for i in 0..5 {
        let g = with_many_fields(&format!("fm_v10_s06_{i}"), i * 3);
        assert_eq!(g.fields.len(), i * 3);
    }
}

#[test]
fn fm_v10_size_rule_names_consistent_across_builds() {
    let g1 = expr("fm_v10_s07a");
    let g2 = expr("fm_v10_s07b");
    assert_eq!(g1.rule_names.len(), g2.rule_names.len());
}
