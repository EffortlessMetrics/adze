//! Tests for the table compression algorithms.

use adze_tablegen::compress::*;

#[test]
fn table_compressor_creation() {
    let _compressor = TableCompressor::new();
}

#[test]
fn compressed_action_entry_new_shift() {
    let entry =
        CompressedActionEntry::new(5, adze_glr_core::Action::Shift(adze_glr_core::StateId(10)));
    assert_eq!(entry.symbol, 5);
}

#[test]
fn compressed_action_entry_new_reduce() {
    let entry =
        CompressedActionEntry::new(3, adze_glr_core::Action::Reduce(adze_glr_core::RuleId(2)));
    assert_eq!(entry.symbol, 3);
}

#[test]
fn compressed_action_entry_new_accept() {
    let entry = CompressedActionEntry::new(0, adze_glr_core::Action::Accept);
    assert_eq!(entry.symbol, 0);
}

#[test]
fn encode_action_small_shift() {
    let compressor = TableCompressor::new();
    let result =
        compressor.encode_action_small(&adze_glr_core::Action::Shift(adze_glr_core::StateId(5)));
    assert!(result.is_ok());
    let encoded = result.unwrap();
    assert!(encoded > 0);
}

#[test]
fn encode_action_small_reduce() {
    let compressor = TableCompressor::new();
    let result =
        compressor.encode_action_small(&adze_glr_core::Action::Reduce(adze_glr_core::RuleId(3)));
    assert!(result.is_ok());
}

#[test]
fn encode_action_small_accept() {
    let compressor = TableCompressor::new();
    let result = compressor.encode_action_small(&adze_glr_core::Action::Accept);
    assert!(result.is_ok());
}

#[test]
fn compressed_tables_validate_empty_rejects() {
    let pt = adze_glr_core::ParseTable::default();
    let tables = CompressedTables {
        action_table: CompressedActionTable {
            data: vec![],
            default_actions: vec![],
            row_offsets: vec![],
        },
        goto_table: CompressedGotoTable {
            data: vec![],
            row_offsets: vec![],
        },
        small_table_threshold: 0,
    };
    let result = tables.validate(&pt);
    assert!(result.is_err());
}

#[test]
fn compressed_parse_table_from_empty() {
    let pt = adze_glr_core::ParseTable::default();
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.state_count(), 0);
    assert_eq!(cpt.symbol_count(), 0);
}
