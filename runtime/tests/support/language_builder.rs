#![cfg(feature = "pure-rust")]

use rust_sitter::pure_parser::{ExternalScanner, TSLanguage, TSLexState, TSParseAction};
use rust_sitter_glr_core::{Action, ParseTable};
use rust_sitter_ir::{Grammar, StateId};
use std::collections::HashMap;
use std::ffi::CString;

/// Build a stable action set and return a flat parse_table (indices into `ts_actions`)
fn encode_actions(parse_table: &ParseTable) -> (Vec<TSParseAction>, Vec<u16>) {
    // 0 = Error
    let mut ts_actions: Vec<TSParseAction> = vec![TSParseAction {
        action_type: 0,
        extra: 0,
        child_count: 0,
        dynamic_precedence: 0,
        symbol: 0,
    }];

    // Intern identical actions so we reuse indices
    let mut intern: HashMap<(u8, u8, u8, u16), u16> = HashMap::new();
    intern.insert((0, 0, 0, 0), 0);

    // Helper: intern and return index
    let mut push_action = |a: TSParseAction| -> u16 {
        let key = (a.action_type, a.extra, a.child_count, a.symbol);
        if let Some(&idx) = intern.get(&key) {
            return idx;
        }
        let idx = ts_actions.len() as u16;
        ts_actions.push(a);
        intern.insert(key, idx);
        idx
    };

    // For each state×symbol pick **one** action (simple LR(1) surface)
    let mut flat: Vec<u16> =
        Vec::with_capacity(parse_table.state_count * parse_table.index_to_symbol.len());

    for s in 0..parse_table.state_count {
        for c in 0..parse_table.index_to_symbol.len() {
            let cell = parse_table
                .action_table
                .get(s)
                .and_then(|row| row.get(c))
                .cloned()
                .unwrap_or_default();

            // Pick the first action as the representative (GLR cells may contain multiple)
            let idx = if let Some(a) = cell.first() {
                match *a {
                    Action::Shift(StateId(tgt)) => push_action(TSParseAction {
                        action_type: 1, // Shift
                        extra: 0,
                        child_count: 0,
                        dynamic_precedence: 0,
                        symbol: tgt as u16,
                    }),
                    Action::Reduce(rule_idx) => {
                        let pr = &parse_table.rules[rule_idx.0 as usize];
                        push_action(TSParseAction {
                            action_type: 2, // Reduce
                            extra: 0,
                            child_count: pr.rhs_len as u8, // RHS length
                            dynamic_precedence: 0,
                            symbol: pr.lhs.0 as u16, // LHS symbol
                        })
                    }
                    Action::Accept => push_action(TSParseAction {
                        action_type: 3,
                        extra: 0,
                        child_count: 0,
                        dynamic_precedence: 0,
                        symbol: 0,
                    }),
                    Action::Fork(_) => 0, // treat as error for the TS surface for now
                    _ => 0,               // treat other actions as error
                }
            } else {
                0 // error
            };
            flat.push(idx);
        }
    }

    (ts_actions, flat)
}

/// Build a TSLanguage from grammar and parse table
pub fn build_ts_language(grammar: &Grammar, parse_table: &ParseTable) -> TSLanguage {
    // Build symbol names as C strings (*const u8)
    let mut symbol_names_c: Vec<CString> = Vec::new();
    let mut symbol_names_ptrs: Vec<*const u8> = Vec::new();

    // Add all symbols (tokens and non-terminals)
    for i in 0..parse_table.symbol_count {
        let sym_id = parse_table.index_to_symbol[i];
        let name = grammar
            .rule_names
            .get(&sym_id)
            .cloned()
            .or_else(|| grammar.tokens.get(&sym_id).map(|t| t.name.clone()))
            .unwrap_or_else(|| format!("symbol_{}", sym_id.0));

        let c_string = CString::new(name).unwrap();
        symbol_names_ptrs.push(c_string.as_ptr() as *const u8);
        symbol_names_c.push(c_string);
    }

    // Leak the data to get static references
    let symbol_names_c = Box::leak(Box::new(symbol_names_c));
    let symbol_names_ptrs = Box::leak(Box::new(symbol_names_ptrs));

    // Build field names as C strings (*const u8)
    let mut field_names_c: Vec<CString> = Vec::new();
    let mut field_names_ptrs: Vec<*const u8> = Vec::new();

    // First entry is always empty
    let empty = CString::new("").unwrap();
    field_names_ptrs.push(empty.as_ptr() as *const u8);
    field_names_c.push(empty);

    // Add field names in sorted order
    let mut field_ids: Vec<_> = grammar.fields.keys().cloned().collect();
    field_ids.sort_by_key(|f| f.0);
    for field_id in field_ids {
        let c_string = CString::new(grammar.fields[&field_id].clone()).unwrap();
        field_names_ptrs.push(c_string.as_ptr() as *const u8);
        field_names_c.push(c_string);
    }

    let field_names_c = Box::leak(Box::new(field_names_c));
    let field_names_ptrs = Box::leak(Box::new(field_names_ptrs));

    // Build symbol metadata
    let mut symbol_metadata = Vec::new();
    // Use index_to_symbol.len() instead of symbol_count to match actual array size
    for i in 0..parse_table.index_to_symbol.len() {
        if i < parse_table.symbol_metadata.len() {
            let meta = &parse_table.symbol_metadata[i];
            // Pack into a single byte: bit 0 = visible, bit 1 = named
            let byte = (meta.visible as u8) | ((meta.named as u8) << 1);
            symbol_metadata.push(byte);
        } else {
            // Default metadata for any extra symbols
            symbol_metadata.push(0);
        }
    }
    let symbol_metadata = Box::leak(Box::new(symbol_metadata));

    // Build lex modes
    let mut lex_modes = Vec::new();
    for _ in 0..parse_table.state_count {
        lex_modes.push(TSLexState {
            lex_state: 0,
            external_lex_state: 0,
        });
    }
    let lex_modes = Box::leak(Box::new(lex_modes));

    // Build parse actions & full (uncompressed) parse table with real actions
    let (ts_actions, full_parse_table) = encode_actions(parse_table);
    let parse_actions = Box::leak(Box::new(ts_actions));
    let full_parse_table = Box::leak(Box::new(full_parse_table));

    // Build productions (lhs per rule)
    let mut production_lhs = Vec::new();
    for r in &parse_table.rules {
        production_lhs.push(r.lhs.0 as u16);
    }
    let production_lhs = Box::leak(Box::new(production_lhs));

    // If we have states, they should all be large states for simplicity
    // since we're not implementing compression
    TSLanguage {
        version: 15,
        symbol_count: parse_table.index_to_symbol.len() as u32,
        alias_count: 0,
        token_count: parse_table.token_count as u32,
        external_token_count: parse_table.external_token_count as u32,
        state_count: parse_table.state_count as u32,
        large_state_count: parse_table.state_count as u32, // All states are large states
        production_id_count: 0,
        field_count: grammar.fields.len() as u32,
        max_alias_sequence_length: 0,
        production_id_map: std::ptr::null(),
        parse_table: full_parse_table.as_ptr(),
        small_parse_table: std::ptr::null(),
        small_parse_table_map: std::ptr::null(),
        parse_actions: parse_actions.as_ptr(),
        symbol_names: symbol_names_ptrs.as_ptr() as *const *const u8,
        field_names: field_names_ptrs.as_ptr() as *const *const u8,
        field_map_slices: std::ptr::null(),
        field_map_entries: std::ptr::null(),
        symbol_metadata: symbol_metadata.as_ptr(),
        public_symbol_map: std::ptr::null(),
        alias_map: std::ptr::null(),
        alias_sequences: std::ptr::null(),
        lex_modes: lex_modes.as_ptr(),
        lex_fn: None,
        keyword_lex_fn: None,
        keyword_capture_token: 0,
        external_scanner: ExternalScanner::default(),
        primary_state_ids: std::ptr::null(),
        production_lhs_index: production_lhs.as_ptr(),
        production_count: parse_table.rules.len() as u16,
        eof_symbol: parse_table.eof_symbol.0 as u16,
    }
}
