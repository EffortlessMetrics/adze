//! This test fixture intentionally omits multiple required fields to ensure compile-time errors
//! This helps catch regressions where ParseTable initializers forget required fields

use rust_sitter_glr_core::{ParseTable, Action, StateId, SymbolId};
use std::collections::BTreeMap;

fn main() {
    // This should fail to compile because many required fields are missing
    let _parse_table = ParseTable {
        action_table: vec![vec![vec![Action::Accept]; 2]; 2],
        goto_table: vec![vec![StateId(0); 2]; 2],
        state_count: 2,
        symbol_count: 2,
        symbol_to_index: BTreeMap::new(),
        // MISSING: index_to_symbol, symbol_metadata, external_scanner_states, 
        // rules, nonterminal_to_index, eof_symbol, start_symbol, grammar,
        // initial_state, token_count, external_token_count, lex_modes,
        // extras, dynamic_prec_by_rule, alias_sequences, field_names, field_map
    };
}