//! This test fixture intentionally omits the index_to_symbol field to ensure compile-time errors
//! This helps catch regressions where ParseTable initializers forget required fields

use rust_sitter_glr_core::{
    Action, Grammar, LexMode, ParseTable, StateId, SymbolId, SymbolMetadata,
};
use std::collections::BTreeMap;

fn main() {
    let mut symbol_to_index = BTreeMap::new();
    symbol_to_index.insert(SymbolId(0), 0);
    symbol_to_index.insert(SymbolId(1), 1);

    let grammar = Grammar::default();

    // This should fail to compile because index_to_symbol is missing
    let _parse_table = ParseTable {
        action_table: vec![vec![vec![Action::Accept]; 2]; 2],
        goto_table: vec![vec![StateId(0); 2]; 2],
        state_count: 2,
        symbol_count: 2,
        symbol_to_index,
        // MISSING: index_to_symbol
        symbol_metadata: vec![
            SymbolMetadata {
                name: "token".to_string(),
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(0),
            },
            SymbolMetadata {
                name: "S".to_string(),
                is_terminal: false,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(1),
            },
        ],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(1),
        grammar,
        initial_state: StateId(0),
        token_count: 1,
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0
            };
            2
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    };
}
