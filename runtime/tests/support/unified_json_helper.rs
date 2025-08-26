mod language_builder;

use rust_sitter::pure_parser::TSLanguage;
use rust_sitter_glr_core::{Action, ParseRule, ParseTable, SymbolMetadata};
use rust_sitter_glr_core::GotoIndexing;
use rust_sitter_ir::{Grammar, StateId, SymbolId, Token, TokenPattern};
use ts_bridge::{extract, schema::{Action as TsAction}};
use std::ffi::c_void;

#[repr(C)]
struct UpstreamLanguage {
    _prefix: [u8; 0],
    // fields up to lex_fn
    version: u32,
    symbol_count: u32,
    alias_count: u32,
    token_count: u32,
    external_token_count: u32,
    state_count: u32,
    large_state_count: u32,
    production_id_count: u32,
    field_count: u32,
    max_alias_sequence_length: u16,
    parse_table: *const u16,
    small_parse_table: *const u16,
    small_parse_table_map: *const u32,
    parse_actions: *const u16,
    symbol_names: *const *const u8,
    field_names: *const *const u8,
    field_map_slices: *const u16,
    field_map_entries: *const u16,
    symbol_metadata: *const u8,
    public_symbol_map: *const u16,
    alias_map: *const u16,
    alias_sequences: *const u16,
    lex_modes: *const u8,
    lex_fn: Option<unsafe extern "C" fn(*mut c_void, StateId) -> bool>,
}

/// Return a `TSLanguage` built from the real Tree-sitter JSON grammar.
///
/// Rather than casting the upstream `tree_sitter_json` pointer into our
/// `TSLanguage` (which has a different ABI), we use the `ts-bridge` extractor
/// to decode the Tree-sitter parse tables and rebuild a fresh language using
/// our pure-Rust layout.
pub fn unified_json_language() -> &'static TSLanguage {
    // Extract parse table data from upstream Tree-sitter JSON grammar
    let lang_fn: unsafe extern "C" fn() -> *const ts_bridge::ffi::TSLanguage =
        unsafe { std::mem::transmute(tree_sitter_json::LANGUAGE.into_raw()) };
    let data = extract(lang_fn).expect("extract tree-sitter json");

    // Build minimal Grammar with symbol names and token stubs
    let mut grammar = Grammar::new("ts_json".to_string());
    for (i, sym) in data.symbols.iter().enumerate() {
        let sid = SymbolId(i as u16);
        grammar.rule_names.insert(sid, sym.name.clone());
        if (i as u32) < data.token_count + data.external_token_count {
            grammar.tokens.insert(
                sid,
                Token {
                    name: sym.name.clone(),
                    pattern: TokenPattern::String(sym.name.clone()),
                    fragile: false,
                },
            );
        }
    }

    // Convert extracted data into our ParseTable representation
    let state_count = data.state_count as usize;
    let symbol_count = data.symbol_count as usize;
    let mut table = ParseTable {
        action_table: vec![vec![Vec::new(); symbol_count]; state_count],
        goto_table: vec![vec![StateId(0); symbol_count]; state_count],
        symbol_metadata: Vec::with_capacity(symbol_count),
        state_count,
        symbol_count,
        symbol_to_index: std::collections::BTreeMap::new(),
        index_to_symbol: Vec::with_capacity(symbol_count),
        external_scanner_states: vec![vec![false; data.external_token_count as usize]; state_count],
        rules: data
            .rules
            .iter()
            .map(|r| ParseRule {
                lhs: SymbolId(r.lhs),
                rhs_len: r.rhs_len,
            })
            .collect(),
        nonterminal_to_index: std::collections::BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(data.start_symbol),
        grammar: grammar.clone(),
        initial_state: StateId(0),
        token_count: data.token_count as usize,
        external_token_count: data.external_token_count as usize,
        lex_modes: vec![rust_sitter_glr_core::LexMode {
            lex_state: 0,
            external_lex_state: 0,
        }; state_count],
        extras: Vec::new(),
        dynamic_prec_by_rule: vec![0; data.rules.len()],
        rule_assoc_by_rule: vec![0; data.rules.len()],
        alias_sequences: vec![vec![]; data.rules.len()],
        field_names: Vec::new(),
        field_map: std::collections::BTreeMap::new(),
    };

    for (i, sym) in data.symbols.iter().enumerate() {
        table.symbol_metadata.push(SymbolMetadata {
            name: sym.name.clone(),
            visible: sym.visible,
            named: sym.named,
            supertype: false,
        });
        let sid = SymbolId(i as u16);
        table.symbol_to_index.insert(sid, i);
        table.index_to_symbol.push(sid);
        if (i as u32) >= data.token_count + data.external_token_count {
            table.nonterminal_to_index.insert(sid, i);
        }
    }

    for cell in &data.actions {
        let sym = if cell.symbol == data.eof_symbol { 0 } else { cell.symbol };
        let row = &mut table.action_table[cell.state as usize][sym as usize];
        for a in &cell.actions {
            row.push(match a {
                TsAction::Shift { state, .. } => Action::Shift(StateId(*state)),
                TsAction::Reduce { rule, .. } => Action::Reduce(rust_sitter_ir::RuleId(*rule)),
                TsAction::Accept => Action::Accept,
                TsAction::Recover => Action::Recover,
            });
        }
    }

    for cell in &data.gotos {
        if let Some(next) = cell.next_state {
            table.goto_table[cell.state as usize][cell.symbol as usize] = StateId(next);
        }
    }

    // Normalize for Tree-sitter layout and build final language
    language_builder::normalize_table_for_ts(&mut table);
    let lang = language_builder::build_json_ts_language(&grammar, &table);
    Box::leak(Box::new(lang))
}
