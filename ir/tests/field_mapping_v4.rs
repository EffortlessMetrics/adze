//! Field mapping tests for adze-ir (v4).
//!
//! Covers: field creation, ID uniqueness, lookup, ordering,
//! grammar-with-fields, determinism, and edge cases.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{FieldId, Grammar};
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("minimal")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build()
}

fn expr_grammar() -> Grammar {
    GrammarBuilder::new("expr")
        .token("NUMBER", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

/// Build a grammar with three fields in lexicographic order.
fn grammar_with_fields() -> Grammar {
    let mut g = expr_grammar();
    g.fields.insert(FieldId(0), "left".to_string());
    g.fields.insert(FieldId(1), "operator".to_string());
    g.fields.insert(FieldId(2), "right".to_string());
    g
}

/// Build a grammar and wire field mappings into the ternary rule.
fn grammar_with_rule_fields() -> Grammar {
    let mut g = grammar_with_fields();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    if let Some(rules) = g.rules.get_mut(&expr_id) {
        for rule in rules.iter_mut() {
            if rule.rhs.len() == 3 {
                rule.fields = vec![(FieldId(0), 0), (FieldId(1), 1), (FieldId(2), 2)];
            }
        }
    }
    g
}

// ===========================================================================
// 1. Field creation — adding fields to grammar (8 tests)
// ===========================================================================

#[test]
fn test_create_single_field() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "value".to_string());
    assert_eq!(g.fields.len(), 1);
}

#[test]
fn test_create_field_stores_name() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "name".to_string());
    assert_eq!(g.fields[&FieldId(0)], "name");
}

#[test]
fn test_create_multiple_fields() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "alpha".to_string());
    g.fields.insert(FieldId(1), "beta".to_string());
    g.fields.insert(FieldId(2), "gamma".to_string());
    assert_eq!(g.fields.len(), 3);
}

#[test]
fn test_create_field_replaces_on_same_id() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "old".to_string());
    g.fields.insert(FieldId(0), "new".to_string());
    assert_eq!(g.fields.len(), 1);
    assert_eq!(g.fields[&FieldId(0)], "new");
}

#[test]
fn test_create_fields_noncontiguous_ids() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "a".to_string());
    g.fields.insert(FieldId(5), "b".to_string());
    g.fields.insert(FieldId(100), "c".to_string());
    assert_eq!(g.fields.len(), 3);
    assert_eq!(g.fields[&FieldId(100)], "c");
}

#[test]
fn test_create_field_with_empty_name() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), String::new());
    assert_eq!(g.fields[&FieldId(0)], "");
}

#[test]
fn test_create_field_preserves_existing_grammar_data() {
    let mut g = expr_grammar();
    let rule_count_before = g.rules.len();
    let token_count_before = g.tokens.len();
    g.fields.insert(FieldId(0), "field".to_string());
    assert_eq!(g.rules.len(), rule_count_before);
    assert_eq!(g.tokens.len(), token_count_before);
}

#[test]
fn test_create_field_on_built_grammar() {
    let mut g = GrammarBuilder::new("g")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    assert!(g.fields.is_empty());
    g.fields.insert(FieldId(0), "added".to_string());
    assert!(!g.fields.is_empty());
}

// ===========================================================================
// 2. Field ID uniqueness — no duplicate IDs (8 tests)
// ===========================================================================

#[test]
fn test_unique_ids_in_grammar_with_fields() {
    let g = grammar_with_fields();
    let ids: HashSet<FieldId> = g.fields.keys().copied().collect();
    assert_eq!(ids.len(), g.fields.len());
}

#[test]
fn test_inserting_duplicate_id_overwrites() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "first".to_string());
    g.fields.insert(FieldId(0), "second".to_string());
    assert_eq!(g.fields.len(), 1);
    assert_eq!(g.fields[&FieldId(0)], "second");
}

#[test]
fn test_distinct_ids_are_separate_entries() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "a".to_string());
    g.fields.insert(FieldId(1), "b".to_string());
    assert_ne!(g.fields[&FieldId(0)], g.fields[&FieldId(1)]);
}

#[test]
fn test_unique_ids_after_many_insertions() {
    let mut g = minimal_grammar();
    for i in 0..20 {
        g.fields.insert(FieldId(i), format!("field_{i}"));
    }
    let ids: HashSet<FieldId> = g.fields.keys().copied().collect();
    assert_eq!(ids.len(), 20);
}

#[test]
fn test_field_id_zero_and_one_distinct() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "zero".to_string());
    g.fields.insert(FieldId(1), "one".to_string());
    assert!(g.fields.contains_key(&FieldId(0)));
    assert!(g.fields.contains_key(&FieldId(1)));
    assert_eq!(g.fields.len(), 2);
}

#[test]
fn test_field_id_max_u16_is_unique() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "first".to_string());
    g.fields.insert(FieldId(u16::MAX), "last".to_string());
    assert_eq!(g.fields.len(), 2);
    assert_ne!(FieldId(0), FieldId(u16::MAX));
}

#[test]
fn test_uniqueness_via_hashset_collection() {
    let ids = vec![FieldId(0), FieldId(1), FieldId(2), FieldId(0), FieldId(1)];
    let unique: HashSet<FieldId> = ids.into_iter().collect();
    assert_eq!(unique.len(), 3);
}

#[test]
fn test_uniqueness_in_hashmap_as_key() {
    let mut map = HashMap::new();
    map.insert(FieldId(10), "ten");
    map.insert(FieldId(20), "twenty");
    map.insert(FieldId(10), "ten_again");
    assert_eq!(map.len(), 2);
    assert_eq!(map[&FieldId(10)], "ten_again");
}

// ===========================================================================
// 3. Field lookup — find by name (8 tests)
// ===========================================================================

#[test]
fn test_lookup_existing_field_by_id() {
    let g = grammar_with_fields();
    assert_eq!(g.fields.get(&FieldId(0)), Some(&"left".to_string()));
}

#[test]
fn test_lookup_returns_none_for_missing_id() {
    let g = grammar_with_fields();
    assert!(g.fields.get(&FieldId(99)).is_none());
}

#[test]
fn test_lookup_contains_key_true() {
    let g = grammar_with_fields();
    assert!(g.fields.contains_key(&FieldId(1)));
}

#[test]
fn test_lookup_contains_key_false() {
    let g = grammar_with_fields();
    assert!(!g.fields.contains_key(&FieldId(50)));
}

#[test]
fn test_lookup_find_id_by_name() {
    let g = grammar_with_fields();
    let found = g
        .fields
        .iter()
        .find(|(_, name)| name.as_str() == "operator")
        .map(|(id, _)| *id);
    assert_eq!(found, Some(FieldId(1)));
}

#[test]
fn test_lookup_find_name_not_present() {
    let g = grammar_with_fields();
    let found = g.fields.iter().find(|(_, name)| name.as_str() == "missing");
    assert!(found.is_none());
}

#[test]
fn test_lookup_all_values() {
    let g = grammar_with_fields();
    let names: Vec<&str> = g.fields.values().map(|s| s.as_str()).collect();
    assert!(names.contains(&"left"));
    assert!(names.contains(&"operator"));
    assert!(names.contains(&"right"));
}

#[test]
fn test_lookup_all_keys() {
    let g = grammar_with_fields();
    let keys: Vec<FieldId> = g.fields.keys().copied().collect();
    assert!(keys.contains(&FieldId(0)));
    assert!(keys.contains(&FieldId(1)));
    assert!(keys.contains(&FieldId(2)));
}

// ===========================================================================
// 4. Field ordering — insertion order preserved (IndexMap) (8 tests)
// ===========================================================================

#[test]
fn test_insertion_order_preserved_keys() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(2), "c".to_string());
    g.fields.insert(FieldId(0), "a".to_string());
    g.fields.insert(FieldId(1), "b".to_string());
    let keys: Vec<FieldId> = g.fields.keys().copied().collect();
    assert_eq!(keys, vec![FieldId(2), FieldId(0), FieldId(1)]);
}

#[test]
fn test_insertion_order_preserved_values() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(2), "gamma".to_string());
    g.fields.insert(FieldId(0), "alpha".to_string());
    g.fields.insert(FieldId(1), "beta".to_string());
    let values: Vec<&str> = g.fields.values().map(|s| s.as_str()).collect();
    assert_eq!(values, vec!["gamma", "alpha", "beta"]);
}

#[test]
fn test_first_inserted_is_first_iterated() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(99), "first".to_string());
    g.fields.insert(FieldId(0), "second".to_string());
    let (first_key, first_val) = g.fields.iter().next().unwrap();
    assert_eq!(*first_key, FieldId(99));
    assert_eq!(first_val, "first");
}

#[test]
fn test_last_inserted_is_last_iterated() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "a".to_string());
    g.fields.insert(FieldId(1), "b".to_string());
    g.fields.insert(FieldId(2), "c".to_string());
    let (last_key, last_val) = g.fields.iter().last().unwrap();
    assert_eq!(*last_key, FieldId(2));
    assert_eq!(last_val, "c");
}

#[test]
fn test_indexmap_get_index() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(10), "x".to_string());
    g.fields.insert(FieldId(20), "y".to_string());
    let (k, v) = g.fields.get_index(0).unwrap();
    assert_eq!(*k, FieldId(10));
    assert_eq!(v, "x");
    let (k2, v2) = g.fields.get_index(1).unwrap();
    assert_eq!(*k2, FieldId(20));
    assert_eq!(v2, "y");
}

#[test]
fn test_overwrite_preserves_original_position() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "a".to_string());
    g.fields.insert(FieldId(1), "b".to_string());
    g.fields.insert(FieldId(0), "a_updated".to_string());
    // IndexMap keeps insertion order; overwrite stays at original index
    let (k, v) = g.fields.get_index(0).unwrap();
    assert_eq!(*k, FieldId(0));
    assert_eq!(v, "a_updated");
}

#[test]
fn test_order_is_not_sorted_by_id() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(5), "e".to_string());
    g.fields.insert(FieldId(3), "c".to_string());
    g.fields.insert(FieldId(1), "a".to_string());
    let keys: Vec<u16> = g.fields.keys().map(|f| f.0).collect();
    // Insertion order, not numeric order
    assert_eq!(keys, vec![5, 3, 1]);
}

#[test]
fn test_collect_pairs_preserves_order() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(7), "seven".to_string());
    g.fields.insert(FieldId(3), "three".to_string());
    g.fields.insert(FieldId(9), "nine".to_string());
    let pairs: Vec<(FieldId, String)> = g.fields.iter().map(|(k, v)| (*k, v.clone())).collect();
    assert_eq!(pairs[0], (FieldId(7), "seven".to_string()));
    assert_eq!(pairs[1], (FieldId(3), "three".to_string()));
    assert_eq!(pairs[2], (FieldId(9), "nine".to_string()));
}

// ===========================================================================
// 5. Grammar with fields — builder produces correct fields (8 tests)
// ===========================================================================

#[test]
fn test_builder_produces_empty_fields() {
    let g = GrammarBuilder::new("test")
        .token("T", "t")
        .rule("s", vec!["T"])
        .start("s")
        .build();
    assert!(g.fields.is_empty());
}

#[test]
fn test_grammar_name_preserved_with_fields() {
    let g = grammar_with_fields();
    assert_eq!(g.name, "expr");
}

#[test]
fn test_grammar_rules_intact_after_adding_fields() {
    let g = grammar_with_fields();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.rules.contains_key(&expr_id));
}

#[test]
fn test_grammar_tokens_intact_after_adding_fields() {
    let g = grammar_with_fields();
    assert!(!g.tokens.is_empty());
}

#[test]
fn test_rule_fields_wired_correctly() {
    let g = grammar_with_rule_fields();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.rules.get(&expr_id).unwrap();
    let ternary = rules.iter().find(|r| r.rhs.len() == 3).unwrap();
    assert_eq!(ternary.fields.len(), 3);
    assert_eq!(ternary.fields[0], (FieldId(0), 0));
    assert_eq!(ternary.fields[1], (FieldId(1), 1));
    assert_eq!(ternary.fields[2], (FieldId(2), 2));
}

#[test]
fn test_unary_rule_has_no_fields() {
    let g = grammar_with_rule_fields();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.rules.get(&expr_id).unwrap();
    let unary = rules.iter().find(|r| r.rhs.len() == 1).unwrap();
    assert!(unary.fields.is_empty());
}

#[test]
fn test_validate_passes_with_lexicographic_fields() {
    let g = grammar_with_fields();
    assert!(g.validate().is_ok());
}

#[test]
fn test_validate_fails_with_nonlexicographic_fields() {
    let mut g = expr_grammar();
    // "z" comes after "a" but we insert z first — not lexicographic
    g.fields.insert(FieldId(0), "z_last".to_string());
    g.fields.insert(FieldId(1), "a_first".to_string());
    assert!(g.validate().is_err());
}

// ===========================================================================
// 6. Field determinism — same grammar → same field IDs (8 tests)
// ===========================================================================

#[test]
fn test_determinism_same_fields_twice() {
    let g1 = grammar_with_fields();
    let g2 = grammar_with_fields();
    assert_eq!(g1.fields, g2.fields);
}

#[test]
fn test_determinism_field_count() {
    let g1 = grammar_with_fields();
    let g2 = grammar_with_fields();
    assert_eq!(g1.fields.len(), g2.fields.len());
}

#[test]
fn test_determinism_field_keys_match() {
    let g1 = grammar_with_fields();
    let g2 = grammar_with_fields();
    let k1: Vec<FieldId> = g1.fields.keys().copied().collect();
    let k2: Vec<FieldId> = g2.fields.keys().copied().collect();
    assert_eq!(k1, k2);
}

#[test]
fn test_determinism_field_values_match() {
    let g1 = grammar_with_fields();
    let g2 = grammar_with_fields();
    let v1: Vec<&String> = g1.fields.values().collect();
    let v2: Vec<&String> = g2.fields.values().collect();
    assert_eq!(v1, v2);
}

#[test]
fn test_determinism_iteration_order() {
    let g1 = grammar_with_fields();
    let g2 = grammar_with_fields();
    let pairs1: Vec<_> = g1.fields.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let pairs2: Vec<_> = g2.fields.iter().map(|(k, v)| (*k, v.as_str())).collect();
    assert_eq!(pairs1, pairs2);
}

#[test]
fn test_determinism_with_rule_fields() {
    let g1 = grammar_with_rule_fields();
    let g2 = grammar_with_rule_fields();
    let expr_id1 = g1.find_symbol_by_name("expr").unwrap();
    let expr_id2 = g2.find_symbol_by_name("expr").unwrap();
    let r1 = &g1.rules[&expr_id1];
    let r2 = &g2.rules[&expr_id2];
    for (a, b) in r1.iter().zip(r2.iter()) {
        assert_eq!(a.fields, b.fields);
    }
}

#[test]
fn test_determinism_builder_empty_fields() {
    let g1 = minimal_grammar();
    let g2 = minimal_grammar();
    assert_eq!(g1.fields, g2.fields);
    assert!(g1.fields.is_empty());
}

#[test]
fn test_determinism_manual_field_insertion() {
    fn make() -> Grammar {
        let mut g = minimal_grammar();
        g.fields.insert(FieldId(0), "alpha".to_string());
        g.fields.insert(FieldId(1), "beta".to_string());
        g
    }
    assert_eq!(make().fields, make().fields);
}

// ===========================================================================
// 7. Edge cases (15+ tests)
// ===========================================================================

#[test]
fn test_edge_no_fields_grammar_valid() {
    let g = minimal_grammar();
    assert!(g.validate().is_ok());
}

#[test]
fn test_edge_single_field() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "only".to_string());
    assert_eq!(g.fields.len(), 1);
    assert!(g.validate().is_ok());
}

#[test]
fn test_edge_many_fields() {
    let mut g = minimal_grammar();
    // 50 fields in lex order
    let mut names: Vec<String> = (0..50).map(|i| format!("field_{i:03}")).collect();
    names.sort();
    for (i, name) in names.into_iter().enumerate() {
        g.fields.insert(FieldId(i as u16), name);
    }
    assert_eq!(g.fields.len(), 50);
    assert!(g.validate().is_ok());
}

#[test]
fn test_edge_field_name_with_underscore() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "my_field".to_string());
    assert_eq!(g.fields[&FieldId(0)], "my_field");
}

#[test]
fn test_edge_field_name_numeric_string() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "123".to_string());
    assert_eq!(g.fields[&FieldId(0)], "123");
}

#[test]
fn test_edge_field_name_with_hyphen() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "my-field".to_string());
    assert_eq!(g.fields[&FieldId(0)], "my-field");
}

#[test]
fn test_edge_field_name_with_dot() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "a.b".to_string());
    assert_eq!(g.fields[&FieldId(0)], "a.b");
}

#[test]
fn test_edge_field_name_unicode() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "名前".to_string());
    assert_eq!(g.fields[&FieldId(0)], "名前");
}

#[test]
fn test_edge_field_name_with_spaces() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "has space".to_string());
    assert_eq!(g.fields[&FieldId(0)], "has space");
}

#[test]
fn test_edge_field_id_zero() {
    assert_eq!(FieldId(0).0, 0);
    assert_eq!(format!("{}", FieldId(0)), "Field(0)");
}

#[test]
fn test_edge_field_id_max() {
    let f = FieldId(u16::MAX);
    assert_eq!(f.0, 65535);
    assert_eq!(format!("{f}"), "Field(65535)");
}

#[test]
fn test_edge_field_removal() {
    let mut g = grammar_with_fields();
    assert_eq!(g.fields.len(), 3);
    g.fields.shift_remove(&FieldId(1));
    assert_eq!(g.fields.len(), 2);
    assert!(g.fields.get(&FieldId(1)).is_none());
}

#[test]
fn test_edge_field_removal_preserves_others() {
    let mut g = grammar_with_fields();
    g.fields.shift_remove(&FieldId(1));
    assert_eq!(g.fields[&FieldId(0)], "left");
    assert_eq!(g.fields[&FieldId(2)], "right");
}

#[test]
fn test_edge_clear_all_fields() {
    let mut g = grammar_with_fields();
    g.fields.clear();
    assert!(g.fields.is_empty());
    // Grammar is still valid with no fields
    assert!(g.validate().is_ok());
}

#[test]
fn test_edge_duplicate_field_names_different_ids() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "dup".to_string());
    g.fields.insert(FieldId(1), "dup".to_string());
    // IndexMap allows same values with different keys
    assert_eq!(g.fields.len(), 2);
    assert_eq!(g.fields[&FieldId(0)], "dup");
    assert_eq!(g.fields[&FieldId(1)], "dup");
}

#[test]
fn test_edge_field_name_long_string() {
    let mut g = minimal_grammar();
    let long_name = "a".repeat(1000);
    g.fields.insert(FieldId(0), long_name.clone());
    assert_eq!(g.fields[&FieldId(0)], long_name);
}

#[test]
fn test_edge_fields_from_fresh_indexmap() {
    let mut g = minimal_grammar();
    let mut new_fields = IndexMap::new();
    new_fields.insert(FieldId(0), "alpha".to_string());
    new_fields.insert(FieldId(1), "beta".to_string());
    g.fields = new_fields;
    assert_eq!(g.fields.len(), 2);
    assert_eq!(g.fields[&FieldId(0)], "alpha");
}

#[test]
fn test_edge_field_id_display_various() {
    assert_eq!(format!("{}", FieldId(0)), "Field(0)");
    assert_eq!(format!("{}", FieldId(1)), "Field(1)");
    assert_eq!(format!("{}", FieldId(255)), "Field(255)");
    assert_eq!(format!("{}", FieldId(1000)), "Field(1000)");
}

#[test]
fn test_edge_field_id_debug_various() {
    let dbg = format!("{:?}", FieldId(42));
    assert!(dbg.contains("42"));
    assert!(dbg.contains("FieldId"));
}

#[test]
fn test_edge_field_id_copy_semantics() {
    let a = FieldId(7);
    let b = a; // Copy
    let c = a; // Still valid — Copy
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn test_edge_field_id_in_vec() {
    let ids = [FieldId(0), FieldId(1), FieldId(2)];
    assert_eq!(ids.len(), 3);
    assert_eq!(ids[1], FieldId(1));
}

#[test]
fn test_edge_validate_single_field_lex_ok() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "z".to_string());
    // Single field is trivially in lex order
    assert!(g.validate().is_ok());
}

#[test]
fn test_edge_validate_two_fields_lex_ok() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "aaa".to_string());
    g.fields.insert(FieldId(1), "zzz".to_string());
    assert!(g.validate().is_ok());
}

#[test]
fn test_edge_validate_two_fields_lex_bad() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "zzz".to_string());
    g.fields.insert(FieldId(1), "aaa".to_string());
    assert!(g.validate().is_err());
}

#[test]
fn test_edge_validate_equal_names_lex_ok() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "same".to_string());
    g.fields.insert(FieldId(1), "same".to_string());
    // Equal names are trivially "sorted"
    assert!(g.validate().is_ok());
}
