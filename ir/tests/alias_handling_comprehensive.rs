#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for AliasSequence and alias handling in adze-ir.
//!
//! Covers: AliasSequence construction, multiple aliases per symbol,
//! alias with production ID mapping, alias display/debug, alias in
//! grammar context, alias serde roundtrip, empty alias sequences,
//! and alias equality.

use adze_ir::{AliasSequence, Grammar, ProductionId, RuleId};

// ---------------------------------------------------------------------------
// AliasSequence construction
// ---------------------------------------------------------------------------

#[test]
fn alias_sequence_construct_empty_aliases_vec() {
    let seq = AliasSequence { aliases: vec![] };
    assert!(seq.aliases.is_empty());
    assert_eq!(seq.aliases.len(), 0);
}

#[test]
fn alias_sequence_construct_single_some() {
    let seq = AliasSequence {
        aliases: vec![Some("expr".to_string())],
    };
    assert_eq!(seq.aliases.len(), 1);
    assert_eq!(seq.aliases[0].as_deref(), Some("expr"));
}

#[test]
fn alias_sequence_construct_single_none() {
    let seq = AliasSequence {
        aliases: vec![None],
    };
    assert_eq!(seq.aliases.len(), 1);
    assert!(seq.aliases[0].is_none());
}

#[test]
fn alias_sequence_construct_mixed() {
    let seq = AliasSequence {
        aliases: vec![
            Some("statement".to_string()),
            None,
            Some("expression".to_string()),
            None,
            None,
        ],
    };
    assert_eq!(seq.aliases.len(), 5);
    assert_eq!(seq.aliases[0].as_deref(), Some("statement"));
    assert!(seq.aliases[1].is_none());
    assert_eq!(seq.aliases[2].as_deref(), Some("expression"));
    for i in 3..5 {
        assert!(seq.aliases[i].is_none());
    }
}

#[test]
fn alias_sequence_construct_all_some() {
    let names: Vec<String> = (0..4).map(|i| format!("alias_{i}")).collect();
    let seq = AliasSequence {
        aliases: names.iter().map(|n| Some(n.clone())).collect(),
    };
    for i in 0..4 {
        assert_eq!(seq.aliases[i].as_deref(), Some(names[i].as_str()));
    }
}

#[test]
fn alias_sequence_construct_all_none() {
    let seq = AliasSequence {
        aliases: vec![None; 6],
    };
    assert_eq!(seq.aliases.len(), 6);
    for i in 0..6 {
        assert!(seq.aliases[i].is_none());
    }
}

// ---------------------------------------------------------------------------
// Multiple aliases per symbol (multiple productions)
// ---------------------------------------------------------------------------

#[test]
fn multiple_alias_sequences_in_grammar() {
    let mut grammar = Grammar::new("multi_alias".to_string());
    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("call_expr".to_string()), None],
        },
    );
    grammar.alias_sequences.insert(
        ProductionId(1),
        AliasSequence {
            aliases: vec![None, Some("arg_list".to_string())],
        },
    );
    assert_eq!(grammar.alias_sequences.len(), 2);
    assert_eq!(
        grammar.alias_sequences[&ProductionId(0)].aliases[0].as_deref(),
        Some("call_expr")
    );
    assert_eq!(
        grammar.alias_sequences[&ProductionId(1)].aliases[1].as_deref(),
        Some("arg_list")
    );
}

#[test]
fn same_alias_name_different_positions() {
    let seq = AliasSequence {
        aliases: vec![Some("ident".to_string()), None, Some("ident".to_string())],
    };
    assert_eq!(seq.aliases[0], seq.aliases[2]);
    assert_ne!(seq.aliases[0], seq.aliases[1]);
}

// ---------------------------------------------------------------------------
// Alias with production ID mapping
// ---------------------------------------------------------------------------

#[test]
fn production_id_maps_to_alias_sequence() {
    let mut grammar = Grammar::new("prod_alias".to_string());
    let rid = RuleId(5);
    let pid = ProductionId(3);
    grammar.production_ids.insert(rid, pid);
    grammar.alias_sequences.insert(
        pid,
        AliasSequence {
            aliases: vec![Some("binary_expr".to_string())],
        },
    );

    let resolved_pid = grammar.production_ids[&rid];
    let seq = &grammar.alias_sequences[&resolved_pid];
    assert_eq!(seq.aliases[0].as_deref(), Some("binary_expr"));
}

#[test]
fn multiple_rule_ids_map_to_same_production_id() {
    let mut grammar = Grammar::new("shared_prod".to_string());
    let pid = ProductionId(10);
    grammar.production_ids.insert(RuleId(0), pid);
    grammar.production_ids.insert(RuleId(1), pid);
    grammar.alias_sequences.insert(
        pid,
        AliasSequence {
            aliases: vec![Some("shared_alias".to_string())],
        },
    );

    assert_eq!(
        grammar.production_ids[&RuleId(0)],
        grammar.production_ids[&RuleId(1)]
    );
    let seq = &grammar.alias_sequences[&pid];
    assert_eq!(seq.aliases[0].as_deref(), Some("shared_alias"));
}

#[test]
fn production_id_without_alias_sequence() {
    let mut grammar = Grammar::new("no_alias".to_string());
    grammar.production_ids.insert(RuleId(0), ProductionId(99));
    // No alias_sequence inserted for ProductionId(99)
    assert!(!grammar.alias_sequences.contains_key(&ProductionId(99)));
}

// ---------------------------------------------------------------------------
// Alias display/debug
// ---------------------------------------------------------------------------

#[test]
fn alias_sequence_debug_empty() {
    let seq = AliasSequence { aliases: vec![] };
    let dbg = format!("{:?}", seq);
    assert!(dbg.contains("AliasSequence"));
    assert!(dbg.contains("aliases"));
}

#[test]
fn alias_sequence_debug_with_values() {
    let seq = AliasSequence {
        aliases: vec![Some("foo".to_string()), None],
    };
    let dbg = format!("{:?}", seq);
    assert!(dbg.contains("foo"));
    assert!(dbg.contains("None"));
}

#[test]
fn production_id_display() {
    let pid = ProductionId(42);
    assert_eq!(format!("{pid}"), "Production(42)");
}

#[test]
fn rule_id_display() {
    let rid = RuleId(7);
    assert_eq!(format!("{rid}"), "Rule(7)");
}

// ---------------------------------------------------------------------------
// Alias in grammar context
// ---------------------------------------------------------------------------

#[test]
fn grammar_new_has_empty_alias_sequences() {
    let grammar = Grammar::new("empty".to_string());
    assert!(grammar.alias_sequences.is_empty());
    assert_eq!(grammar.max_alias_sequence_length, 0);
}

#[test]
fn grammar_max_alias_sequence_length_tracks_longest() {
    let mut grammar = Grammar::new("tracking".to_string());
    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![None, Some("a".to_string())],
        },
    );
    grammar.alias_sequences.insert(
        ProductionId(1),
        AliasSequence {
            aliases: vec![None; 5],
        },
    );
    // Manually set max_alias_sequence_length as the Grammar struct expects the caller to maintain it
    grammar.max_alias_sequence_length = grammar
        .alias_sequences
        .values()
        .map(|s| s.aliases.len())
        .max()
        .unwrap_or(0);
    assert_eq!(grammar.max_alias_sequence_length, 5);
}

#[test]
fn grammar_alias_sequences_insertion_order_preserved() {
    let mut grammar = Grammar::new("order".to_string());
    let pids = [ProductionId(3), ProductionId(1), ProductionId(7)];
    for pid in &pids {
        grammar.alias_sequences.insert(
            *pid,
            AliasSequence {
                aliases: vec![Some(format!("alias_{}", pid.0))],
            },
        );
    }
    let keys: Vec<ProductionId> = grammar.alias_sequences.keys().copied().collect();
    assert_eq!(keys, pids.to_vec());
}

#[test]
fn grammar_default_has_empty_alias_fields() {
    let grammar = Grammar::default();
    assert!(grammar.alias_sequences.is_empty());
    assert!(grammar.production_ids.is_empty());
    assert_eq!(grammar.max_alias_sequence_length, 0);
}

// ---------------------------------------------------------------------------
// Alias serde roundtrip
// ---------------------------------------------------------------------------

#[test]
fn alias_sequence_serde_roundtrip_empty() {
    let seq = AliasSequence { aliases: vec![] };
    let json = serde_json::to_string(&seq).unwrap();
    let deser: AliasSequence = serde_json::from_str(&json).unwrap();
    assert_eq!(seq, deser);
}

#[test]
fn alias_sequence_serde_roundtrip_mixed() {
    let seq = AliasSequence {
        aliases: vec![
            Some("declaration".to_string()),
            None,
            Some("identifier".to_string()),
        ],
    };
    let json = serde_json::to_string(&seq).unwrap();
    let deser: AliasSequence = serde_json::from_str(&json).unwrap();
    assert_eq!(seq, deser);
}

#[test]
fn alias_sequence_serde_roundtrip_all_none() {
    let seq = AliasSequence {
        aliases: vec![None, None, None],
    };
    let json = serde_json::to_string(&seq).unwrap();
    let deser: AliasSequence = serde_json::from_str(&json).unwrap();
    assert_eq!(seq, deser);
}

#[test]
fn grammar_alias_sequences_serde_roundtrip() {
    let mut grammar = Grammar::new("roundtrip".to_string());
    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("type_alias".to_string()), None],
        },
    );
    grammar.alias_sequences.insert(
        ProductionId(5),
        AliasSequence {
            aliases: vec![None, None, Some("param".to_string())],
        },
    );
    grammar.production_ids.insert(RuleId(0), ProductionId(0));
    grammar.production_ids.insert(RuleId(1), ProductionId(5));
    grammar.max_alias_sequence_length = 3;

    let json = serde_json::to_string_pretty(&grammar).unwrap();
    let deser: Grammar = serde_json::from_str(&json).unwrap();

    assert_eq!(grammar.alias_sequences, deser.alias_sequences);
    assert_eq!(grammar.production_ids, deser.production_ids);
    assert_eq!(
        grammar.max_alias_sequence_length,
        deser.max_alias_sequence_length
    );
}

#[test]
fn production_id_serde_roundtrip() {
    let pid = ProductionId(255);
    let json = serde_json::to_string(&pid).unwrap();
    let deser: ProductionId = serde_json::from_str(&json).unwrap();
    assert_eq!(pid, deser);
}

// ---------------------------------------------------------------------------
// Empty alias sequences
// ---------------------------------------------------------------------------

#[test]
fn empty_alias_sequence_is_no_op() {
    let seq = AliasSequence { aliases: vec![] };
    assert!(seq.aliases.iter().all(|a| a.is_none()));
    assert_eq!(seq.aliases.iter().flatten().count(), 0);
}

#[test]
fn all_none_alias_sequence_has_no_effective_aliases() {
    let seq = AliasSequence {
        aliases: vec![None, None, None, None],
    };
    assert_eq!(seq.aliases.iter().flatten().count(), 0);
}

// ---------------------------------------------------------------------------
// Alias equality
// ---------------------------------------------------------------------------

#[test]
fn alias_sequence_equality_same() {
    let a = AliasSequence {
        aliases: vec![Some("x".to_string()), None],
    };
    let b = AliasSequence {
        aliases: vec![Some("x".to_string()), None],
    };
    assert_eq!(a, b);
}

#[test]
fn alias_sequence_inequality_different_values() {
    let a = AliasSequence {
        aliases: vec![Some("x".to_string())],
    };
    let b = AliasSequence {
        aliases: vec![Some("y".to_string())],
    };
    assert_ne!(a, b);
}

#[test]
fn alias_sequence_inequality_different_lengths() {
    let a = AliasSequence {
        aliases: vec![Some("x".to_string())],
    };
    let b = AliasSequence {
        aliases: vec![Some("x".to_string()), None],
    };
    assert_ne!(a, b);
}

#[test]
fn alias_sequence_inequality_some_vs_none() {
    let a = AliasSequence {
        aliases: vec![Some("x".to_string())],
    };
    let b = AliasSequence {
        aliases: vec![None],
    };
    assert_ne!(a, b);
}

#[test]
fn alias_sequence_clone_equals_original() {
    let original = AliasSequence {
        aliases: vec![Some("cloned".to_string()), None, Some("value".to_string())],
    };
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn production_id_equality() {
    assert_eq!(ProductionId(0), ProductionId(0));
    assert_ne!(ProductionId(0), ProductionId(1));
}

#[test]
fn production_id_ordering() {
    assert!(ProductionId(0) < ProductionId(1));
    assert!(ProductionId(100) > ProductionId(99));
}

#[test]
fn production_id_hash_consistent() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert(ProductionId(1), "first");
    map.insert(ProductionId(2), "second");
    assert_eq!(map[&ProductionId(1)], "first");
    assert_eq!(map[&ProductionId(2)], "second");
    assert!(!map.contains_key(&ProductionId(3)));
}
