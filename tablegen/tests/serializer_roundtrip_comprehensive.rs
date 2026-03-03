#![allow(clippy::needless_range_loop)]

//! Comprehensive serialize → deserialize roundtrip tests for `adze_tablegen::serializer`.
//!
//! Covers:
//! - Serialize → deserialize → compare
//! - JSON structure validation
//! - Field completeness
//! - Special values (empty arrays, nulls, large numbers)
//! - Unicode in names
//! - Multiple grammars serialized independently
//! - Version field preservation
//! - Symbol table roundtrip
//! - Parse action encoding roundtrip

use std::collections::BTreeMap;

use adze_glr_core::{GotoIndexing, LexMode, ParseTable};
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tablegen::serializer::{SerializableLanguage, SerializableLexState, serialize_language};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: adze_ir::StateId = adze_ir::StateId(u16::MAX);

/// Construct a minimal ParseTable suitable for unit-level tests.
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

/// Build a simple grammar with the given number of tokens and rules.
fn grammar_with_tokens_and_rules(name: &str, token_names: &[&str], rule_names: &[&str]) -> Grammar {
    let mut grammar = Grammar::new(name.to_string());
    for (i, tname) in token_names.iter().enumerate() {
        grammar.tokens.insert(
            SymbolId((i + 1) as u16),
            Token {
                name: tname.to_string(),
                pattern: TokenPattern::String(tname.to_string()),
                fragile: false,
            },
        );
    }
    let base = token_names.len() + 1;
    for (i, rname) in rule_names.iter().enumerate() {
        let sid = SymbolId((base + i) as u16);
        grammar.rules.entry(sid).or_default().push(Rule {
            lhs: sid,
            rhs: vec![],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
        grammar.rule_names.insert(sid, rname.to_string());
    }
    grammar
}

// =========================================================================
// 1. Basic roundtrip: serialize → deserialize → compare
// =========================================================================

#[test]
fn roundtrip_empty_grammar() {
    let grammar = Grammar::new("empty".to_string());
    let pt = make_empty_table(1, 0, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    // Version must match ABI constant
    assert_eq!(deser.version, 15);
    // First symbol is always "end"
    assert_eq!(deser.symbol_names[0], "end");
}

#[test]
fn roundtrip_single_token_grammar() {
    let grammar = grammar_with_tokens_and_rules("single_tok", &["plus"], &[]);
    let pt = make_empty_table(1, 1, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    assert_eq!(deser.token_count, 1);
    assert!(deser.symbol_names.contains(&"plus".to_string()));
}

#[test]
fn roundtrip_tokens_and_rules() {
    let grammar = grammar_with_tokens_and_rules("arith", &["plus", "number"], &["expression"]);
    let pt = make_empty_table(2, 2, 1, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    assert_eq!(deser.token_count, 2);
    assert!(deser.symbol_names.contains(&"expression".to_string()));
    assert!(deser.symbol_names.contains(&"plus".to_string()));
    assert!(deser.symbol_names.contains(&"number".to_string()));
}

#[test]
fn roundtrip_preserves_all_counts() {
    let grammar = grammar_with_tokens_and_rules("counts", &["a", "b", "c"], &["r1", "r2"]);
    let pt = make_empty_table(3, 3, 2, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let first: SerializableLanguage = serde_json::from_str(&json).unwrap();

    // Re-serialize the deserialized struct
    let json2 = serde_json::to_string_pretty(&first).unwrap();
    let second: SerializableLanguage = serde_json::from_str(&json2).unwrap();
    assert_eq!(first, second);
}

// =========================================================================
// 2. JSON structure validation
// =========================================================================

#[test]
fn json_has_all_top_level_fields() {
    let grammar = Grammar::new("fields_check".to_string());
    let pt = make_empty_table(1, 0, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let obj = value.as_object().unwrap();

    let expected_keys = [
        "version",
        "symbol_count",
        "alias_count",
        "token_count",
        "external_token_count",
        "state_count",
        "large_state_count",
        "production_id_count",
        "field_count",
        "symbol_names",
        "field_names",
        "symbol_metadata",
        "parse_table",
        "small_parse_table_map",
        "lex_modes",
    ];
    for key in &expected_keys {
        assert!(obj.contains_key(*key), "Missing top-level key: {key}");
    }
}

#[test]
fn json_types_are_correct() {
    let grammar = Grammar::new("type_check".to_string());
    let pt = make_empty_table(1, 0, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(value["version"].is_u64());
    assert!(value["symbol_count"].is_u64());
    assert!(value["symbol_names"].is_array());
    assert!(value["field_names"].is_array());
    assert!(value["symbol_metadata"].is_array());
    assert!(value["parse_table"].is_array());
    assert!(value["small_parse_table_map"].is_array());
    assert!(value["lex_modes"].is_array());
}

#[test]
fn json_is_valid_pretty_printed() {
    let grammar = grammar_with_tokens_and_rules("pretty", &["x"], &["y"]);
    let pt = make_empty_table(1, 1, 1, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();

    // Must contain newlines (pretty-printed)
    assert!(json.contains('\n'));
    // Must parse without error
    let _: serde_json::Value = serde_json::from_str(&json).unwrap();
}

// =========================================================================
// 3. Field completeness
// =========================================================================

#[test]
fn field_names_roundtrip() {
    let mut grammar = Grammar::new("field_test".to_string());
    grammar.fields.insert(FieldId(0), "left".to_string());
    grammar.fields.insert(FieldId(1), "right".to_string());
    grammar.fields.insert(FieldId(2), "operator".to_string());
    let pt = make_empty_table(1, 0, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    assert_eq!(deser.field_count, 3);
    // Fields are sorted lexicographically
    assert_eq!(deser.field_names, vec!["left", "operator", "right"]);
}

#[test]
fn empty_fields_roundtrip() {
    let grammar = Grammar::new("no_fields".to_string());
    let pt = make_empty_table(1, 0, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    assert_eq!(deser.field_count, 0);
    assert!(deser.field_names.is_empty());
}

#[test]
fn production_id_count_matches_rules() {
    let mut grammar = Grammar::new("prod_count".to_string());
    let sid = SymbolId(10);
    // Add two productions for the same symbol
    grammar.rules.entry(sid).or_default().push(Rule {
        lhs: sid,
        rhs: vec![],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rules.entry(sid).or_default().push(Rule {
        lhs: sid,
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    grammar.rule_names.insert(sid, "stmt".to_string());
    let pt = make_empty_table(1, 0, 1, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    assert_eq!(deser.production_id_count, 2);
}

// =========================================================================
// 4. Special values: empty arrays, zeros, large numbers
// =========================================================================

#[test]
fn empty_arrays_survive_roundtrip() {
    let grammar = Grammar::new("empties".to_string());
    let pt = make_empty_table(1, 0, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    // No compressed tables → empty parse_table and small_parse_table_map
    assert!(deser.parse_table.is_empty());
    assert!(deser.small_parse_table_map.is_empty());
    assert!(deser.field_names.is_empty());
}

#[test]
fn zero_counts_roundtrip() {
    let grammar = Grammar::new("zeros".to_string());
    let pt = make_empty_table(1, 0, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    assert_eq!(deser.alias_count, 0);
    assert_eq!(deser.large_state_count, 0);
    assert_eq!(deser.external_token_count, 0);
}

#[test]
fn large_token_count_roundtrip() {
    // Grammar with many tokens
    let mut grammar = Grammar::new("large".to_string());
    for i in 1..=200 {
        grammar.tokens.insert(
            SymbolId(i),
            Token {
                name: format!("tok_{i}"),
                pattern: TokenPattern::String(format!("t{i}")),
                fragile: false,
            },
        );
    }
    let pt = make_empty_table(1, 200, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    assert_eq!(deser.token_count, 200);
    // 1 (end) + 200 tokens = 201 symbol names (+ any extra from rules/externals)
    assert!(deser.symbol_names.len() >= 201);
}

#[test]
fn many_states_roundtrip() {
    let grammar = Grammar::new("many_states".to_string());
    let pt = make_empty_table(500, 1, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    assert_eq!(deser.state_count, 500);
    assert_eq!(deser.lex_modes.len(), 500);
}

// =========================================================================
// 5. Unicode in names
// =========================================================================

#[test]
fn unicode_token_names_roundtrip() {
    let mut grammar = Grammar::new("unicode_test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "日本語".to_string(),
            pattern: TokenPattern::String("日本語".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "émoji_🎉".to_string(),
            pattern: TokenPattern::String("🎉".to_string()),
            fragile: false,
        },
    );
    let pt = make_empty_table(1, 2, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    assert!(deser.symbol_names.contains(&"日本語".to_string()));
    assert!(deser.symbol_names.contains(&"émoji_🎉".to_string()));
}

#[test]
fn unicode_field_names_roundtrip() {
    let mut grammar = Grammar::new("unicode_fields".to_string());
    grammar.fields.insert(FieldId(0), "名前".to_string());
    grammar.fields.insert(FieldId(1), "αβγ".to_string());
    let pt = make_empty_table(1, 0, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    assert_eq!(deser.field_count, 2);
    assert!(deser.field_names.contains(&"名前".to_string()));
    assert!(deser.field_names.contains(&"αβγ".to_string()));
}

#[test]
fn unicode_rule_names_roundtrip() {
    let grammar = grammar_with_tokens_and_rules("uni_rules", &[], &["عبارة", "表达式"]);
    let pt = make_empty_table(1, 0, 2, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    assert!(deser.symbol_names.contains(&"عبارة".to_string()));
    assert!(deser.symbol_names.contains(&"表达式".to_string()));
}

// =========================================================================
// 6. Multiple grammars serialized independently
// =========================================================================

#[test]
fn two_grammars_produce_different_json() {
    let g1 = grammar_with_tokens_and_rules("lang_a", &["kw_if", "kw_else"], &["stmt"]);
    let g2 = grammar_with_tokens_and_rules("lang_b", &["fn", "let", "mut"], &["item", "expr"]);
    let pt1 = make_empty_table(2, 2, 1, 0);
    let pt2 = make_empty_table(3, 3, 2, 0);

    let json1 = serialize_language(&g1, &pt1, None).unwrap();
    let json2 = serialize_language(&g2, &pt2, None).unwrap();

    assert_ne!(json1, json2);

    let d1: SerializableLanguage = serde_json::from_str(&json1).unwrap();
    let d2: SerializableLanguage = serde_json::from_str(&json2).unwrap();
    assert_ne!(d1.symbol_names, d2.symbol_names);
    assert_ne!(d1.token_count, d2.token_count);
}

#[test]
fn independent_serialization_no_cross_contamination() {
    let g1 = grammar_with_tokens_and_rules("iso_a", &["x"], &["r"]);
    let g2 = grammar_with_tokens_and_rules("iso_b", &["y", "z"], &["s", "t"]);
    let pt1 = make_empty_table(1, 1, 1, 0);
    let pt2 = make_empty_table(1, 2, 2, 0);

    let d1: SerializableLanguage =
        serde_json::from_str(&serialize_language(&g1, &pt1, None).unwrap()).unwrap();
    let d2: SerializableLanguage =
        serde_json::from_str(&serialize_language(&g2, &pt2, None).unwrap()).unwrap();

    // g1 symbols must not appear in g2 and vice versa
    assert!(!d1.symbol_names.contains(&"y".to_string()));
    assert!(!d1.symbol_names.contains(&"z".to_string()));
    assert!(!d2.symbol_names.contains(&"x".to_string()));
}

#[test]
fn same_grammar_serializes_deterministically() {
    let grammar = grammar_with_tokens_and_rules("det", &["a", "b"], &["r"]);
    let pt = make_empty_table(2, 2, 1, 0);

    let json1 = serialize_language(&grammar, &pt, None).unwrap();
    let json2 = serialize_language(&grammar, &pt, None).unwrap();
    assert_eq!(json1, json2);
}

// =========================================================================
// 7. Version field preservation
// =========================================================================

#[test]
fn version_is_tree_sitter_abi_15() {
    let grammar = Grammar::new("ver".to_string());
    let pt = make_empty_table(1, 0, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.version, 15);
}

#[test]
fn version_survives_double_roundtrip() {
    let grammar = Grammar::new("ver2".to_string());
    let pt = make_empty_table(1, 0, 0, 0);
    let json1 = serialize_language(&grammar, &pt, None).unwrap();
    let d1: SerializableLanguage = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string_pretty(&d1).unwrap();
    let d2: SerializableLanguage = serde_json::from_str(&json2).unwrap();
    assert_eq!(d1.version, d2.version);
    assert_eq!(d2.version, 15);
}

#[test]
fn version_in_json_is_numeric() {
    let grammar = Grammar::new("ver_num".to_string());
    let pt = make_empty_table(1, 0, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["version"], serde_json::json!(15));
}

// =========================================================================
// 8. Symbol table roundtrip
// =========================================================================

#[test]
fn symbol_names_start_with_end() {
    let grammar = grammar_with_tokens_and_rules("sym", &["id", "num"], &["expr"]);
    let pt = make_empty_table(1, 2, 1, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    assert_eq!(deser.symbol_names[0], "end");
}

#[test]
fn symbol_count_matches_names_length() {
    let grammar = grammar_with_tokens_and_rules("cnt", &["a", "b"], &["r1"]);
    let pt = make_empty_table(1, 2, 1, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    // symbol_count = 1 (EOF) + tokens + rules + externals
    assert_eq!(
        deser.symbol_count as usize,
        deser.symbol_names.len(),
        "symbol_count must equal symbol_names length"
    );
}

#[test]
fn symbol_metadata_length_matches_symbol_count() {
    let grammar = grammar_with_tokens_and_rules("meta", &["x", "y"], &["q"]);
    let pt = make_empty_table(1, 2, 1, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    assert_eq!(
        deser.symbol_metadata.len(),
        deser.symbol_count as usize,
        "metadata length must equal symbol_count"
    );
}

#[test]
fn symbol_ordering_tokens_by_id() {
    let mut grammar = Grammar::new("ordering".to_string());
    // Insert in reverse order
    grammar.tokens.insert(
        SymbolId(3),
        Token {
            name: "gamma".to_string(),
            pattern: TokenPattern::String("g".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "alpha".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "beta".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );
    let pt = make_empty_table(1, 3, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    // Tokens should be ordered by SymbolId, not insertion order
    assert_eq!(deser.symbol_names[0], "end");
    assert_eq!(deser.symbol_names[1], "alpha");
    assert_eq!(deser.symbol_names[2], "beta");
    assert_eq!(deser.symbol_names[3], "gamma");
}

#[test]
fn external_tokens_appear_in_symbol_names() {
    let mut grammar = Grammar::new("ext_test".to_string());
    grammar.externals.push(ExternalToken {
        name: "indent".to_string(),
        symbol_id: SymbolId(100),
    });
    grammar.externals.push(ExternalToken {
        name: "dedent".to_string(),
        symbol_id: SymbolId(101),
    });
    let pt = make_empty_table(1, 0, 0, 2);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    assert!(deser.symbol_names.contains(&"indent".to_string()));
    assert!(deser.symbol_names.contains(&"dedent".to_string()));
    assert_eq!(deser.external_token_count, 2);
}

// =========================================================================
// 9. Parse action encoding roundtrip (via SerializableLanguage)
// =========================================================================

#[test]
fn no_compressed_tables_yields_empty_parse_table() {
    let grammar = Grammar::new("no_compress".to_string());
    let pt = make_empty_table(1, 0, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    assert!(deser.parse_table.is_empty());
    assert!(deser.small_parse_table_map.is_empty());
}

#[test]
fn lex_modes_roundtrip_count() {
    let grammar = Grammar::new("lex_modes".to_string());
    let pt = make_empty_table(5, 1, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    assert_eq!(deser.lex_modes.len(), 5);
    for (i, mode) in deser.lex_modes.iter().enumerate() {
        assert_eq!(mode.lex_state, i as u16);
        assert_eq!(mode.external_lex_state, 0);
    }
}

#[test]
fn lex_state_serde_roundtrip() {
    let state = SerializableLexState {
        lex_state: 42,
        external_lex_state: 7,
    };
    let json = serde_json::to_string(&state).unwrap();
    let deser: SerializableLexState = serde_json::from_str(&json).unwrap();
    assert_eq!(state, deser);
}

#[test]
fn serializable_language_direct_serde_roundtrip() {
    let lang = SerializableLanguage {
        version: 15,
        symbol_count: 3,
        alias_count: 0,
        token_count: 1,
        external_token_count: 0,
        state_count: 2,
        large_state_count: 0,
        production_id_count: 1,
        field_count: 1,
        symbol_names: vec!["end".into(), "tok".into(), "rule".into()],
        field_names: vec!["name".into()],
        symbol_metadata: vec![1, 2, 3],
        parse_table: vec![100, 200, 300],
        small_parse_table_map: vec![0, 10],
        lex_modes: vec![
            SerializableLexState {
                lex_state: 0,
                external_lex_state: 0,
            },
            SerializableLexState {
                lex_state: 1,
                external_lex_state: 0,
            },
        ],
    };

    let json = serde_json::to_string_pretty(&lang).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();
    assert_eq!(lang, deser);
}

// =========================================================================
// 10. Edge cases
// =========================================================================

#[test]
fn hidden_token_name_underscore_prefix() {
    let mut grammar = Grammar::new("hidden".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "_whitespace".to_string(),
            pattern: TokenPattern::Regex(r"\s+".to_string()),
            fragile: false,
        },
    );
    let pt = make_empty_table(1, 1, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    assert!(deser.symbol_names.contains(&"_whitespace".to_string()));
    // Hidden tokens (underscore prefix) have visible=false in metadata
    // EOF is at index 0, _whitespace is at index 1
    let ws_meta = deser.symbol_metadata[1];
    // visible bit is 0x01 — hidden tokens should NOT have it set
    assert_eq!(ws_meta & 0x01, 0, "_whitespace should not be visible");
}

#[test]
fn supertype_symbol_metadata_roundtrip() {
    let mut grammar = Grammar::new("supertype".to_string());
    let sid = SymbolId(10);
    grammar.rules.entry(sid).or_default().push(Rule {
        lhs: sid,
        rhs: vec![],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(sid, "expression".to_string());
    grammar.supertypes.push(sid);

    let pt = make_empty_table(1, 0, 1, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    // Find the index for "expression"
    let idx = deser
        .symbol_names
        .iter()
        .position(|n| n == "expression")
        .expect("expression should be in symbol_names");
    let meta = deser.symbol_metadata[idx];
    // supertype bit is 0x10
    assert_ne!(meta & 0x10, 0, "expression should have supertype bit set");
}

#[test]
fn regex_token_is_named() {
    let mut grammar = Grammar::new("named_check".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    let pt = make_empty_table(1, 2, 0, 0);
    let json = serialize_language(&grammar, &pt, None).unwrap();
    let deser: SerializableLanguage = serde_json::from_str(&json).unwrap();

    // identifier (regex, visible) → named=true (bit 0x02)
    let id_idx = deser
        .symbol_names
        .iter()
        .position(|n| n == "identifier")
        .unwrap();
    assert_ne!(
        deser.symbol_metadata[id_idx] & 0x02,
        0,
        "regex token should be named"
    );

    // plus (string literal, visible) → named=false
    let plus_idx = deser.symbol_names.iter().position(|n| n == "plus").unwrap();
    assert_eq!(
        deser.symbol_metadata[plus_idx] & 0x02,
        0,
        "string token should not be named"
    );
}

#[test]
fn full_double_roundtrip_equality() {
    // Build a non-trivial grammar, roundtrip twice, verify equality
    let mut grammar = grammar_with_tokens_and_rules(
        "double_rt",
        &["if", "else", "while", "identifier"],
        &["statement", "block", "program"],
    );
    grammar.fields.insert(FieldId(0), "condition".to_string());
    grammar.fields.insert(FieldId(1), "body".to_string());
    grammar.externals.push(ExternalToken {
        name: "newline".to_string(),
        symbol_id: SymbolId(200),
    });

    let pt = make_empty_table(4, 4, 3, 1);
    let json1 = serialize_language(&grammar, &pt, None).unwrap();
    let d1: SerializableLanguage = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string_pretty(&d1).unwrap();
    let d2: SerializableLanguage = serde_json::from_str(&json2).unwrap();
    let json3 = serde_json::to_string_pretty(&d2).unwrap();
    let d3: SerializableLanguage = serde_json::from_str(&json3).unwrap();

    assert_eq!(d1, d2);
    assert_eq!(d2, d3);
    assert_eq!(json2, json3);
}
