//! Tests for the ABI builder module — symbol metadata, state counts.

use adze_glr_core::*;
use adze_ir::SymbolId;

fn build_simple_parse_table() -> ParseTable {
    let mut pt = ParseTable {
        state_count: 4,
        eof_symbol: SymbolId(0),
        ..ParseTable::default()
    };
    pt.symbol_metadata = vec![
        SymbolMetadata {
            name: "end".to_string(),
            is_visible: false,
            is_named: false,
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(0),
        },
        SymbolMetadata {
            name: "num".to_string(),
            is_visible: true,
            is_named: true,
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(1),
        },
        SymbolMetadata {
            name: "plus".to_string(),
            is_visible: true,
            is_named: false,
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(2),
        },
        SymbolMetadata {
            name: "expr".to_string(),
            is_visible: true,
            is_named: true,
            is_supertype: false,
            is_terminal: false,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(3),
        },
    ];
    pt
}

#[test]
fn symbol_metadata_visible_named() {
    let meta = SymbolMetadata {
        name: "identifier".to_string(),
        is_visible: true,
        is_named: true,
        is_supertype: false,
        is_terminal: true,
        is_extra: false,
        is_fragile: false,
        symbol_id: SymbolId(5),
    };
    assert!(meta.is_visible);
    assert!(meta.is_named);
    assert!(!meta.is_supertype);
}

#[test]
fn symbol_metadata_extra() {
    let meta = SymbolMetadata {
        name: "whitespace".to_string(),
        is_visible: false,
        is_named: false,
        is_supertype: false,
        is_terminal: true,
        is_extra: true,
        is_fragile: false,
        symbol_id: SymbolId(10),
    };
    assert!(meta.is_extra);
    assert!(!meta.is_visible);
}

#[test]
fn symbol_metadata_fragile() {
    let meta = SymbolMetadata {
        name: "keyword_if".to_string(),
        is_visible: true,
        is_named: true,
        is_supertype: false,
        is_terminal: true,
        is_extra: false,
        is_fragile: true,
        symbol_id: SymbolId(20),
    };
    assert!(meta.is_fragile);
}

#[test]
fn parse_table_symbol_count() {
    let pt = build_simple_parse_table();
    assert_eq!(pt.symbol_metadata.len(), 4);
}

#[test]
fn parse_table_state_count() {
    let pt = build_simple_parse_table();
    assert_eq!(pt.state_count, 4);
}

#[test]
fn parse_table_eof_symbol() {
    let pt = build_simple_parse_table();
    assert_eq!(pt.eof_symbol, SymbolId(0));
}

#[test]
fn symbol_metadata_debug() {
    let meta = SymbolMetadata {
        name: "test".to_string(),
        is_visible: true,
        is_named: true,
        is_supertype: false,
        is_terminal: true,
        is_extra: false,
        is_fragile: false,
        symbol_id: SymbolId(1),
    };
    let debug = format!("{meta:?}");
    assert!(debug.contains("test"));
}

#[test]
fn parse_rule_fields() {
    let rule = ParseRule {
        lhs: SymbolId(3),
        rhs_len: 2,
    };
    assert_eq!(rule.lhs, SymbolId(3));
    assert_eq!(rule.rhs_len, 2);
}

#[test]
fn action_variants_are_distinct() {
    let shift = Action::Shift(StateId(1));
    let reduce = Action::Reduce(RuleId(1));
    let accept = Action::Accept;
    assert_ne!(format!("{shift:?}"), format!("{reduce:?}"));
    assert_ne!(format!("{reduce:?}"), format!("{accept:?}"));
    assert_ne!(format!("{shift:?}"), format!("{accept:?}"));
}
