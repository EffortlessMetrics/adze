//! Comprehensive tests for field mapping and grammar fields in adze-ir (v3).
//!
//! Covers: empty defaults, FieldId properties, field name lookups,
//! normalize preservation, multiple fields, field-rule interaction, edge cases.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{FieldId, Grammar};
use std::collections::HashSet;

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
        .token("PLUS", "+")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

fn grammar_with_fields() -> Grammar {
    let mut g = expr_grammar();
    // Fields must be in lexicographic order
    g.fields.insert(FieldId(0), "left".to_string());
    g.fields.insert(FieldId(1), "operator".to_string());
    g.fields.insert(FieldId(2), "right".to_string());
    g
}

fn grammar_with_field_mappings() -> Grammar {
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
// 1. Empty fields by default (8 tests)
// ===========================================================================

#[test]
fn test_default_fields_is_empty() {
    let g = minimal_grammar();
    assert!(g.fields.is_empty());
}

#[test]
fn test_default_fields_len_zero() {
    let g = minimal_grammar();
    assert_eq!(g.fields.len(), 0);
}

#[test]
fn test_default_fields_iter_empty() {
    let g = minimal_grammar();
    assert_eq!(g.fields.iter().count(), 0);
}

#[test]
fn test_default_fields_values_empty() {
    let g = minimal_grammar();
    assert_eq!(g.fields.values().count(), 0);
}

#[test]
fn test_default_fields_keys_empty() {
    let g = minimal_grammar();
    assert_eq!(g.fields.keys().count(), 0);
}

#[test]
fn test_expr_grammar_fields_empty_by_default() {
    let g = expr_grammar();
    assert!(g.fields.is_empty());
}

#[test]
fn test_default_fields_get_returns_none() {
    let g = minimal_grammar();
    assert!(g.fields.get(&FieldId(0)).is_none());
}

#[test]
fn test_default_fields_contains_key_false() {
    let g = minimal_grammar();
    assert!(!g.fields.contains_key(&FieldId(0)));
}

// ===========================================================================
// 2. FieldId properties (8 tests)
// ===========================================================================

#[test]
fn test_field_id_copy() {
    let a = FieldId(42);
    let b = a; // Copy, not move
    assert_eq!(a, b);
}

#[test]
fn test_field_id_clone_eq_copy() {
    let a = FieldId(7);
    #[allow(clippy::clone_on_copy)]
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn test_field_id_debug_format() {
    let f = FieldId(3);
    let dbg = format!("{f:?}");
    assert!(dbg.contains("FieldId"));
    assert!(dbg.contains('3'));
}

#[test]
fn test_field_id_display_format() {
    let f = FieldId(5);
    let disp = format!("{f}");
    assert_eq!(disp, "Field(5)");
}

#[test]
fn test_field_id_eq_same_value() {
    assert_eq!(FieldId(0), FieldId(0));
}

#[test]
fn test_field_id_ne_different_value() {
    assert_ne!(FieldId(0), FieldId(1));
}

#[test]
fn test_field_id_hash_consistent() {
    use std::hash::{Hash, Hasher};
    let mut h1 = std::collections::hash_map::DefaultHasher::new();
    let mut h2 = std::collections::hash_map::DefaultHasher::new();
    FieldId(10).hash(&mut h1);
    FieldId(10).hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn test_field_id_usable_as_hash_key() {
    let mut set = HashSet::new();
    set.insert(FieldId(0));
    set.insert(FieldId(1));
    set.insert(FieldId(0)); // duplicate
    assert_eq!(set.len(), 2);
}

// ===========================================================================
// 3. Field names (8 tests)
// ===========================================================================

#[test]
fn test_field_name_inserted_and_retrieved() {
    let g = grammar_with_fields();
    assert_eq!(g.fields.get(&FieldId(0)), Some(&"left".to_string()));
}

#[test]
fn test_field_name_operator() {
    let g = grammar_with_fields();
    assert_eq!(g.fields.get(&FieldId(1)).unwrap(), "operator");
}

#[test]
fn test_field_name_right() {
    let g = grammar_with_fields();
    assert_eq!(g.fields[&FieldId(2)], "right");
}

#[test]
fn test_field_names_count() {
    let g = grammar_with_fields();
    assert_eq!(g.fields.len(), 3);
}

#[test]
fn test_field_names_values_collected() {
    let g = grammar_with_fields();
    let names: Vec<&String> = g.fields.values().collect();
    assert_eq!(names, &["left", "operator", "right"]);
}

#[test]
fn test_field_names_keys_collected() {
    let g = grammar_with_fields();
    let ids: Vec<FieldId> = g.fields.keys().copied().collect();
    assert_eq!(ids, [FieldId(0), FieldId(1), FieldId(2)]);
}

#[test]
fn test_field_lookup_missing_returns_none() {
    let g = grammar_with_fields();
    assert!(g.fields.get(&FieldId(99)).is_none());
}

#[test]
fn test_field_reverse_lookup_by_value() {
    let g = grammar_with_fields();
    let found = g
        .fields
        .iter()
        .find(|(_id, name)| name.as_str() == "operator")
        .map(|(id, _)| *id);
    assert_eq!(found, Some(FieldId(1)));
}

// ===========================================================================
// 4. Fields after normalize (8 tests)
// ===========================================================================

#[test]
fn test_normalize_preserves_field_count() {
    let mut g = grammar_with_fields();
    let before = g.fields.len();
    g.normalize();
    assert_eq!(g.fields.len(), before);
}

#[test]
fn test_normalize_preserves_field_names() {
    let mut g = grammar_with_fields();
    let before: Vec<String> = g.fields.values().cloned().collect();
    g.normalize();
    let after: Vec<String> = g.fields.values().cloned().collect();
    assert_eq!(before, after);
}

#[test]
fn test_normalize_preserves_field_ids() {
    let mut g = grammar_with_fields();
    let before: Vec<FieldId> = g.fields.keys().copied().collect();
    g.normalize();
    let after: Vec<FieldId> = g.fields.keys().copied().collect();
    assert_eq!(before, after);
}

#[test]
fn test_normalize_preserves_field_order() {
    let mut g = grammar_with_fields();
    g.normalize();
    let names: Vec<&str> = g.fields.values().map(String::as_str).collect();
    assert_eq!(names, ["left", "operator", "right"]);
}

#[test]
fn test_normalize_empty_fields_stay_empty() {
    let mut g = minimal_grammar();
    g.normalize();
    assert!(g.fields.is_empty());
}

#[test]
fn test_normalize_preserves_field_mappings_on_rules() {
    let mut g = grammar_with_field_mappings();
    g.normalize();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.rules.get(&expr_id).unwrap();
    let mapped_rule = rules.iter().find(|r| r.rhs.len() == 3);
    assert!(mapped_rule.is_some());
    let fields = &mapped_rule.unwrap().fields;
    assert_eq!(fields.len(), 3);
}

#[test]
fn test_normalize_field_mapping_positions_intact() {
    let mut g = grammar_with_field_mappings();
    g.normalize();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.rules.get(&expr_id).unwrap();
    let mapped = rules.iter().find(|r| !r.fields.is_empty()).unwrap();
    assert_eq!(mapped.fields[0], (FieldId(0), 0));
    assert_eq!(mapped.fields[1], (FieldId(1), 1));
    assert_eq!(mapped.fields[2], (FieldId(2), 2));
}

#[test]
fn test_normalize_idempotent_for_fields() {
    let mut g = grammar_with_fields();
    g.normalize();
    let after_first: Vec<(FieldId, String)> = g
        .fields
        .iter()
        .map(|(k, v): (&FieldId, &String)| (*k, v.clone()))
        .collect();
    g.normalize();
    let after_second: Vec<(FieldId, String)> = g
        .fields
        .iter()
        .map(|(k, v): (&FieldId, &String)| (*k, v.clone()))
        .collect();
    assert_eq!(after_first, after_second);
}

// ===========================================================================
// 5. Multiple fields (7 tests)
// ===========================================================================

#[test]
fn test_multiple_fields_unique_ids() {
    let g = grammar_with_fields();
    let ids: Vec<FieldId> = g.fields.keys().copied().collect();
    let unique: HashSet<FieldId> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len());
}

#[test]
fn test_multiple_fields_unique_names() {
    let g = grammar_with_fields();
    let names: Vec<&String> = g.fields.values().collect();
    let unique: HashSet<&String> = names.iter().copied().collect();
    assert_eq!(names.len(), unique.len());
}

#[test]
fn test_five_fields_coexist() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "alpha".to_string());
    g.fields.insert(FieldId(1), "beta".to_string());
    g.fields.insert(FieldId(2), "delta".to_string());
    g.fields.insert(FieldId(3), "gamma".to_string());
    g.fields.insert(FieldId(4), "zeta".to_string());
    assert_eq!(g.fields.len(), 5);
}

#[test]
fn test_fields_insertion_order_preserved() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(10), "zebra".to_string());
    g.fields.insert(FieldId(5), "apple".to_string());
    g.fields.insert(FieldId(20), "mango".to_string());
    let ids: Vec<u16> = g.fields.keys().map(|f| f.0).collect();
    // IndexMap preserves insertion order
    assert_eq!(ids, [10, 5, 20]);
}

#[test]
fn test_fields_non_contiguous_ids() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "a".to_string());
    g.fields.insert(FieldId(100), "b".to_string());
    g.fields.insert(FieldId(50), "c".to_string());
    assert_eq!(g.fields.len(), 3);
    assert_eq!(g.fields[&FieldId(100)], "b");
}

#[test]
fn test_fields_overwrite_same_key() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "old_name".to_string());
    g.fields.insert(FieldId(0), "new_name".to_string());
    assert_eq!(g.fields.len(), 1);
    assert_eq!(g.fields[&FieldId(0)], "new_name");
}

#[test]
fn test_fields_remove_decreases_count() {
    let mut g = grammar_with_fields();
    assert_eq!(g.fields.len(), 3);
    g.fields.shift_remove(&FieldId(1));
    assert_eq!(g.fields.len(), 2);
    assert!(g.fields.get(&FieldId(1)).is_none());
}

// ===========================================================================
// 6. Field-rule interaction (8 tests)
// ===========================================================================

#[test]
fn test_rule_fields_default_empty() {
    let g = expr_grammar();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    for rule in g.rules[&expr_id].iter() {
        assert!(rule.fields.is_empty());
    }
}

#[test]
fn test_rule_field_mapping_attached() {
    let g = grammar_with_field_mappings();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let has_fields = g.rules[&expr_id].iter().any(|r| !r.fields.is_empty());
    assert!(has_fields);
}

#[test]
fn test_rule_field_mapping_references_grammar_fields() {
    let g = grammar_with_field_mappings();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    for rule in &g.rules[&expr_id] {
        for (field_id, _pos) in &rule.fields {
            assert!(
                g.fields.contains_key(field_id),
                "Rule references FieldId({}) not in grammar.fields",
                field_id.0
            );
        }
    }
}

#[test]
fn test_rule_field_position_within_rhs_bounds() {
    let g = grammar_with_field_mappings();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    for rule in &g.rules[&expr_id] {
        for (_field_id, pos) in &rule.fields {
            assert!(
                *pos < rule.rhs.len(),
                "Field position {} out of bounds for rhs len {}",
                pos,
                rule.rhs.len()
            );
        }
    }
}

#[test]
fn test_field_names_relate_to_rhs_semantics() {
    let g = grammar_with_field_mappings();
    // FieldId(0) -> position 0 in "expr PLUS expr" should be "left"
    assert_eq!(g.fields[&FieldId(0)], "left");
    // FieldId(2) -> position 2 should be "right"
    assert_eq!(g.fields[&FieldId(2)], "right");
}

#[test]
fn test_unmapped_rule_has_no_fields() {
    let g = grammar_with_field_mappings();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    // The `expr -> NUMBER` rule should not have fields
    let simple_rule = g.rules[&expr_id].iter().find(|r| r.rhs.len() == 1).unwrap();
    assert!(simple_rule.fields.is_empty());
}

#[test]
fn test_field_mapping_count_matches_rhs_for_ternary_rule() {
    let g = grammar_with_field_mappings();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let ternary = g.rules[&expr_id].iter().find(|r| r.rhs.len() == 3).unwrap();
    assert_eq!(ternary.fields.len(), 3);
}

#[test]
fn test_grammar_rules_and_fields_coexist() {
    let g = grammar_with_fields();
    assert!(!g.rules.is_empty());
    assert!(!g.fields.is_empty());
    assert!(!g.tokens.is_empty());
}

// ===========================================================================
// 7. Edge cases (8 tests)
// ===========================================================================

#[test]
fn test_empty_field_name_allowed_by_indexmap() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), String::new());
    assert_eq!(g.fields[&FieldId(0)], "");
}

#[test]
fn test_field_id_zero() {
    let f = FieldId(0);
    assert_eq!(f.0, 0);
    assert_eq!(format!("{f}"), "Field(0)");
}

#[test]
fn test_field_id_max_u16() {
    let f = FieldId(u16::MAX);
    assert_eq!(f.0, 65535);
    assert_eq!(format!("{f}"), "Field(65535)");
}

#[test]
fn test_many_fields_inserted() {
    let mut g = minimal_grammar();
    for i in 0..100 {
        g.fields.insert(FieldId(i), format!("field_{i}"));
    }
    assert_eq!(g.fields.len(), 100);
    assert_eq!(g.fields[&FieldId(99)], "field_99");
}

#[test]
fn test_field_id_inner_value_accessible() {
    let f = FieldId(42);
    let inner: u16 = f.0;
    assert_eq!(inner, 42);
}

#[test]
fn test_fields_cleared() {
    let mut g = grammar_with_fields();
    assert!(!g.fields.is_empty());
    g.fields.clear();
    assert!(g.fields.is_empty());
}

#[test]
fn test_field_with_unicode_name() {
    let mut g = minimal_grammar();
    g.fields.insert(FieldId(0), "名前".to_string());
    assert_eq!(g.fields[&FieldId(0)], "名前");
}

#[test]
fn test_field_with_long_name() {
    let mut g = minimal_grammar();
    let long_name = "a".repeat(1000);
    g.fields.insert(FieldId(0), long_name.clone());
    assert_eq!(g.fields[&FieldId(0)], long_name);
}
