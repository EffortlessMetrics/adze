// Test helper functions for creating valid ParseTable instances
use rust_sitter_glr_core::{Action, LexMode, ParseRule, ParseTable, SymbolMetadata};
use rust_sitter_ir::{Grammar, StateId, SymbolId};
use std::collections::BTreeMap;

/// Create a minimal valid ParseTable for testing
pub fn create_minimal_parse_table(grammar: Grammar) -> ParseTable {
    ParseTable {
        action_table: vec![vec![vec![Action::Accept]]],  // Single state with accept action
        goto_table: vec![vec![StateId(0)]],
        symbol_metadata: vec![SymbolMetadata {
            name: "EOF".to_string(),
            visible: true,
            named: true,
            supertype: false,
        }],
        state_count: 1,
        symbol_count: 1,
        symbol_to_index: {
            let mut map = BTreeMap::new();
            map.insert(SymbolId(0), 0);  // EOF
            map
        },
        index_to_symbol: vec![SymbolId(0)],
        external_scanner_states: vec![vec![false]],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(1),
        grammar,
        initial_state: StateId(0),
        token_count: 0,
        external_token_count: 0,
        lex_modes: vec![LexMode {
            lex_state: 0,
            external_lex_state: 0,
        }],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }
}

/// Create a test ParseTable with some actual content
pub fn create_test_parse_table_with_content(grammar: Grammar, state_count: usize, symbol_count: usize) -> ParseTable {
    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = Vec::new();
    
    for i in 0..symbol_count {
        let symbol_id = SymbolId(i as u16);
        symbol_to_index.insert(symbol_id, i);
        index_to_symbol.push(symbol_id);
    }

    ParseTable {
        action_table: vec![vec![vec![Action::Error]]; state_count],
        goto_table: vec![vec![StateId(0); symbol_count]; state_count],
        symbol_metadata: vec![
            SymbolMetadata {
                name: "symbol".to_string(),
                visible: true,
                named: true,
                supertype: false,
            };
            symbol_count
        ],
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        external_scanner_states: vec![vec![false; 10]; state_count],  // Assuming max 10 external tokens
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(1),
        grammar,
        initial_state: StateId(0),
        token_count: symbol_count / 2,  // Rough approximation
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            state_count
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }
}