use adze_glr_core::{Action, LexMode, ParseRule, ParseTable};
use adze_ir::{Grammar, StateId, SymbolId};
use std::collections::BTreeMap;

pub const INVALID: StateId = StateId(u16::MAX);

pub fn make_minimal_table(
    actions: Vec<Vec<Vec<Action>>>,
    gotos: Vec<Vec<StateId>>,
    rules: Vec<ParseRule>,
    start_symbol: SymbolId,
    eof_symbol: SymbolId,
    external_token_count: usize,
) -> ParseTable {
    let state_count = actions.len();
    assert!(state_count > 0, "need at least 1 state");

    let action_cols = actions[0].len();
    let goto_cols = gotos.first().map(|r| r.len()).unwrap_or(0);
    let symbol_count = action_cols.max(goto_cols);
    assert!(symbol_count > 0, "need at least 1 symbol column");

    let eof_idx = eof_symbol.0 as usize;
    assert!(
        eof_idx > 0 && eof_idx <= symbol_count,
        "EOF column must be within symbol table"
    );
    let token_count = eof_idx
        .checked_sub(external_token_count)
        .expect("EOF < externals?");

    let mut symbol_to_index: BTreeMap<SymbolId, usize> = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }

    let mut nonterminal_to_index: BTreeMap<SymbolId, usize> = BTreeMap::new();
    for (col, _) in (0..symbol_count).enumerate() {
        let any_real_goto = gotos
            .iter()
            .any(|row| row.get(col).copied().unwrap_or(INVALID) != INVALID);
        if any_real_goto {
            nonterminal_to_index.insert(SymbolId(col as u16), col);
        }
    }
    nonterminal_to_index
        .entry(start_symbol)
        .or_insert_with(|| start_symbol.0 as usize);

    let mut actions = actions;
    for row in &mut actions {
        if row.len() < symbol_count {
            row.resize_with(symbol_count, Vec::new);
        }
    }

    let mut gotos = gotos;
    if gotos.len() < state_count {
        gotos.resize_with(state_count, || vec![INVALID; symbol_count]);
    }
    for row in &mut gotos {
        if row.len() < symbol_count {
            row.resize(symbol_count, INVALID);
        }
    }

    let lex_modes = vec![
        LexMode {
            lex_state: 0,
            external_lex_state: 0,
        };
        state_count
    ];

    let mut index_to_symbol = vec![SymbolId(u16::MAX); symbol_to_index.len()];
    for (sym, &idx) in &symbol_to_index {
        index_to_symbol[idx] = *sym;
    }

    ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules,
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        goto_indexing: adze_glr_core::GotoIndexing::NonterminalMap,
        symbol_metadata: vec![],
        token_count,
        external_token_count,
        eof_symbol,
        start_symbol,
        initial_state: StateId(0),
        lex_modes,
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        grammar: Grammar::default(),
    }
}
