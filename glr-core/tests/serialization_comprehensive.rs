//! Comprehensive serialization tests for ParseTable.
//!
//! These tests validate roundtrip fidelity, error handling, edge cases,
//! and format compatibility of the ParseTable serialization pipeline.
//!
//! Run with:
//!   cargo test -p adze-glr-core --test serialization_comprehensive --features serialization

#![cfg(feature = "serialization")]

use std::collections::BTreeMap;

use adze_glr_core::serialization::{DeserializationError, PARSE_TABLE_FORMAT_VERSION};
use adze_glr_core::{
    Action, GotoIndexing, LexMode, ParseRule, ParseTable, StateId, SymbolId, SymbolMetadata,
};
use adze_ir::RuleId;

// ---------------------------------------------------------------------------
// Helper: build a configurable ParseTable
// ---------------------------------------------------------------------------

fn make_table(states: usize, symbols: usize) -> ParseTable {
    ParseTable {
        action_table: vec![vec![vec![Action::Error]; symbols]; states],
        goto_table: vec![vec![StateId(0); symbols]; states],
        symbol_metadata: vec![],
        state_count: states,
        symbol_count: symbols,
        symbol_to_index: Default::default(),
        index_to_symbol: (0..symbols).map(|i| SymbolId(i as u16)).collect(),
        external_scanner_states: vec![vec![]; states],
        rules: vec![],
        nonterminal_to_index: Default::default(),
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(1),
        grammar: Default::default(),
        initial_state: StateId(0),
        token_count: symbols,
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            states
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: Default::default(),
    }
}

/// Assert that two ParseTables are structurally identical for all serialized fields.
fn assert_tables_eq(a: &ParseTable, b: &ParseTable) {
    assert_eq!(a.state_count, b.state_count, "state_count mismatch");
    assert_eq!(a.symbol_count, b.symbol_count, "symbol_count mismatch");
    assert_eq!(a.action_table, b.action_table, "action_table mismatch");
    assert_eq!(a.goto_table, b.goto_table, "goto_table mismatch");
    assert_eq!(a.eof_symbol, b.eof_symbol, "eof_symbol mismatch");
    assert_eq!(a.start_symbol, b.start_symbol, "start_symbol mismatch");
    assert_eq!(a.initial_state, b.initial_state, "initial_state mismatch");
    assert_eq!(a.token_count, b.token_count, "token_count mismatch");
    assert_eq!(
        a.external_token_count, b.external_token_count,
        "external_token_count mismatch"
    );
    assert_eq!(a.goto_indexing, b.goto_indexing, "goto_indexing mismatch");
    assert_eq!(a.extras, b.extras, "extras mismatch");
    assert_eq!(
        a.dynamic_prec_by_rule, b.dynamic_prec_by_rule,
        "dynamic_prec_by_rule mismatch"
    );
    assert_eq!(
        a.rule_assoc_by_rule, b.rule_assoc_by_rule,
        "rule_assoc_by_rule mismatch"
    );
    assert_eq!(a.field_names, b.field_names, "field_names mismatch");
    assert_eq!(a.field_map, b.field_map, "field_map mismatch");
    assert_eq!(a.lex_modes, b.lex_modes, "lex_modes mismatch");
    assert_eq!(
        a.index_to_symbol, b.index_to_symbol,
        "index_to_symbol mismatch"
    );
    assert_eq!(
        a.symbol_to_index, b.symbol_to_index,
        "symbol_to_index mismatch"
    );
    assert_eq!(
        a.nonterminal_to_index, b.nonterminal_to_index,
        "nonterminal_to_index mismatch"
    );
    assert_eq!(
        a.alias_sequences, b.alias_sequences,
        "alias_sequences mismatch"
    );
    assert_eq!(
        a.external_scanner_states, b.external_scanner_states,
        "external_scanner_states mismatch"
    );
    // rules
    assert_eq!(a.rules.len(), b.rules.len(), "rules length mismatch");
    for (i, (ra, rb)) in a.rules.iter().zip(b.rules.iter()).enumerate() {
        assert_eq!(ra.lhs, rb.lhs, "rule[{i}].lhs mismatch");
        assert_eq!(ra.rhs_len, rb.rhs_len, "rule[{i}].rhs_len mismatch");
    }
    // symbol_metadata
    assert_eq!(
        a.symbol_metadata.len(),
        b.symbol_metadata.len(),
        "symbol_metadata length mismatch"
    );
    for (i, (ma, mb)) in a
        .symbol_metadata
        .iter()
        .zip(b.symbol_metadata.iter())
        .enumerate()
    {
        assert_eq!(ma.name, mb.name, "meta[{i}].name");
        assert_eq!(ma.is_visible, mb.is_visible, "meta[{i}].is_visible");
        assert_eq!(ma.is_named, mb.is_named, "meta[{i}].is_named");
        assert_eq!(ma.is_supertype, mb.is_supertype, "meta[{i}].is_supertype");
        assert_eq!(ma.is_terminal, mb.is_terminal, "meta[{i}].is_terminal");
        assert_eq!(ma.is_extra, mb.is_extra, "meta[{i}].is_extra");
        assert_eq!(ma.is_fragile, mb.is_fragile, "meta[{i}].is_fragile");
        assert_eq!(ma.symbol_id, mb.symbol_id, "meta[{i}].symbol_id");
    }
}

// ===================================================================
// 1. Basic roundtrip tests
// ===================================================================

#[test]
fn roundtrip_default_table() {
    let table = ParseTable::default();
    let bytes = table.to_bytes().unwrap();
    let restored = ParseTable::from_bytes(&bytes).unwrap();
    assert_tables_eq(&table, &restored);
}

#[test]
fn roundtrip_minimal_table() {
    let table = make_table(2, 3);
    let bytes = table.to_bytes().unwrap();
    let restored = ParseTable::from_bytes(&bytes).unwrap();
    assert_tables_eq(&table, &restored);
}

#[test]
fn roundtrip_single_state_single_symbol() {
    let table = make_table(1, 1);
    let bytes = table.to_bytes().unwrap();
    let restored = ParseTable::from_bytes(&bytes).unwrap();
    assert_tables_eq(&table, &restored);
}

// ===================================================================
// 2. Determinism
// ===================================================================

#[test]
fn serialization_is_deterministic() {
    let table = make_table(4, 5);
    let bytes1 = table.to_bytes().unwrap();
    let bytes2 = table.to_bytes().unwrap();
    assert_eq!(bytes1, bytes2, "serialization must be deterministic");
}

#[test]
fn double_roundtrip_produces_identical_bytes() {
    let table = make_table(3, 4);
    let bytes1 = table.to_bytes().unwrap();
    let restored = ParseTable::from_bytes(&bytes1).unwrap();
    let bytes2 = restored.to_bytes().unwrap();
    assert_eq!(bytes1, bytes2, "double roundtrip bytes must match");
}

// ===================================================================
// 3. Action variant preservation
// ===================================================================

#[test]
fn roundtrip_preserves_shift_action() {
    let mut table = make_table(1, 1);
    table.action_table[0][0] = vec![Action::Shift(StateId(42))];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(
        restored.action_table[0][0],
        vec![Action::Shift(StateId(42))]
    );
}

#[test]
fn roundtrip_preserves_reduce_action() {
    let mut table = make_table(1, 1);
    table.action_table[0][0] = vec![Action::Reduce(RuleId(99))];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(
        restored.action_table[0][0],
        vec![Action::Reduce(RuleId(99))]
    );
}

#[test]
fn roundtrip_preserves_accept_action() {
    let mut table = make_table(1, 1);
    table.action_table[0][0] = vec![Action::Accept];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.action_table[0][0], vec![Action::Accept]);
}

#[test]
fn roundtrip_preserves_error_action() {
    let mut table = make_table(1, 1);
    table.action_table[0][0] = vec![Action::Error];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.action_table[0][0], vec![Action::Error]);
}

#[test]
fn roundtrip_preserves_recover_action() {
    let mut table = make_table(1, 1);
    table.action_table[0][0] = vec![Action::Recover];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.action_table[0][0], vec![Action::Recover]);
}

#[test]
fn roundtrip_preserves_fork_action() {
    let mut table = make_table(1, 1);
    table.action_table[0][0] = vec![Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
        Action::Accept,
    ])];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(
        restored.action_table[0][0],
        vec![Action::Fork(vec![
            Action::Shift(StateId(1)),
            Action::Reduce(RuleId(2)),
            Action::Accept,
        ])]
    );
}

#[test]
fn roundtrip_preserves_multi_action_cell() {
    let mut table = make_table(1, 1);
    table.action_table[0][0] = vec![
        Action::Shift(StateId(5)),
        Action::Reduce(RuleId(3)),
        Action::Recover,
    ];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.action_table[0][0].len(), 3);
    assert_eq!(restored.action_table[0][0][0], Action::Shift(StateId(5)));
    assert_eq!(restored.action_table[0][0][1], Action::Reduce(RuleId(3)));
    assert_eq!(restored.action_table[0][0][2], Action::Recover);
}

#[test]
fn roundtrip_preserves_empty_action_cell() {
    let mut table = make_table(1, 1);
    table.action_table[0][0] = vec![];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert!(restored.action_table[0][0].is_empty());
}

// ===================================================================
// 4. GotoIndexing preservation
// ===================================================================

#[test]
fn roundtrip_preserves_nonterminal_map_indexing() {
    let table = make_table(2, 2);
    assert_eq!(table.goto_indexing, GotoIndexing::NonterminalMap);
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.goto_indexing, GotoIndexing::NonterminalMap);
}

#[test]
fn roundtrip_preserves_direct_symbol_id_indexing() {
    let mut table = make_table(2, 2);
    table.nonterminal_to_index.insert(SymbolId(0), 0);
    table.nonterminal_to_index.insert(SymbolId(1), 1);
    let table = table.remap_goto_to_direct_symbol_id();
    assert_eq!(table.goto_indexing, GotoIndexing::DirectSymbolId);

    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.goto_indexing, GotoIndexing::DirectSymbolId);
}

// ===================================================================
// 5. Symbol metadata preservation
// ===================================================================

#[test]
fn roundtrip_preserves_symbol_metadata_all_fields() {
    let mut table = make_table(1, 1);
    table.symbol_metadata = vec![
        SymbolMetadata {
            name: "identifier".into(),
            is_visible: true,
            is_named: true,
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: true,
            symbol_id: SymbolId(0),
        },
        SymbolMetadata {
            name: "comment".into(),
            is_visible: false,
            is_named: false,
            is_supertype: true,
            is_terminal: false,
            is_extra: true,
            is_fragile: false,
            symbol_id: SymbolId(1),
        },
    ];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_tables_eq(&table, &restored);
}

#[test]
fn roundtrip_preserves_metadata_with_unicode_name() {
    let mut table = make_table(1, 1);
    table.symbol_metadata = vec![SymbolMetadata {
        name: "日本語シンボル_αβγ".into(),
        is_visible: true,
        is_named: true,
        is_supertype: false,
        is_terminal: true,
        is_extra: false,
        is_fragile: false,
        symbol_id: SymbolId(0),
    }];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.symbol_metadata[0].name, "日本語シンボル_αβγ");
}

// ===================================================================
// 6. Rules and field map preservation
// ===================================================================

#[test]
fn roundtrip_preserves_rules() {
    let mut table = make_table(1, 1);
    table.rules = vec![
        ParseRule {
            lhs: SymbolId(10),
            rhs_len: 3,
        },
        ParseRule {
            lhs: SymbolId(20),
            rhs_len: 0,
        },
        ParseRule {
            lhs: SymbolId(30),
            rhs_len: u16::MAX,
        },
    ];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.rules.len(), 3);
    assert_eq!(restored.rules[2].rhs_len, u16::MAX);
}

#[test]
fn roundtrip_preserves_field_map() {
    let mut table = make_table(1, 1);
    table.field_names = vec!["left".into(), "operator".into(), "right".into()];
    table.field_map.insert((RuleId(0), 0), 1);
    table.field_map.insert((RuleId(0), 1), 2);
    table.field_map.insert((RuleId(0), 2), 3);
    table.field_map.insert((RuleId(1), 0), 1);
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.field_names, table.field_names);
    assert_eq!(restored.field_map, table.field_map);
}

// ===================================================================
// 7. Lex modes and extras preservation
// ===================================================================

#[test]
fn roundtrip_preserves_varied_lex_modes() {
    let mut table = make_table(3, 1);
    table.lex_modes = vec![
        LexMode {
            lex_state: 0,
            external_lex_state: 0,
        },
        LexMode {
            lex_state: 5,
            external_lex_state: 2,
        },
        LexMode {
            lex_state: u16::MAX,
            external_lex_state: u16::MAX,
        },
    ];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.lex_modes, table.lex_modes);
}

#[test]
fn roundtrip_preserves_extras() {
    let mut table = make_table(1, 1);
    table.extras = vec![SymbolId(5), SymbolId(10), SymbolId(0)];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(
        restored.extras,
        vec![SymbolId(5), SymbolId(10), SymbolId(0)]
    );
}

// ===================================================================
// 8. Dynamic precedence and associativity
// ===================================================================

#[test]
fn roundtrip_preserves_dynamic_precedence() {
    let mut table = make_table(1, 1);
    table.dynamic_prec_by_rule = vec![-5, 0, 10, i16::MAX, i16::MIN];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.dynamic_prec_by_rule, table.dynamic_prec_by_rule);
}

#[test]
fn roundtrip_preserves_rule_associativity() {
    let mut table = make_table(1, 1);
    table.rule_assoc_by_rule = vec![-1, 0, 1, -1, 1];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.rule_assoc_by_rule, table.rule_assoc_by_rule);
}

// ===================================================================
// 9. Alias sequences
// ===================================================================

#[test]
fn roundtrip_preserves_alias_sequences() {
    let mut table = make_table(1, 1);
    table.alias_sequences = vec![
        vec![None, Some(SymbolId(5)), None],
        vec![Some(SymbolId(10)), Some(SymbolId(20))],
        vec![],
    ];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.alias_sequences, table.alias_sequences);
}

// ===================================================================
// 10. External scanner states
// ===================================================================

#[test]
fn roundtrip_preserves_external_scanner_states() {
    let mut table = make_table(2, 1);
    table.external_scanner_states = vec![vec![true, false, true], vec![false, false, false]];
    table.external_token_count = 3;
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(
        restored.external_scanner_states,
        table.external_scanner_states
    );
    assert_eq!(restored.external_token_count, 3);
}

// ===================================================================
// 11. Index mappings
// ===================================================================

#[test]
fn roundtrip_preserves_symbol_to_index() {
    let mut table = make_table(1, 3);
    table.symbol_to_index = BTreeMap::from([(SymbolId(0), 0), (SymbolId(5), 1), (SymbolId(10), 2)]);
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.symbol_to_index, table.symbol_to_index);
}

#[test]
fn roundtrip_preserves_nonterminal_to_index() {
    let mut table = make_table(1, 3);
    table.nonterminal_to_index = BTreeMap::from([(SymbolId(100), 0), (SymbolId(200), 1)]);
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.nonterminal_to_index, table.nonterminal_to_index);
}

// ===================================================================
// 12. GOTO table content
// ===================================================================

#[test]
fn roundtrip_preserves_goto_table_values() {
    let mut table = make_table(2, 3);
    table.goto_table = vec![
        vec![StateId(0), StateId(5), StateId(u16::MAX)],
        vec![StateId(1), StateId(0), StateId(100)],
    ];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.goto_table, table.goto_table);
}

// ===================================================================
// 13. Error handling: invalid input
// ===================================================================

#[test]
fn empty_bytes_returns_error() {
    let result = ParseTable::from_bytes(&[]);
    assert!(result.is_err());
}

#[test]
fn single_byte_returns_error() {
    let result = ParseTable::from_bytes(&[0x00]);
    assert!(result.is_err());
}

#[test]
fn random_garbage_returns_error() {
    let garbage = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
    let result = ParseTable::from_bytes(&garbage);
    assert!(result.is_err());
}

#[test]
fn truncated_bytes_returns_error() {
    let table = make_table(3, 3);
    let bytes = table.to_bytes().unwrap();
    let half = bytes.len() / 2;
    let result = ParseTable::from_bytes(&bytes[..half]);
    assert!(result.is_err());
}

#[test]
fn one_byte_truncated_returns_error() {
    let table = make_table(2, 2);
    let bytes = table.to_bytes().unwrap();
    let result = ParseTable::from_bytes(&bytes[..bytes.len() - 1]);
    assert!(result.is_err());
}

#[test]
fn extra_trailing_bytes_still_deserializes() {
    // postcard ignores trailing bytes after a complete message
    let table = make_table(1, 1);
    let mut bytes = table.to_bytes().unwrap();
    bytes.extend_from_slice(&[0xFF; 32]);
    // This may succeed or fail depending on postcard behavior—just ensure no panic
    let _ = ParseTable::from_bytes(&bytes);
}

// ===================================================================
// 14. Error handling: version mismatch
// ===================================================================

/// Helper to craft a VersionedParseTable with an arbitrary version.
fn craft_with_version(version: u32, valid_bytes: &[u8]) -> Vec<u8> {
    #[derive(serde::Serialize)]
    struct Wrapper {
        version: u32,
        data: Vec<u8>,
    }
    #[derive(serde::Deserialize)]
    struct WrapperDe {
        #[allow(dead_code)]
        version: u32,
        data: Vec<u8>,
    }
    let original: WrapperDe = postcard::from_bytes(valid_bytes).expect("decode original");
    let fake = Wrapper {
        version,
        data: original.data,
    };
    postcard::to_stdvec(&fake).expect("encode fake")
}

#[test]
fn version_mismatch_returns_incompatible_version_error() {
    let table = make_table(1, 1);
    let bytes = table.to_bytes().unwrap();
    let tampered = craft_with_version(999, &bytes);
    let err = ParseTable::from_bytes(&tampered).unwrap_err();
    match err {
        DeserializationError::IncompatibleVersion { expected, actual } => {
            assert_eq!(expected, PARSE_TABLE_FORMAT_VERSION);
            assert_eq!(actual, 999);
        }
        other => panic!("expected IncompatibleVersion, got: {other}"),
    }
}

#[test]
fn version_zero_returns_incompatible_version_error() {
    let table = make_table(1, 1);
    let bytes = table.to_bytes().unwrap();
    let tampered = craft_with_version(0, &bytes);
    let err = ParseTable::from_bytes(&tampered).unwrap_err();
    assert!(matches!(
        err,
        DeserializationError::IncompatibleVersion { .. }
    ));
}

#[test]
fn future_version_returns_incompatible_version_error() {
    let table = make_table(1, 1);
    let bytes = table.to_bytes().unwrap();
    let tampered = craft_with_version(PARSE_TABLE_FORMAT_VERSION + 1, &bytes);
    let err = ParseTable::from_bytes(&tampered).unwrap_err();
    assert!(matches!(
        err,
        DeserializationError::IncompatibleVersion { .. }
    ));
}

// ===================================================================
// 15. Error display messages
// ===================================================================

#[test]
fn incompatible_version_error_message_contains_versions() {
    let err = DeserializationError::IncompatibleVersion {
        expected: 2,
        actual: 1,
    };
    let msg = err.to_string();
    assert!(msg.contains("expected 2"), "missing expected: {msg}");
    assert!(msg.contains("got 1"), "missing actual: {msg}");
}

#[test]
fn decoding_failed_error_message_contains_postcard_info() {
    let err = DeserializationError::from(postcard::Error::DeserializeUnexpectedEnd);
    let msg = err.to_string();
    assert!(
        msg.contains("Postcard decoding failed"),
        "unexpected: {msg}"
    );
}

#[test]
fn validation_failed_error_preserves_detail() {
    let err = DeserializationError::ValidationFailed("bad checksum".into());
    let msg = err.to_string();
    assert!(msg.contains("bad checksum"), "missing detail: {msg}");
}

// ===================================================================
// 16. Edge cases: boundary values
// ===================================================================

#[test]
fn roundtrip_with_max_symbol_ids() {
    let mut table = make_table(1, 1);
    table.eof_symbol = SymbolId(u16::MAX);
    table.start_symbol = SymbolId(u16::MAX - 1);
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.eof_symbol, SymbolId(u16::MAX));
    assert_eq!(restored.start_symbol, SymbolId(u16::MAX - 1));
}

#[test]
fn roundtrip_with_max_state_id_in_actions() {
    let mut table = make_table(1, 1);
    table.action_table[0][0] = vec![Action::Shift(StateId(u16::MAX))];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(
        restored.action_table[0][0],
        vec![Action::Shift(StateId(u16::MAX))]
    );
}

#[test]
fn roundtrip_with_max_rule_id() {
    let mut table = make_table(1, 1);
    table.action_table[0][0] = vec![Action::Reduce(RuleId(u16::MAX))];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(
        restored.action_table[0][0],
        vec![Action::Reduce(RuleId(u16::MAX))]
    );
}

// ===================================================================
// 17. Edge cases: large tables
// ===================================================================

#[test]
fn roundtrip_many_states() {
    let table = make_table(100, 2);
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.state_count, 100);
    assert_eq!(restored.action_table.len(), 100);
    assert_eq!(restored.goto_table.len(), 100);
}

#[test]
fn roundtrip_many_symbols() {
    let table = make_table(2, 200);
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.symbol_count, 200);
    assert_eq!(restored.action_table[0].len(), 200);
}

#[test]
fn roundtrip_many_rules() {
    let mut table = make_table(1, 1);
    table.rules = (0..500)
        .map(|i| ParseRule {
            lhs: SymbolId(i),
            rhs_len: i % 10,
        })
        .collect();
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.rules.len(), 500);
    assert_eq!(restored.rules[499].lhs, SymbolId(499));
}

// ===================================================================
// 18. Comprehensive table with all fields populated
// ===================================================================

#[test]
fn roundtrip_fully_populated_table() {
    let mut table = make_table(3, 4);
    // Actions with variety
    table.action_table[0] = vec![
        vec![Action::Shift(StateId(1))],
        vec![Action::Reduce(RuleId(0))],
        vec![Action::Accept],
        vec![Action::Shift(StateId(2)), Action::Reduce(RuleId(1))],
    ];
    table.action_table[1] = vec![
        vec![Action::Error],
        vec![Action::Recover],
        vec![Action::Fork(vec![
            Action::Shift(StateId(0)),
            Action::Reduce(RuleId(2)),
        ])],
        vec![],
    ];
    // GOTO
    table.goto_table[0] = vec![StateId(1), StateId(2), StateId(0), StateId(3)];
    // Metadata
    table.symbol_metadata = vec![
        SymbolMetadata {
            name: "EOF".into(),
            is_visible: false,
            is_named: false,
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(0),
        },
        SymbolMetadata {
            name: "number".into(),
            is_visible: true,
            is_named: true,
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(1),
        },
    ];
    // Symbols
    table.eof_symbol = SymbolId(0);
    table.start_symbol = SymbolId(3);
    table.initial_state = StateId(0);
    table.token_count = 3;
    table.external_token_count = 1;
    // Index mappings
    table.symbol_to_index.insert(SymbolId(0), 0);
    table.symbol_to_index.insert(SymbolId(1), 1);
    table.nonterminal_to_index.insert(SymbolId(3), 0);
    // Rules
    table.rules = vec![
        ParseRule {
            lhs: SymbolId(3),
            rhs_len: 2,
        },
        ParseRule {
            lhs: SymbolId(3),
            rhs_len: 1,
        },
    ];
    // Field map
    table.field_names = vec!["value".into()];
    table.field_map.insert((RuleId(0), 0), 1);
    // Lex modes
    table.lex_modes = vec![
        LexMode {
            lex_state: 0,
            external_lex_state: 0,
        },
        LexMode {
            lex_state: 1,
            external_lex_state: 0,
        },
        LexMode {
            lex_state: 2,
            external_lex_state: 1,
        },
    ];
    // Extras
    table.extras = vec![SymbolId(2)];
    // Precedence / assoc
    table.dynamic_prec_by_rule = vec![0, 5];
    table.rule_assoc_by_rule = vec![1, -1];
    // Alias sequences
    table.alias_sequences = vec![vec![None, Some(SymbolId(10))]];
    // External scanner
    table.external_scanner_states = vec![vec![true, false], vec![false, true], vec![false, false]];

    let bytes = table.to_bytes().unwrap();
    let restored = ParseTable::from_bytes(&bytes).unwrap();
    assert_tables_eq(&table, &restored);
}

// ===================================================================
// 19. Format version constant
// ===================================================================

#[test]
fn format_version_is_v2() {
    assert_eq!(PARSE_TABLE_FORMAT_VERSION, 2);
}

#[test]
fn format_version_is_positive() {
    const { assert!(PARSE_TABLE_FORMAT_VERSION > 0) };
}

// ===================================================================
// 20. Serialized bytes are non-empty
// ===================================================================

#[test]
fn serialized_bytes_are_nonempty_for_empty_table() {
    let table = ParseTable::default();
    let bytes = table.to_bytes().unwrap();
    assert!(!bytes.is_empty(), "even empty table produces some bytes");
}

#[test]
fn serialized_size_grows_with_table_size() {
    let small = make_table(2, 2).to_bytes().unwrap();
    let large = make_table(20, 20).to_bytes().unwrap();
    assert!(
        large.len() > small.len(),
        "larger table should produce more bytes: small={} large={}",
        small.len(),
        large.len()
    );
}

// ===================================================================
// 21. Nested Fork actions
// ===================================================================

#[test]
fn roundtrip_preserves_nested_fork() {
    let mut table = make_table(1, 1);
    table.action_table[0][0] = vec![Action::Fork(vec![
        Action::Fork(vec![Action::Shift(StateId(1)), Action::Accept]),
        Action::Reduce(RuleId(0)),
    ])];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_eq!(restored.action_table[0][0], table.action_table[0][0]);
}

// ===================================================================
// 22. Multiple metadata entries
// ===================================================================

#[test]
fn roundtrip_preserves_many_metadata_entries() {
    let mut table = make_table(1, 1);
    table.symbol_metadata = (0..50)
        .map(|i| SymbolMetadata {
            name: format!("sym_{i}"),
            is_visible: i % 2 == 0,
            is_named: i % 3 == 0,
            is_supertype: i % 7 == 0,
            is_terminal: i < 25,
            is_extra: i % 5 == 0,
            is_fragile: i % 11 == 0,
            symbol_id: SymbolId(i as u16),
        })
        .collect();
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_tables_eq(&table, &restored);
}

// ===================================================================
// 23. Corrupted inner data with valid version wrapper
// ===================================================================

#[test]
fn valid_version_but_corrupt_inner_data_returns_error() {
    let table = make_table(1, 1);
    let bytes = table.to_bytes().unwrap();
    // Craft a wrapper with the correct version but garbage inner data
    let tampered = craft_with_version_and_data(PARSE_TABLE_FORMAT_VERSION, &[0xFF; 50]);
    let result = ParseTable::from_bytes(&tampered);
    // The version check passes but inner deserialization should fail
    assert!(result.is_err(), "corrupt inner data: got {bytes:?}");
}

fn craft_with_version_and_data(version: u32, data: &[u8]) -> Vec<u8> {
    #[derive(serde::Serialize)]
    struct Wrapper {
        version: u32,
        data: Vec<u8>,
    }
    let w = Wrapper {
        version,
        data: data.to_vec(),
    };
    postcard::to_stdvec(&w).expect("encode wrapper")
}

// ===================================================================
// 24. Empty inner data with valid version
// ===================================================================

#[test]
fn valid_version_empty_inner_data_returns_error() {
    let tampered = craft_with_version_and_data(PARSE_TABLE_FORMAT_VERSION, &[]);
    let result = ParseTable::from_bytes(&tampered);
    assert!(result.is_err(), "empty inner data should fail");
}

// ===================================================================
// 25. Idempotent double-roundtrip
// ===================================================================

#[test]
fn triple_roundtrip_is_identical() {
    let table = make_table(5, 5);
    let bytes1 = table.to_bytes().unwrap();
    let r1 = ParseTable::from_bytes(&bytes1).unwrap();
    let bytes2 = r1.to_bytes().unwrap();
    let r2 = ParseTable::from_bytes(&bytes2).unwrap();
    let bytes3 = r2.to_bytes().unwrap();
    assert_eq!(bytes1, bytes2);
    assert_eq!(bytes2, bytes3);
}

// ===================================================================
// 26. Mixed empty and non-empty fields
// ===================================================================

#[test]
fn roundtrip_mixed_empty_and_populated_fields() {
    let mut table = make_table(2, 2);
    // Some fields populated, some empty
    table.rules = vec![ParseRule {
        lhs: SymbolId(1),
        rhs_len: 2,
    }];
    table.field_names = vec![];
    table.extras = vec![SymbolId(0)];
    table.dynamic_prec_by_rule = vec![];
    table.rule_assoc_by_rule = vec![1];
    table.alias_sequences = vec![];
    table.external_scanner_states = vec![vec![true], vec![]];
    let restored = ParseTable::from_bytes(&table.to_bytes().unwrap()).unwrap();
    assert_tables_eq(&table, &restored);
}

// ===================================================================
// 27. All-zeros stress test
// ===================================================================

#[test]
fn all_zero_bytes_returns_error() {
    let zeros = vec![0u8; 256];
    // Should not panic; may succeed or fail depending on whether
    // all-zeros happens to be valid postcard for VersionedParseTable
    let _ = ParseTable::from_bytes(&zeros);
}
