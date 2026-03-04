//! Comprehensive tests for FieldId, field mappings in Rules, and Grammar.fields.

use std::collections::{HashMap, HashSet};

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, FieldId, Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId,
};

// ===========================================================================
// 1. FieldId construction
// ===========================================================================

#[test]
fn field_id_zero() {
    let id = FieldId(0);
    assert_eq!(id.0, 0);
}

#[test]
fn field_id_one() {
    let id = FieldId(1);
    assert_eq!(id.0, 1);
}

#[test]
fn field_id_max() {
    let id = FieldId(u16::MAX);
    assert_eq!(id.0, u16::MAX);
}

#[test]
fn field_id_mid_range() {
    let id = FieldId(32768);
    assert_eq!(id.0, 32768);
}

#[test]
fn field_id_inner_value_accessible() {
    let id = FieldId(42);
    let inner: u16 = id.0;
    assert_eq!(inner, 42);
}

// ===========================================================================
// 2. FieldId equality and ordering
// ===========================================================================

#[test]
fn field_id_equal_same_value() {
    assert_eq!(FieldId(5), FieldId(5));
}

#[test]
fn field_id_not_equal_different_value() {
    assert_ne!(FieldId(1), FieldId(2));
}

#[test]
fn field_id_reflexive_equality() {
    let a = FieldId(10);
    assert_eq!(a, a);
}

#[test]
fn field_id_symmetric_equality() {
    let a = FieldId(7);
    let b = FieldId(7);
    assert_eq!(a, b);
    assert_eq!(b, a);
}

#[test]
fn field_id_transitive_equality() {
    let a = FieldId(3);
    let b = FieldId(3);
    let c = FieldId(3);
    assert_eq!(a, b);
    assert_eq!(b, c);
    assert_eq!(a, c);
}

#[test]
fn field_id_zero_and_max_not_equal() {
    assert_ne!(FieldId(0), FieldId(u16::MAX));
}

// ===========================================================================
// 3. FieldId hashing
// ===========================================================================

#[test]
fn field_id_hashset_insert_unique() {
    let mut set = HashSet::new();
    set.insert(FieldId(0));
    set.insert(FieldId(1));
    set.insert(FieldId(2));
    assert_eq!(set.len(), 3);
}

#[test]
fn field_id_hashset_duplicates_dedup() {
    let mut set = HashSet::new();
    set.insert(FieldId(5));
    set.insert(FieldId(5));
    assert_eq!(set.len(), 1);
}

#[test]
fn field_id_hashset_contains() {
    let mut set = HashSet::new();
    set.insert(FieldId(99));
    assert!(set.contains(&FieldId(99)));
    assert!(!set.contains(&FieldId(100)));
}

#[test]
fn field_id_hashmap_key() {
    let mut map = HashMap::new();
    map.insert(FieldId(0), "name");
    map.insert(FieldId(1), "value");
    assert_eq!(map[&FieldId(0)], "name");
    assert_eq!(map[&FieldId(1)], "value");
}

#[test]
fn field_id_hashset_remove() {
    let mut set = HashSet::new();
    set.insert(FieldId(10));
    set.insert(FieldId(20));
    assert!(set.remove(&FieldId(10)));
    assert_eq!(set.len(), 1);
    assert!(!set.contains(&FieldId(10)));
}

// ===========================================================================
// 4. FieldId serialization roundtrip
// ===========================================================================

#[test]
fn field_id_json_roundtrip_zero() {
    let original = FieldId(0);
    let json = serde_json::to_string(&original).unwrap();
    let back: FieldId = serde_json::from_str(&json).unwrap();
    assert_eq!(original, back);
}

#[test]
fn field_id_json_roundtrip_max() {
    let original = FieldId(u16::MAX);
    let json = serde_json::to_string(&original).unwrap();
    let back: FieldId = serde_json::from_str(&json).unwrap();
    assert_eq!(original, back);
}

#[test]
fn field_id_json_roundtrip_arbitrary() {
    for val in [1u16, 50, 100, 1000, 10000, 30000] {
        let original = FieldId(val);
        let json = serde_json::to_string(&original).unwrap();
        let back: FieldId = serde_json::from_str(&json).unwrap();
        assert_eq!(original, back, "roundtrip failed for {val}");
    }
}

#[test]
fn field_id_json_format() {
    let json = serde_json::to_string(&FieldId(42)).unwrap();
    assert_eq!(json, "42");
}

#[test]
fn field_id_deserialize_from_integer() {
    let back: FieldId = serde_json::from_str("7").unwrap();
    assert_eq!(back, FieldId(7));
}

// ===========================================================================
// 5. FieldId debug format
// ===========================================================================

#[test]
fn field_id_debug_format() {
    let dbg = format!("{:?}", FieldId(0));
    assert_eq!(dbg, "FieldId(0)");
}

#[test]
fn field_id_debug_format_large() {
    let dbg = format!("{:?}", FieldId(65535));
    assert_eq!(dbg, "FieldId(65535)");
}

#[test]
fn field_id_display_format_zero() {
    let disp = format!("{}", FieldId(0));
    assert_eq!(disp, "Field(0)");
}

#[test]
fn field_id_display_format_nonzero() {
    let disp = format!("{}", FieldId(100));
    assert_eq!(disp, "Field(100)");
}

// ===========================================================================
// 6. FieldId copy semantics
// ===========================================================================

#[test]
fn field_id_copy_on_assign() {
    let a = FieldId(8);
    let b = a; // Copy
    assert_eq!(a, b);
    // `a` is still usable after move because FieldId is Copy
    assert_eq!(a.0, 8);
}

#[test]
fn field_id_copy_into_function() {
    fn take_id(id: FieldId) -> u16 {
        id.0
    }
    let id = FieldId(55);
    let val = take_id(id);
    assert_eq!(val, 55);
    // id is still usable
    assert_eq!(id, FieldId(55));
}

#[test]
fn field_id_clone_equals_original() {
    let a = FieldId(33);
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn field_id_copy_in_vec() {
    let ids = vec![FieldId(1), FieldId(2), FieldId(3)];
    let first = ids[0]; // Copy from indexing
    assert_eq!(first, FieldId(1));
    assert_eq!(ids.len(), 3); // Vec is still intact
}

// ===========================================================================
// 7. FieldId in collections
// ===========================================================================

#[test]
fn field_id_vec_sort_by_inner() {
    let mut ids = vec![FieldId(3), FieldId(1), FieldId(2)];
    ids.sort_by_key(|f| f.0);
    assert_eq!(ids, vec![FieldId(1), FieldId(2), FieldId(3)]);
}

#[test]
fn field_id_hashset_large_count() {
    let mut set = HashSet::new();
    for i in 0..100u16 {
        set.insert(FieldId(i));
    }
    assert_eq!(set.len(), 100);
}

#[test]
fn field_id_hashset_intersection() {
    let a: HashSet<_> = [FieldId(1), FieldId(2), FieldId(3)].into();
    let b: HashSet<_> = [FieldId(2), FieldId(3), FieldId(4)].into();
    let inter: HashSet<_> = a.intersection(&b).copied().collect();
    assert_eq!(inter.len(), 2);
    assert!(inter.contains(&FieldId(2)));
    assert!(inter.contains(&FieldId(3)));
}

#[test]
fn field_id_hashmap_overwrite() {
    let mut map = HashMap::new();
    map.insert(FieldId(0), "old");
    map.insert(FieldId(0), "new");
    assert_eq!(map.len(), 1);
    assert_eq!(map[&FieldId(0)], "new");
}

#[test]
fn field_id_collect_from_iterator() {
    let set: HashSet<FieldId> = (0..5u16).map(FieldId).collect();
    assert_eq!(set.len(), 5);
    for i in 0..5u16 {
        assert!(set.contains(&FieldId(i)));
    }
}

// ===========================================================================
// 8. Rule.fields access
// ===========================================================================

#[test]
fn rule_fields_default_is_empty() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(0),
    };
    assert!(rule.fields.is_empty());
}

#[test]
fn rule_fields_single_mapping() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    };
    assert_eq!(rule.fields.len(), 1);
    assert_eq!(rule.fields[0].0, FieldId(0));
    assert_eq!(rule.fields[0].1, 0);
}

#[test]
fn rule_fields_multiple_mappings() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(SymbolId(1)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 1), (FieldId(2), 2)],
        production_id: ProductionId(0),
    };
    assert_eq!(rule.fields.len(), 3);
}

#[test]
fn rule_fields_position_matches_rhs_index() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::Terminal(SymbolId(10)),
            Symbol::Terminal(SymbolId(20)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 1)],
        production_id: ProductionId(0),
    };
    for &(_, pos) in &rule.fields {
        assert!(pos < rule.rhs.len());
    }
}

#[test]
fn rule_fields_extract_field_ids() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::Terminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(3)),
            Symbol::Terminal(SymbolId(4)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(10), 0), (FieldId(20), 1), (FieldId(30), 2)],
        production_id: ProductionId(0),
    };
    let ids: Vec<FieldId> = rule.fields.iter().map(|&(fid, _)| fid).collect();
    assert_eq!(ids, vec![FieldId(10), FieldId(20), FieldId(30)]);
}

#[test]
fn rule_fields_extract_positions() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2)), Symbol::Terminal(SymbolId(3))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 1)],
        production_id: ProductionId(0),
    };
    let positions: Vec<usize> = rule.fields.iter().map(|&(_, p)| p).collect();
    assert_eq!(positions, vec![0, 1]);
}

#[test]
fn rule_fields_clone_preserves_mappings() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(5), 0)],
        production_id: ProductionId(0),
    };
    let cloned = rule.clone();
    assert_eq!(rule.fields, cloned.fields);
}

#[test]
fn rule_fields_equality_same() {
    let r1 = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    };
    let r2 = r1.clone();
    assert_eq!(r1, r2);
}

#[test]
fn rule_fields_equality_different_field_id() {
    let r1 = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    };
    let r2 = Rule {
        fields: vec![(FieldId(1), 0)],
        ..r1.clone()
    };
    assert_ne!(r1, r2);
}

#[test]
fn rule_fields_equality_different_position() {
    let r1 = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2)), Symbol::Terminal(SymbolId(3))],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    };
    let r2 = Rule {
        fields: vec![(FieldId(0), 1)],
        ..r1.clone()
    };
    assert_ne!(r1, r2);
}

#[test]
fn rule_fields_with_precedence_and_assoc() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(SymbolId(1)),
        ],
        precedence: Some(PrecedenceKind::Static(10)),
        associativity: Some(Associativity::Left),
        fields: vec![(FieldId(0), 0), (FieldId(1), 1), (FieldId(2), 2)],
        production_id: ProductionId(7),
    };
    assert_eq!(rule.fields.len(), 3);
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(10)));
    assert_eq!(rule.associativity, Some(Associativity::Left));
}

#[test]
fn rule_fields_serde_roundtrip() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(SymbolId(3)),
        ],
        precedence: Some(PrecedenceKind::Dynamic(3)),
        associativity: Some(Associativity::Right),
        fields: vec![(FieldId(0), 0), (FieldId(1), 1)],
        production_id: ProductionId(99),
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: Rule = serde_json::from_str(&json).unwrap();
    assert_eq!(rule, back);
}

// ===========================================================================
// 9. Grammar.fields access
// ===========================================================================

#[test]
fn grammar_fields_empty_by_default() {
    let g = Grammar::new("test".to_string());
    assert!(g.fields.is_empty());
}

#[test]
fn grammar_fields_insert_one() {
    let mut g = Grammar::new("t".to_string());
    g.fields.insert(FieldId(0), "name".to_string());
    assert_eq!(g.fields.len(), 1);
    assert_eq!(g.fields[&FieldId(0)], "name");
}

#[test]
fn grammar_fields_insert_multiple_ordered() {
    let mut g = Grammar::new("t".to_string());
    g.fields.insert(FieldId(0), "alpha".to_string());
    g.fields.insert(FieldId(1), "beta".to_string());
    g.fields.insert(FieldId(2), "gamma".to_string());
    assert_eq!(g.fields.len(), 3);
    let names: Vec<&str> = g.fields.values().map(|s| s.as_str()).collect();
    assert_eq!(names, vec!["alpha", "beta", "gamma"]);
}

#[test]
fn grammar_fields_overwrite_same_key() {
    let mut g = Grammar::new("t".to_string());
    g.fields.insert(FieldId(0), "old".to_string());
    g.fields.insert(FieldId(0), "new".to_string());
    assert_eq!(g.fields.len(), 1);
    assert_eq!(g.fields[&FieldId(0)], "new");
}

#[test]
fn grammar_fields_get_missing_returns_none() {
    let g = Grammar::new("t".to_string());
    assert!(g.fields.get(&FieldId(999)).is_none());
}

#[test]
fn grammar_fields_iter_keys() {
    let mut g = Grammar::new("t".to_string());
    g.fields.insert(FieldId(0), "a".to_string());
    g.fields.insert(FieldId(1), "b".to_string());
    let keys: Vec<FieldId> = g.fields.keys().copied().collect();
    assert_eq!(keys, vec![FieldId(0), FieldId(1)]);
}

#[test]
fn grammar_fields_iter_values() {
    let mut g = Grammar::new("t".to_string());
    g.fields.insert(FieldId(0), "x".to_string());
    g.fields.insert(FieldId(1), "y".to_string());
    let vals: Vec<&str> = g.fields.values().map(|s| s.as_str()).collect();
    assert_eq!(vals, vec!["x", "y"]);
}

#[test]
fn grammar_fields_contains_key() {
    let mut g = Grammar::new("t".to_string());
    g.fields.insert(FieldId(7), "seven".to_string());
    assert!(g.fields.contains_key(&FieldId(7)));
    assert!(!g.fields.contains_key(&FieldId(8)));
}

#[test]
fn grammar_fields_validate_lexicographic_ok() {
    let mut g = Grammar::new("t".to_string());
    g.fields.insert(FieldId(0), "aaa".to_string());
    g.fields.insert(FieldId(1), "bbb".to_string());
    g.fields.insert(FieldId(2), "ccc".to_string());
    assert!(g.validate().is_ok());
}

#[test]
fn grammar_fields_validate_single_field_ok() {
    let mut g = Grammar::new("t".to_string());
    g.fields.insert(FieldId(0), "only".to_string());
    assert!(g.validate().is_ok());
}

#[test]
fn grammar_fields_validate_empty_ok() {
    let g = Grammar::new("t".to_string());
    assert!(g.validate().is_ok());
}

#[test]
fn grammar_fields_validate_non_lexicographic_fails() {
    let mut g = Grammar::new("t".to_string());
    g.fields.insert(FieldId(0), "zebra".to_string());
    g.fields.insert(FieldId(1), "alpha".to_string());
    assert!(g.validate().is_err());
}

// ===========================================================================
// 10. Field mappings in built grammars
// ===========================================================================

fn build_expr_grammar_with_fields() -> Grammar {
    let mut grammar = GrammarBuilder::new("expr")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    grammar.fields.insert(FieldId(0), "left".to_string());
    grammar.fields.insert(FieldId(1), "operator".to_string());
    grammar.fields.insert(FieldId(2), "right".to_string());

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    if let Some(rules) = grammar.rules.get_mut(&expr_id) {
        for rule in rules.iter_mut() {
            if rule.rhs.len() == 3 {
                rule.fields = vec![(FieldId(0), 0), (FieldId(1), 1), (FieldId(2), 2)];
            }
        }
    }
    grammar
}

#[test]
fn built_grammar_has_three_fields() {
    let g = build_expr_grammar_with_fields();
    assert_eq!(g.fields.len(), 3);
}

#[test]
fn built_grammar_field_names_correct() {
    let g = build_expr_grammar_with_fields();
    assert_eq!(g.fields[&FieldId(0)], "left");
    assert_eq!(g.fields[&FieldId(1)], "operator");
    assert_eq!(g.fields[&FieldId(2)], "right");
}

#[test]
fn built_grammar_binary_rules_have_fields() {
    let g = build_expr_grammar_with_fields();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    let binary: Vec<_> = rules.iter().filter(|r| r.rhs.len() == 3).collect();
    assert_eq!(binary.len(), 2);
    for rule in &binary {
        assert_eq!(rule.fields.len(), 3);
    }
}

#[test]
fn built_grammar_unary_rule_has_no_fields() {
    let g = build_expr_grammar_with_fields();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    let unary: Vec<_> = rules.iter().filter(|r| r.rhs.len() == 1).collect();
    assert_eq!(unary.len(), 1);
    assert!(unary[0].fields.is_empty());
}

#[test]
fn built_grammar_validates_with_fields() {
    let g = build_expr_grammar_with_fields();
    assert!(g.validate().is_ok());
}

#[test]
fn built_grammar_field_ids_in_rules_reference_grammar_fields() {
    let g = build_expr_grammar_with_fields();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    for rule in rules {
        for &(fid, _) in &rule.fields {
            assert!(
                g.fields.contains_key(&fid),
                "FieldId({}) not in grammar.fields",
                fid.0
            );
        }
    }
}

#[test]
fn built_grammar_serde_roundtrip_preserves_fields() {
    let g = build_expr_grammar_with_fields();
    let json = serde_json::to_string(&g).unwrap();
    let back: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.fields, back.fields);
    let expr_id_orig = g.find_symbol_by_name("expr").unwrap();
    let expr_id_back = back.find_symbol_by_name("expr").unwrap();
    let rules_orig = g.get_rules_for_symbol(expr_id_orig).unwrap();
    let rules_back = back.get_rules_for_symbol(expr_id_back).unwrap();
    assert_eq!(rules_orig.len(), rules_back.len());
    for (ro, rb) in rules_orig.iter().zip(rules_back.iter()) {
        assert_eq!(ro.fields, rb.fields);
    }
}

#[test]
fn builder_default_rules_have_empty_fields() {
    let g = GrammarBuilder::new("simple")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    for rule in g.all_rules() {
        assert!(rule.fields.is_empty());
    }
}

#[test]
fn builder_fields_map_is_empty_by_default() {
    let g = GrammarBuilder::new("simple")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    assert!(g.fields.is_empty());
}

#[test]
fn field_id_used_as_tuple_first_element() {
    let mapping: (FieldId, usize) = (FieldId(0), 0);
    assert_eq!(mapping.0, FieldId(0));
    assert_eq!(mapping.1, 0);
}

#[test]
fn field_id_vec_of_tuples_lookup() {
    let mappings = vec![(FieldId(0), 0usize), (FieldId(1), 1), (FieldId(2), 2)];
    let found = mappings.iter().find(|&&(fid, _)| fid == FieldId(1));
    assert!(found.is_some());
    assert_eq!(found.unwrap().1, 1);
}

#[test]
fn field_id_not_found_in_empty_mappings() {
    let mappings: Vec<(FieldId, usize)> = vec![];
    let found = mappings.iter().find(|&&(fid, _)| fid == FieldId(0));
    assert!(found.is_none());
}

#[test]
fn multiple_rules_can_share_same_field_ids() {
    let r1 = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(SymbolId(1)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 1), (FieldId(2), 2)],
        production_id: ProductionId(0),
    };
    let r2 = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(3)),
            Symbol::NonTerminal(SymbolId(1)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 1), (FieldId(2), 2)],
        production_id: ProductionId(1),
    };
    // Same field IDs but different productions
    let ids1: Vec<FieldId> = r1.fields.iter().map(|&(f, _)| f).collect();
    let ids2: Vec<FieldId> = r2.fields.iter().map(|&(f, _)| f).collect();
    assert_eq!(ids1, ids2);
    assert_ne!(r1.production_id, r2.production_id);
}

#[test]
fn grammar_fields_preserve_insertion_order() {
    let mut g = Grammar::new("t".to_string());
    g.fields.insert(FieldId(5), "fifth".to_string());
    g.fields.insert(FieldId(2), "second".to_string());
    g.fields.insert(FieldId(9), "ninth".to_string());
    // IndexMap preserves insertion order
    let keys: Vec<u16> = g.fields.keys().map(|f| f.0).collect();
    assert_eq!(keys, vec![5, 2, 9]);
}

#[test]
fn grammar_all_rules_iterates_field_mappings() {
    let g = build_expr_grammar_with_fields();
    let total_field_mappings: usize = g.all_rules().map(|r| r.fields.len()).sum();
    // 2 binary rules × 3 fields + 1 unary rule × 0 fields = 6
    assert_eq!(total_field_mappings, 6);
}

#[test]
fn rule_fields_empty_for_epsilon_rule() {
    let g = GrammarBuilder::new("nullable")
        .rule("empty", vec![])
        .start("empty")
        .build();
    let empty_id = g.find_symbol_by_name("empty").unwrap();
    let rules = g.get_rules_for_symbol(empty_id).unwrap();
    for rule in rules {
        assert!(rule.fields.is_empty());
    }
}
