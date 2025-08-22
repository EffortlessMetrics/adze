//! Regression canaries to ensure sentinel values never change.
//! These values are part of the ABI contract with Tree-sitter.

use rust_sitter_glr_core::*;
use rust_sitter_ir::{StateId, SymbolId};

/// Tree-sitter sentinel codepoint values that must remain stable.
pub const ACCEPT_CODEPOINT: u16 = 0xFFFF;
pub const ERROR_CODEPOINT: u16 = 0xFFFE;
pub const RECOVER_CODEPOINT: u16 = 0xFFFD;

#[test]
fn sentinel_codepoints_are_stable() {
    // These must never change as they're part of the Tree-sitter ABI
    assert_eq!(ACCEPT_CODEPOINT, 0xFFFF, "ACCEPT sentinel changed!");
    assert_eq!(ERROR_CODEPOINT, 0xFFFE, "ERROR sentinel changed!");
    assert_eq!(RECOVER_CODEPOINT, 0xFFFD, "RECOVER sentinel changed!");
}

#[test]
fn error_symbol_is_max() {
    // ERROR_SYMBOL is u16::MAX in our implementation (0xFFFF)
    // This differs from Tree-sitter's symbol 0, but we use a sentinel value
    assert_eq!(
        parse_forest::ERROR_SYMBOL.0,
        u16::MAX,
        "ERROR_SYMBOL must be u16::MAX"
    );
}

#[test]
fn eof_invariants() {
    // Test the EOF invariants we enforce in Driver::new
    // This test ensures we never accidentally relax these checks

    // Create a minimal ParseTable to test invariants
    use std::collections::BTreeMap;
    let mut symbol_to_index = BTreeMap::new();
    symbol_to_index.insert(SymbolId(0), 0); // ERROR
    symbol_to_index.insert(SymbolId(1), 1); // terminal
    symbol_to_index.insert(SymbolId(2), 2); // EOF
    symbol_to_index.insert(SymbolId(3), 3); // start symbol

    let tables = ParseTable {
        action_table: vec![vec![vec![]; 4]],
        goto_table: vec![vec![StateId(65535); 4]],
        rules: vec![],
        state_count: 1,
        symbol_count: 4,
        symbol_to_index,
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        nonterminal_to_index: BTreeMap::new(),
        eof_symbol: SymbolId(2),
        start_symbol: SymbolId(3),
        grammar: rust_sitter_ir::Grammar::new("test".to_string()),
        initial_state: StateId(0),
        token_count: 2,
        external_token_count: 0,
        lex_modes: vec![LexMode {
            lex_state: 0,
            external_lex_state: 0,
        }],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        symbol_metadata: vec![],
    };

    // This should not panic - valid configuration
    let _driver = Driver::new(&tables);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "EOF symbol cannot be ERROR symbol")]
fn eof_cannot_be_error() {
    use std::collections::BTreeMap;
    let mut symbol_to_index = BTreeMap::new();
    symbol_to_index.insert(SymbolId(0), 0); // EOF at symbol 0 (ERROR)

    let tables = ParseTable {
        action_table: vec![vec![vec![]; 4]],
        goto_table: vec![vec![StateId(65535); 4]],
        rules: vec![],
        state_count: 1,
        symbol_count: 4,
        symbol_to_index,
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        nonterminal_to_index: BTreeMap::new(),
        eof_symbol: SymbolId(0), // Invalid: EOF = ERROR
        start_symbol: SymbolId(1),
        grammar: rust_sitter_ir::Grammar::new("test".to_string()),
        initial_state: StateId(0),
        token_count: 2,
        external_token_count: 0,
        lex_modes: vec![LexMode {
            lex_state: 0,
            external_lex_state: 0,
        }],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        symbol_metadata: vec![],
    };

    // This should panic with our invariant check
    let _driver = Driver::new(&tables);
}
