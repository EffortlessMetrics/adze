#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for alias handling in adze-tablegen.
//!
//! Covers:
//! - Grammar-level `alias_sequences` and `max_alias_sequence_length`
//! - ParseTable-level `alias_sequences` (Vec<Vec<Option<SymbolId>>>)
//! - ABI/validation structs: `alias_count`, `alias_map`, `alias_sequences`,
//!   `max_alias_sequence_length`
//! - LanguageBuilder / serializer / AbiLanguageBuilder defaults
//! - Interaction with production IDs and fields

use std::collections::BTreeMap;

use adze_glr_core::{GotoIndexing, LexMode, ParseTable};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{AliasSequence, Grammar, ProductionId, SymbolId};
use adze_tablegen::generate::LanguageBuilder;
use adze_tablegen::serializer::{SerializableLanguage, serialize_language};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: adze_ir::StateId = adze_ir::StateId(u16::MAX);

/// Construct a minimal ParseTable suitable for unit-level alias tests.
fn make_empty_table(states: usize, terms: usize, nonterms: usize, externals: usize) -> ParseTable {
    let states = states.max(1);
    let eof_idx = 1 + terms + externals;
    let nonterms_eff = if nonterms == 0 { 1 } else { nonterms };
    let symbol_count = eof_idx + 1 + nonterms_eff;

    let actions = vec![vec![vec![]; symbol_count]; states];
    let gotos = vec![vec![INVALID; symbol_count]; states];

    let start_symbol = SymbolId((eof_idx + 1) as u16);
    let eof_symbol = SymbolId(eof_idx as u16);
    let token_count = eof_idx - externals;

    let mut symbol_to_index: BTreeMap<SymbolId, usize> = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let mut nonterminal_to_index: BTreeMap<SymbolId, usize> = BTreeMap::new();
    nonterminal_to_index.insert(start_symbol, start_symbol.0 as usize);

    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for (symbol_id, index) in &symbol_to_index {
        index_to_symbol[*index] = *symbol_id;
    }

    let lex_modes = vec![
        LexMode {
            lex_state: 0,
            external_lex_state: 0,
        };
        states
    ];

    ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules: vec![],
        state_count: states,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        symbol_metadata: vec![],
        token_count,
        external_token_count: externals,
        eof_symbol,
        start_symbol,
        initial_state: adze_ir::StateId(0),
        lex_modes,
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        grammar: Grammar::default(),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

// =========================================================================
// 1. Grammar-level alias_sequences tests
// =========================================================================

/// A freshly created Grammar has no alias sequences.
#[test]
fn grammar_new_has_empty_alias_sequences() {
    let grammar = Grammar::new("test".to_string());
    assert!(grammar.alias_sequences.is_empty());
    assert_eq!(grammar.max_alias_sequence_length, 0);
}

/// Default Grammar also has no alias sequences.
#[test]
fn grammar_default_has_empty_alias_sequences() {
    let grammar = Grammar::default();
    assert!(grammar.alias_sequences.is_empty());
    assert_eq!(grammar.max_alias_sequence_length, 0);
}

/// Inserting a single alias sequence is reflected correctly.
#[test]
fn grammar_single_alias_sequence() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("expr_alias".to_string())],
        },
    );
    grammar.max_alias_sequence_length = 1;

    assert_eq!(grammar.alias_sequences.len(), 1);
    let seq = &grammar.alias_sequences[&ProductionId(0)];
    assert_eq!(seq.aliases.len(), 1);
    assert_eq!(seq.aliases[0], Some("expr_alias".to_string()));
}

/// Multiple alias sequences with varying lengths.
#[test]
fn grammar_multiple_alias_sequences_varying_lengths() {
    let mut grammar = Grammar::new("test".to_string());

    // Production 0: length 2
    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("a".to_string()), None],
        },
    );
    // Production 1: length 3
    grammar.alias_sequences.insert(
        ProductionId(1),
        AliasSequence {
            aliases: vec![None, Some("b".to_string()), Some("c".to_string())],
        },
    );
    grammar.max_alias_sequence_length = 3;

    assert_eq!(grammar.alias_sequences.len(), 2);
    assert_eq!(grammar.max_alias_sequence_length, 3);
    assert_eq!(grammar.alias_sequences[&ProductionId(0)].aliases.len(), 2);
    assert_eq!(grammar.alias_sequences[&ProductionId(1)].aliases.len(), 3);
}

/// All-None alias sequence (no aliases at any position).
#[test]
fn grammar_alias_sequence_all_none() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![None, None, None],
        },
    );
    grammar.max_alias_sequence_length = 3;

    let seq = &grammar.alias_sequences[&ProductionId(0)];
    assert!(seq.aliases.iter().all(|a| a.is_none()));
}

/// All-Some alias sequence (alias at every position).
#[test]
fn grammar_alias_sequence_all_some() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![
                Some("x".to_string()),
                Some("y".to_string()),
                Some("z".to_string()),
            ],
        },
    );
    grammar.max_alias_sequence_length = 3;

    let seq = &grammar.alias_sequences[&ProductionId(0)];
    assert!(seq.aliases.iter().all(|a| a.is_some()));
}

/// An empty alias sequence (length 0).
#[test]
fn grammar_alias_sequence_empty_vec() {
    let mut grammar = Grammar::new("test".to_string());
    grammar
        .alias_sequences
        .insert(ProductionId(0), AliasSequence { aliases: vec![] });
    // max_alias_sequence_length stays 0

    assert_eq!(grammar.alias_sequences[&ProductionId(0)].aliases.len(), 0);
    assert_eq!(grammar.max_alias_sequence_length, 0);
}

// =========================================================================
// 2. ParseTable-level alias_sequences tests
// =========================================================================

/// Default ParseTable has empty alias_sequences.
#[test]
fn parse_table_default_empty_alias_sequences() {
    let pt = ParseTable::default();
    assert!(pt.alias_sequences.is_empty());
}

/// ParseTable alias_sequences can hold Vec<Vec<Option<SymbolId>>>.
#[test]
fn parse_table_alias_sequences_populated() {
    let pt = ParseTable {
        alias_sequences: vec![
            vec![None, Some(SymbolId(5))],
            vec![Some(SymbolId(1)), None, Some(SymbolId(2))],
        ],
        ..ParseTable::default()
    };

    assert_eq!(pt.alias_sequences.len(), 2);
    assert_eq!(pt.alias_sequences[0].len(), 2);
    assert_eq!(pt.alias_sequences[1].len(), 3);
    assert_eq!(pt.alias_sequences[0][1], Some(SymbolId(5)));
    assert_eq!(pt.alias_sequences[1][0], Some(SymbolId(1)));
}

/// ParseTable alias_sequences with all None entries.
#[test]
fn parse_table_alias_sequences_all_none() {
    let pt = ParseTable {
        alias_sequences: vec![vec![None; 5]],
        ..ParseTable::default()
    };

    assert_eq!(pt.alias_sequences.len(), 1);
    assert!(pt.alias_sequences[0].iter().all(|a| a.is_none()));
}

/// ParseTable with many alias sequences (stress test).
#[test]
fn parse_table_many_alias_sequences() {
    let mut pt = ParseTable::default();
    let count = 100;
    for i in 0..count {
        let seq: Vec<Option<SymbolId>> = (0..=(i % 10))
            .map(|j| {
                if j % 2 == 0 {
                    Some(SymbolId(j as u16))
                } else {
                    None
                }
            })
            .collect();
        pt.alias_sequences.push(seq);
    }

    assert_eq!(pt.alias_sequences.len(), count);
    // The longest sequence should have 10+1 = 11 elements (when i%10 == 10, but
    // i%10 maxes at 9 so 10 elements).
    let max_len = pt.alias_sequences.iter().map(|s| s.len()).max().unwrap();
    assert_eq!(max_len, 10);
}

/// make_empty_table produces empty alias_sequences.
#[test]
fn make_empty_table_has_no_alias_sequences() {
    let pt = make_empty_table(2, 3, 1, 0);
    assert!(pt.alias_sequences.is_empty());
}

// =========================================================================
// 3. LanguageBuilder alias defaults
// =========================================================================

/// LanguageBuilder generates alias_count=0 for a grammar without aliases.
#[test]
fn language_builder_alias_count_zero() {
    let grammar = GrammarBuilder::new("test_alias")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();

    let table = ParseTable::default();
    let builder = LanguageBuilder::new(grammar, table);
    let lang = builder
        .generate_language()
        .expect("generate_language failed");

    assert_eq!(lang.alias_count, 0);
    assert_eq!(lang.max_alias_sequence_length, 0);
}

/// LanguageBuilder sets alias_map and alias_sequences to null.
#[test]
fn language_builder_alias_pointers_null() {
    let grammar = GrammarBuilder::new("test_ptr")
        .token("a", "a")
        .rule("root", vec!["a"])
        .start("root")
        .build();

    let table = ParseTable::default();
    let builder = LanguageBuilder::new(grammar, table);
    let lang = builder
        .generate_language()
        .expect("generate_language failed");

    assert!(lang.alias_map.is_null());
    assert!(lang.alias_sequences.is_null());
}

/// LanguageBuilder preserves alias_count=0 with multiple tokens.
#[test]
fn language_builder_many_tokens_alias_zero() {
    let grammar = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();

    let table = ParseTable::default();
    let builder = LanguageBuilder::new(grammar, table);
    let lang = builder
        .generate_language()
        .expect("generate_language failed");

    assert_eq!(lang.alias_count, 0);
    assert_eq!(lang.max_alias_sequence_length, 0);
}

/// LanguageBuilder derives alias counters from grammar alias sequences.
#[test]
fn language_builder_uses_grammar_alias_sequences_for_alias_counters() {
    let mut grammar = GrammarBuilder::new("alias_test")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();

    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("alias_x".to_string())],
        },
    );
    grammar.max_alias_sequence_length = 1;

    let table = ParseTable::default();
    let builder = LanguageBuilder::new(grammar, table);
    let lang = builder
        .generate_language()
        .expect("generate_language failed");

    // production_id_count reflects alias_sequences.len()
    assert_eq!(lang.production_id_count, 1);
    assert_eq!(lang.alias_count, 1);
    assert_eq!(lang.max_alias_sequence_length, 1);
}

/// LanguageBuilder deduplicates repeated alias names across productions.
#[test]
fn language_builder_deduplicates_alias_names() {
    let mut grammar = GrammarBuilder::new("alias_dupe")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x"])
        .start("start")
        .build();

    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![
                Some("same_alias".to_string()),
                Some("same_alias".to_string()),
            ],
        },
    );
    grammar.alias_sequences.insert(
        ProductionId(1),
        AliasSequence {
            aliases: vec![
                Some("same_alias".to_string()),
                None,
                Some("other_alias".to_string()),
            ],
        },
    );
    grammar.max_alias_sequence_length = 3;

    let table = ParseTable::default();
    let builder = LanguageBuilder::new(grammar, table);
    let lang = builder
        .generate_language()
        .expect("generate_language failed");

    assert_eq!(lang.alias_count, 2);
    assert_eq!(lang.max_alias_sequence_length, 3);
}

// =========================================================================
// 4. Serializer alias defaults
// =========================================================================

/// Serialized language has alias_count = 0 for default grammar.
#[test]
fn serializer_alias_count_zero() {
    let grammar = GrammarBuilder::new("ser_test")
        .token("t", "t")
        .rule("start", vec!["t"])
        .start("start")
        .build();
    let table = ParseTable::default();

    let json = serialize_language(&grammar, &table, None).expect("serialization failed");
    let lang: SerializableLanguage = serde_json::from_str(&json).unwrap();

    assert_eq!(lang.alias_count, 0);
}

/// Serialized language roundtrips correctly with zero alias_count.
#[test]
fn serializer_alias_count_roundtrip() {
    let grammar = GrammarBuilder::new("roundtrip")
        .token("num", r"\d+")
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let table = ParseTable::default();

    let json = serialize_language(&grammar, &table, None).expect("serialization failed");
    let lang1: SerializableLanguage = serde_json::from_str(&json).unwrap();

    // Re-serialize and compare
    let json2 = serde_json::to_string_pretty(&lang1).unwrap();
    let lang2: SerializableLanguage = serde_json::from_str(&json2).unwrap();

    assert_eq!(lang1, lang2);
    assert_eq!(lang1.alias_count, 0);
}

// =========================================================================
// 5. AliasSequence struct tests
// =========================================================================

/// AliasSequence can be constructed with mixed Some/None values.
#[test]
fn alias_sequence_mixed_values() {
    let seq = AliasSequence {
        aliases: vec![
            Some("first".to_string()),
            None,
            Some("third".to_string()),
            None,
            None,
        ],
    };
    assert_eq!(seq.aliases.len(), 5);
    assert_eq!(seq.aliases[0].as_deref(), Some("first"));
    assert_eq!(seq.aliases[1], None);
    assert_eq!(seq.aliases[2].as_deref(), Some("third"));
}

/// AliasSequence can be cloned and equality holds on aliases.
#[test]
fn alias_sequence_clone() {
    let seq = AliasSequence {
        aliases: vec![Some("a".to_string()), None],
    };
    let cloned = seq.clone();
    assert_eq!(cloned.aliases.len(), seq.aliases.len());
    for i in 0..seq.aliases.len() {
        assert_eq!(seq.aliases[i], cloned.aliases[i]);
    }
}

/// AliasSequence debug output is non-empty.
#[test]
fn alias_sequence_debug() {
    let seq = AliasSequence {
        aliases: vec![Some("debug_test".to_string())],
    };
    let debug = format!("{:?}", seq);
    assert!(debug.contains("debug_test"));
}

/// AliasSequence serializes to JSON and deserializes back.
#[test]
fn alias_sequence_serde_roundtrip() {
    let seq = AliasSequence {
        aliases: vec![Some("alpha".to_string()), None, Some("beta".to_string())],
    };
    let json = serde_json::to_string(&seq).expect("serialize failed");
    let deserialized: AliasSequence = serde_json::from_str(&json).expect("deserialize failed");

    assert_eq!(deserialized.aliases.len(), 3);
    assert_eq!(deserialized.aliases[0], Some("alpha".to_string()));
    assert_eq!(deserialized.aliases[1], None);
    assert_eq!(deserialized.aliases[2], Some("beta".to_string()));
}

// =========================================================================
// 6. Grammar with alias_sequences and max_alias_sequence_length coherence
// =========================================================================

/// max_alias_sequence_length should match the longest alias sequence.
#[test]
fn grammar_max_alias_length_matches_longest() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("a".to_string())],
        },
    );
    grammar.alias_sequences.insert(
        ProductionId(1),
        AliasSequence {
            aliases: vec![None, None, None, Some("d".to_string())],
        },
    );

    let actual_max = grammar
        .alias_sequences
        .values()
        .map(|s| s.aliases.len())
        .max()
        .unwrap_or(0);

    assert_eq!(actual_max, 4);
    // If we set max_alias_sequence_length properly:
    grammar.max_alias_sequence_length = actual_max;
    assert_eq!(grammar.max_alias_sequence_length, 4);
}

/// ProductionId ordering is preserved in alias_sequences IndexMap.
#[test]
fn grammar_alias_sequences_preserve_insertion_order() {
    let mut grammar = Grammar::new("order".to_string());
    let ids = [
        ProductionId(5),
        ProductionId(2),
        ProductionId(9),
        ProductionId(0),
    ];

    for &pid in &ids {
        grammar.alias_sequences.insert(
            pid,
            AliasSequence {
                aliases: vec![Some(format!("alias_{}", pid.0))],
            },
        );
    }

    let keys: Vec<ProductionId> = grammar.alias_sequences.keys().copied().collect();
    assert_eq!(keys, ids);
}

// =========================================================================
// 7. GrammarBuilder builds with empty aliases
// =========================================================================

/// GrammarBuilder::build() always produces empty alias_sequences.
#[test]
fn grammar_builder_produces_empty_aliases() {
    let grammar = GrammarBuilder::new("builder_test")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();

    assert!(grammar.alias_sequences.is_empty());
    assert_eq!(grammar.max_alias_sequence_length, 0);
}

/// GrammarBuilder with externals also produces empty alias_sequences.
#[test]
fn grammar_builder_with_externals_empty_aliases() {
    let grammar = GrammarBuilder::new("ext_test")
        .token("x", "x")
        .external("INDENT")
        .rule("start", vec!["x"])
        .start("start")
        .build();

    assert!(grammar.alias_sequences.is_empty());
    assert_eq!(grammar.max_alias_sequence_length, 0);
}

// =========================================================================
// 8. Grammar with alias_sequences does not break serialization
// =========================================================================

/// A grammar with populated alias_sequences can still be serialized.
#[test]
fn serializer_grammar_with_aliases_does_not_panic() {
    let mut grammar = GrammarBuilder::new("alias_ser")
        .token("t", "t")
        .rule("s", vec!["t"])
        .start("s")
        .build();

    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("aliased_t".to_string())],
        },
    );
    grammar.max_alias_sequence_length = 1;

    let table = ParseTable::default();
    let json = serialize_language(&grammar, &table, None);
    assert!(json.is_ok(), "serialization should not fail");
}

/// Serialized alias_count remains 0 even when grammar has alias_sequences
/// (serializer does not yet use alias_sequences to compute alias_count).
#[test]
fn serializer_alias_count_still_zero_with_grammar_aliases() {
    let mut grammar = GrammarBuilder::new("alias_ser2")
        .token("t", "t")
        .rule("s", vec!["t"])
        .start("s")
        .build();

    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: vec![Some("my_alias".to_string())],
        },
    );

    let table = ParseTable::default();
    let json = serialize_language(&grammar, &table, None).unwrap();
    let lang: SerializableLanguage = serde_json::from_str(&json).unwrap();
    assert_eq!(lang.alias_count, 0);
}

// =========================================================================
// 9. Large / edge-case alias sequences
// =========================================================================

/// A single alias sequence with many positions.
#[test]
fn grammar_large_alias_sequence() {
    let mut grammar = Grammar::new("large".to_string());
    let len = 256;
    let aliases: Vec<Option<String>> = (0..len)
        .map(|i| {
            if i % 3 == 0 {
                Some(format!("alias_{i}"))
            } else {
                None
            }
        })
        .collect();

    grammar.alias_sequences.insert(
        ProductionId(0),
        AliasSequence {
            aliases: aliases.clone(),
        },
    );
    grammar.max_alias_sequence_length = len;

    assert_eq!(grammar.alias_sequences[&ProductionId(0)].aliases.len(), len);
    // Verify sampling
    assert_eq!(
        grammar.alias_sequences[&ProductionId(0)].aliases[0],
        Some("alias_0".to_string())
    );
    assert_eq!(grammar.alias_sequences[&ProductionId(0)].aliases[1], None);
    assert_eq!(grammar.alias_sequences[&ProductionId(0)].aliases[2], None);
    assert_eq!(
        grammar.alias_sequences[&ProductionId(0)].aliases[3],
        Some("alias_3".to_string())
    );
}

/// Unicode alias names are supported.
#[test]
fn alias_sequence_unicode_names() {
    let seq = AliasSequence {
        aliases: vec![
            Some("日本語".to_string()),
            Some("émoji🎉".to_string()),
            None,
            Some("Ñoño".to_string()),
        ],
    };

    assert_eq!(seq.aliases[0].as_deref(), Some("日本語"));
    assert_eq!(seq.aliases[1].as_deref(), Some("émoji🎉"));
    assert_eq!(seq.aliases[3].as_deref(), Some("Ñoño"));
}

// =========================================================================
// 10. ABI struct defaults (via validation module)
// =========================================================================

/// The LanguageBuilder ABI output has version 15 and correct alias defaults.
#[test]
fn abi_language_version_and_alias_defaults() {
    let grammar = GrammarBuilder::new("abi_test")
        .token("id", "id")
        .rule("root", vec!["id"])
        .start("root")
        .build();

    let table = ParseTable::default();
    let builder = LanguageBuilder::new(grammar, table);
    let lang = builder
        .generate_language()
        .expect("generate_language failed");

    assert_eq!(lang.version, 15);
    assert_eq!(lang.alias_count, 0);
    assert_eq!(lang.max_alias_sequence_length, 0);
    assert!(lang.alias_map.is_null());
    assert!(lang.alias_sequences.is_null());
}

/// set_start_can_be_empty does not affect alias defaults.
#[test]
fn language_builder_start_empty_alias_unaffected() {
    let grammar = GrammarBuilder::new("nullable")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();

    let table = ParseTable::default();
    let mut builder = LanguageBuilder::new(grammar, table);
    builder.set_start_can_be_empty(true);

    let lang = builder
        .generate_language()
        .expect("generate_language failed");

    assert_eq!(lang.alias_count, 0);
    assert_eq!(lang.max_alias_sequence_length, 0);
    assert!(lang.alias_map.is_null());
    assert!(lang.alias_sequences.is_null());
}
