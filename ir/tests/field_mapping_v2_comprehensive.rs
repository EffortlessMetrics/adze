//! Comprehensive tests for field mapping and field_id management in adze-ir.
//!
//! Covers FieldId construction/properties, ordering, Grammar field registration,
//! field name→ID mapping, alias sequences, serialization roundtrips, builder API,
//! and edge cases.

use std::collections::{HashMap, HashSet};

use adze_ir::builder::GrammarBuilder;
use adze_ir::{AliasSequence, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId};
use indexmap::IndexMap;

// ===========================================================================
// 1. FieldId construction and properties (8 tests)
// ===========================================================================

#[test]
fn field_id_construct_zero() {
    let id = FieldId(0);
    assert_eq!(id.0, 0_u16);
}

#[test]
fn field_id_construct_nonzero() {
    let id = FieldId(7);
    assert_eq!(id.0, 7);
}

#[test]
fn field_id_construct_max_u16() {
    let id = FieldId(u16::MAX);
    assert_eq!(id.0, 65535);
}

#[test]
fn field_id_is_copy() {
    let a = FieldId(10);
    let b = a; // Copy, not move
    assert_eq!(a, b);
}

#[test]
fn field_id_debug_format() {
    let id = FieldId(42);
    let dbg = format!("{:?}", id);
    assert!(dbg.contains("42"), "Debug should contain inner value");
}

#[test]
fn field_id_display_format() {
    let id = FieldId(3);
    assert_eq!(format!("{id}"), "Field(3)");
}

#[test]
fn field_id_hash_consistency() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn compute_hash(id: FieldId) -> u64 {
        let mut h = DefaultHasher::new();
        id.hash(&mut h);
        h.finish()
    }

    let a = FieldId(99);
    let b = FieldId(99);
    assert_eq!(compute_hash(a), compute_hash(b));
}

#[test]
fn field_id_different_values_not_equal() {
    assert_ne!(FieldId(0), FieldId(1));
    assert_ne!(FieldId(100), FieldId(200));
}

// ===========================================================================
// 2. FieldId ordering and comparison (5 tests)
// ===========================================================================

#[test]
fn field_id_equality_reflexive() {
    let id = FieldId(5);
    assert_eq!(id, id);
}

#[test]
fn field_id_equality_symmetric() {
    let a = FieldId(8);
    let b = FieldId(8);
    assert_eq!(a, b);
    assert_eq!(b, a);
}

#[test]
fn field_id_inequality() {
    assert_ne!(FieldId(1), FieldId(2));
}

#[test]
fn field_id_used_as_hashmap_key() {
    let mut map = HashMap::new();
    map.insert(FieldId(0), "alpha");
    map.insert(FieldId(1), "beta");
    map.insert(FieldId(2), "gamma");
    assert_eq!(map[&FieldId(0)], "alpha");
    assert_eq!(map[&FieldId(1)], "beta");
    assert_eq!(map[&FieldId(2)], "gamma");
}

#[test]
fn field_id_in_hashset_dedup() {
    let mut set = HashSet::new();
    set.insert(FieldId(10));
    set.insert(FieldId(10));
    set.insert(FieldId(20));
    assert_eq!(set.len(), 2);
}

// ===========================================================================
// 3. Grammar field registration (8 tests)
// ===========================================================================

#[test]
fn grammar_new_has_empty_fields() {
    let g = Grammar::new("test".to_string());
    assert!(g.fields.is_empty());
}

#[test]
fn grammar_insert_single_field() {
    let mut g = Grammar::new("test".to_string());
    g.fields.insert(FieldId(0), "name".to_string());
    assert_eq!(g.fields.len(), 1);
}

#[test]
fn grammar_insert_multiple_fields_preserves_order() {
    let mut g = Grammar::new("test".to_string());
    g.fields.insert(FieldId(0), "alpha".to_string());
    g.fields.insert(FieldId(1), "beta".to_string());
    g.fields.insert(FieldId(2), "gamma".to_string());

    let names: Vec<&String> = g.fields.values().collect();
    assert_eq!(names, vec!["alpha", "beta", "gamma"]);
}

#[test]
fn grammar_field_ids_are_unique_keys() {
    let mut g = Grammar::new("test".to_string());
    g.fields.insert(FieldId(0), "first".to_string());
    g.fields.insert(FieldId(0), "replaced".to_string());
    assert_eq!(g.fields.len(), 1);
    assert_eq!(g.fields[&FieldId(0)], "replaced");
}

#[test]
fn grammar_fields_lexicographic_order_validates() {
    let mut g = Grammar::new("test".to_string());
    g.fields.insert(FieldId(0), "alpha".to_string());
    g.fields.insert(FieldId(1), "beta".to_string());
    g.fields.insert(FieldId(2), "delta".to_string());
    assert!(g.validate().is_ok());
}

#[test]
fn grammar_fields_non_lexicographic_order_fails_validation() {
    let mut g = Grammar::new("test".to_string());
    g.fields.insert(FieldId(1), "zebra".to_string());
    g.fields.insert(FieldId(0), "alpha".to_string());
    assert!(g.validate().is_err());
}

#[test]
fn grammar_fields_can_contain_underscored_names() {
    let mut g = Grammar::new("test".to_string());
    g.fields.insert(FieldId(0), "_hidden".to_string());
    g.fields.insert(FieldId(1), "visible".to_string());
    // Non-lexicographic due to underscore sorting, but IndexMap preserves insertion order
    // Validation checks insertion order matches sorted order
    // '_' < 'v' in ASCII, so this should pass
    assert!(g.validate().is_ok());
}

#[test]
fn grammar_fields_remove_by_id() {
    let mut g = Grammar::new("test".to_string());
    g.fields.insert(FieldId(0), "alpha".to_string());
    g.fields.insert(FieldId(1), "beta".to_string());
    g.fields.shift_remove(&FieldId(0));
    assert_eq!(g.fields.len(), 1);
    assert!(g.fields.get(&FieldId(0)).is_none());
}

// ===========================================================================
// 4. Field name to ID mapping (8 tests)
// ===========================================================================

#[test]
fn field_name_lookup_by_id() {
    let mut g = Grammar::new("test".to_string());
    g.fields.insert(FieldId(0), "body".to_string());
    assert_eq!(g.fields.get(&FieldId(0)).unwrap(), "body");
}

#[test]
fn field_name_lookup_missing_id_returns_none() {
    let g = Grammar::new("test".to_string());
    assert!(g.fields.get(&FieldId(99)).is_none());
}

#[test]
fn field_reverse_lookup_name_to_id() {
    let mut g = Grammar::new("test".to_string());
    g.fields.insert(FieldId(0), "condition".to_string());
    g.fields.insert(FieldId(1), "consequence".to_string());
    g.fields.insert(FieldId(2), "alternative".to_string());

    let found = g.fields.iter().find(|(_, name)| *name == "consequence");
    assert_eq!(found.map(|(id, _)| *id), Some(FieldId(1)));
}

#[test]
fn field_reverse_lookup_missing_name_returns_none() {
    let mut g = Grammar::new("test".to_string());
    g.fields.insert(FieldId(0), "body".to_string());

    let found = g.fields.iter().find(|(_, name)| *name == "nonexistent");
    assert!(found.is_none());
}

#[test]
fn field_id_values_iteration() {
    let mut g = Grammar::new("test".to_string());
    g.fields.insert(FieldId(10), "x".to_string());
    g.fields.insert(FieldId(20), "y".to_string());
    g.fields.insert(FieldId(30), "z".to_string());

    let ids: Vec<FieldId> = g.fields.keys().copied().collect();
    assert_eq!(ids, vec![FieldId(10), FieldId(20), FieldId(30)]);
}

#[test]
fn field_names_iteration() {
    let mut g = Grammar::new("test".to_string());
    g.fields.insert(FieldId(0), "left".to_string());
    g.fields.insert(FieldId(1), "operator".to_string());
    g.fields.insert(FieldId(2), "right".to_string());

    let names: Vec<&str> = g.fields.values().map(|s| s.as_str()).collect();
    assert_eq!(names, vec!["left", "operator", "right"]);
}

#[test]
fn field_mapping_contains_key_check() {
    let mut g = Grammar::new("test".to_string());
    g.fields.insert(FieldId(5), "value".to_string());
    assert!(g.fields.contains_key(&FieldId(5)));
    assert!(!g.fields.contains_key(&FieldId(6)));
}

#[test]
fn field_mapping_indexmap_preserves_insertion_order() {
    let mut fields = IndexMap::new();
    fields.insert(FieldId(3), "charlie".to_string());
    fields.insert(FieldId(1), "alpha".to_string());
    fields.insert(FieldId(2), "bravo".to_string());

    let order: Vec<u16> = fields.keys().map(|f| f.0).collect();
    assert_eq!(order, vec![3, 1, 2]);
}

// ===========================================================================
// 5. Alias sequences (8 tests)
// ===========================================================================

#[test]
fn alias_sequence_empty_grammar_has_no_aliases() {
    let g = Grammar::new("test".to_string());
    assert!(g.alias_sequences.is_empty());
}

#[test]
fn alias_sequence_single_production_no_aliases() {
    let mut g = Grammar::new("test".to_string());
    let pid = ProductionId(0);
    g.alias_sequences.insert(
        pid,
        AliasSequence {
            aliases: vec![None, None],
        },
    );
    assert_eq!(g.alias_sequences.len(), 1);
    assert!(g.alias_sequences[&pid].aliases.iter().all(|a| a.is_none()));
}

#[test]
fn alias_sequence_with_named_alias() {
    let mut g = Grammar::new("test".to_string());
    let pid = ProductionId(0);
    g.alias_sequences.insert(
        pid,
        AliasSequence {
            aliases: vec![Some("expression".to_string()), None],
        },
    );
    assert_eq!(
        g.alias_sequences[&pid].aliases[0],
        Some("expression".to_string())
    );
    assert_eq!(g.alias_sequences[&pid].aliases[1], None);
}

#[test]
fn alias_sequence_multiple_productions() {
    let mut g = Grammar::new("test".to_string());
    g.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("stmt".to_string())],
        },
    );
    g.alias_sequences.insert(
        ProductionId(1),
        AliasSequence {
            aliases: vec![None, Some("expr".to_string())],
        },
    );
    assert_eq!(g.alias_sequences.len(), 2);
}

#[test]
fn alias_sequence_max_length_tracking() {
    let mut g = Grammar::new("test".to_string());
    g.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![None, None, None],
        },
    );
    g.alias_sequences.insert(
        ProductionId(1),
        AliasSequence {
            aliases: vec![None, None, None, None, None],
        },
    );

    let max_len = g
        .alias_sequences
        .values()
        .map(|seq| seq.aliases.len())
        .max()
        .unwrap_or(0);
    g.max_alias_sequence_length = max_len;
    assert_eq!(g.max_alias_sequence_length, 5);
}

#[test]
fn alias_sequence_all_positions_aliased() {
    let seq = AliasSequence {
        aliases: vec![
            Some("a".to_string()),
            Some("b".to_string()),
            Some("c".to_string()),
        ],
    };
    assert!(seq.aliases.iter().all(|a| a.is_some()));
}

#[test]
fn alias_sequence_empty_aliases_vec() {
    let seq = AliasSequence { aliases: vec![] };
    assert!(seq.aliases.is_empty());
}

#[test]
fn alias_sequence_lookup_by_production_id() {
    let mut g = Grammar::new("test".to_string());
    let target = ProductionId(42);
    g.alias_sequences.insert(
        target,
        AliasSequence {
            aliases: vec![Some("target_alias".to_string())],
        },
    );
    assert!(g.alias_sequences.contains_key(&target));
    assert!(!g.alias_sequences.contains_key(&ProductionId(99)));
}

// ===========================================================================
// 6. Field serialization roundtrip (5 tests)
// ===========================================================================

#[test]
fn field_id_json_roundtrip() {
    let original = FieldId(42);
    let json = serde_json::to_string(&original).unwrap();
    let restored: FieldId = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn field_id_zero_json_roundtrip() {
    let original = FieldId(0);
    let json = serde_json::to_string(&original).unwrap();
    let restored: FieldId = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn field_id_max_json_roundtrip() {
    let original = FieldId(u16::MAX);
    let json = serde_json::to_string(&original).unwrap();
    let restored: FieldId = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn grammar_fields_json_roundtrip() {
    let mut g = Grammar::new("roundtrip".to_string());
    g.fields.insert(FieldId(0), "alpha".to_string());
    g.fields.insert(FieldId(1), "beta".to_string());
    g.fields.insert(FieldId(2), "gamma".to_string());

    let json = serde_json::to_string(&g).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.fields.len(), 3);
    assert_eq!(restored.fields[&FieldId(0)], "alpha");
    assert_eq!(restored.fields[&FieldId(1)], "beta");
    assert_eq!(restored.fields[&FieldId(2)], "gamma");
}

#[test]
fn alias_sequence_json_roundtrip() {
    let mut g = Grammar::new("alias_rt".to_string());
    g.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("renamed".to_string()), None, Some("other".to_string())],
        },
    );

    let json = serde_json::to_string(&g).unwrap();
    let restored: Grammar = serde_json::from_str(&json).unwrap();
    let seq = &restored.alias_sequences[&ProductionId(0)];
    assert_eq!(seq.aliases[0], Some("renamed".to_string()));
    assert_eq!(seq.aliases[1], None);
    assert_eq!(seq.aliases[2], Some("other".to_string()));
}

// ===========================================================================
// 7. Builder field API (5 tests)
// ===========================================================================

#[test]
fn builder_produces_empty_fields() {
    let g = GrammarBuilder::new("minimal")
        .token("NUM", r"\d+")
        .rule("start", vec!["NUM"])
        .start("start")
        .build();
    assert!(g.fields.is_empty());
}

#[test]
fn builder_produces_empty_alias_sequences() {
    let g = GrammarBuilder::new("minimal")
        .token("NUM", r"\d+")
        .rule("start", vec!["NUM"])
        .start("start")
        .build();
    assert!(g.alias_sequences.is_empty());
}

#[test]
fn builder_grammar_can_add_fields_after_build() {
    let mut g = GrammarBuilder::new("post_build")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    g.fields.insert(FieldId(0), "left".to_string());
    g.fields.insert(FieldId(1), "right".to_string());
    assert_eq!(g.fields.len(), 2);
    assert!(g.validate().is_ok());
}

#[test]
fn builder_grammar_can_add_alias_sequences_after_build() {
    let mut g = GrammarBuilder::new("post_build")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    g.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("number_literal".to_string())],
        },
    );
    assert_eq!(g.alias_sequences.len(), 1);
}

#[test]
fn builder_default_max_alias_sequence_length_is_zero() {
    let g = GrammarBuilder::new("check")
        .token("ID", r"[a-z]+")
        .rule("item", vec!["ID"])
        .start("item")
        .build();
    assert_eq!(g.max_alias_sequence_length, 0);
}

// ===========================================================================
// 8. Edge cases (8 tests)
// ===========================================================================

#[test]
fn edge_empty_field_name_string() {
    let mut g = Grammar::new("test".to_string());
    g.fields.insert(FieldId(0), String::new());
    assert_eq!(g.fields[&FieldId(0)], "");
}

#[test]
fn edge_field_id_zero_is_valid() {
    let id = FieldId(0);
    assert_eq!(id.0, 0);
    let json = serde_json::to_string(&id).unwrap();
    let restored: FieldId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, restored);
}

#[test]
fn edge_field_id_u16_max_is_valid() {
    let id = FieldId(u16::MAX);
    assert_eq!(format!("{id}"), "Field(65535)");
}

#[test]
fn edge_duplicate_field_names_different_ids() {
    let mut g = Grammar::new("test".to_string());
    g.fields.insert(FieldId(0), "value".to_string());
    g.fields.insert(FieldId(1), "value".to_string());
    // IndexMap allows duplicate values with different keys
    assert_eq!(g.fields.len(), 2);
    assert_eq!(g.fields[&FieldId(0)], "value");
    assert_eq!(g.fields[&FieldId(1)], "value");
}

#[test]
fn edge_many_fields() {
    let mut g = Grammar::new("many_fields".to_string());
    for i in 0..100 {
        g.fields.insert(FieldId(i), format!("field_{i:03}"));
    }
    assert_eq!(g.fields.len(), 100);
    assert_eq!(g.fields[&FieldId(0)], "field_000");
    assert_eq!(g.fields[&FieldId(99)], "field_099");
}

#[test]
fn edge_field_in_rule_maps_position() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(3)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 1)],
        production_id: ProductionId(0),
    };

    assert_eq!(rule.fields.len(), 2);
    assert_eq!(rule.fields[0], (FieldId(0), 0));
    assert_eq!(rule.fields[1], (FieldId(1), 1));
}

#[test]
fn edge_rule_field_references_match_rhs_length() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(SymbolId(3)),
            Symbol::Terminal(SymbolId(4)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 2)],
        production_id: ProductionId(0),
    };

    // All field positions should be within rhs bounds
    for &(_, pos) in &rule.fields {
        assert!(pos < rule.rhs.len());
    }
}

#[test]
fn edge_grammar_default_has_empty_fields() {
    let g = Grammar::default();
    assert!(g.fields.is_empty());
    assert!(g.alias_sequences.is_empty());
    assert_eq!(g.max_alias_sequence_length, 0);
}

// ===========================================================================
// Additional coverage: cross-cutting concerns
// ===========================================================================

#[test]
fn field_id_used_in_vec_of_tuples() {
    let mappings: Vec<(FieldId, usize)> = vec![(FieldId(0), 0), (FieldId(1), 2), (FieldId(2), 4)];
    assert_eq!(mappings.len(), 3);
    assert_eq!(mappings[1].0, FieldId(1));
    assert_eq!(mappings[1].1, 2);
}

#[test]
fn grammar_fields_and_alias_sequences_independent() {
    let mut g = Grammar::new("independent".to_string());
    g.fields.insert(FieldId(0), "body".to_string());
    g.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("alias_body".to_string())],
        },
    );
    // Fields and aliases are stored independently
    assert_eq!(g.fields.len(), 1);
    assert_eq!(g.alias_sequences.len(), 1);
}

#[test]
fn grammar_clone_preserves_fields() {
    let mut g = Grammar::new("original".to_string());
    g.fields.insert(FieldId(0), "left".to_string());
    g.fields.insert(FieldId(1), "right".to_string());

    let cloned = g.clone();
    assert_eq!(cloned.fields.len(), 2);
    assert_eq!(cloned.fields[&FieldId(0)], "left");
    assert_eq!(cloned.fields[&FieldId(1)], "right");
}

#[test]
fn grammar_clone_preserves_alias_sequences() {
    let mut g = Grammar::new("original".to_string());
    g.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("x".to_string()), None],
        },
    );

    let cloned = g.clone();
    assert_eq!(cloned.alias_sequences.len(), 1);
    assert_eq!(
        cloned.alias_sequences[&ProductionId(0)].aliases[0],
        Some("x".to_string())
    );
}

#[test]
fn field_id_collect_into_hashset() {
    let ids: HashSet<FieldId> = (0..10).map(FieldId).collect();
    assert_eq!(ids.len(), 10);
    assert!(ids.contains(&FieldId(0)));
    assert!(ids.contains(&FieldId(9)));
    assert!(!ids.contains(&FieldId(10)));
}

#[test]
fn alias_sequence_clone_independence() {
    let original = AliasSequence {
        aliases: vec![Some("a".to_string()), None],
    };
    let mut cloned = original.clone();
    cloned.aliases.push(Some("b".to_string()));
    assert_eq!(original.aliases.len(), 2);
    assert_eq!(cloned.aliases.len(), 3);
}

#[test]
fn production_id_used_as_alias_key() {
    let mut map: IndexMap<ProductionId, AliasSequence> = IndexMap::new();
    map.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![None],
        },
    );
    map.insert(
        ProductionId(1),
        AliasSequence {
            aliases: vec![Some("named".to_string())],
        },
    );
    assert_eq!(map.len(), 2);
    assert!(map[&ProductionId(1)].aliases[0].is_some());
}
