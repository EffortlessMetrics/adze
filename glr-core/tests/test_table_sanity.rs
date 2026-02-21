use adze_glr_core::{Action, LexMode, ParseTable, sanity_check_tables};
use adze_ir::{Grammar, StateId, SymbolId};
use std::collections::BTreeMap;

#[test]
fn parse_table_is_sane() {
    // Create a simple test parse table
    let pt = create_simple_test_table();
    sanity_check_tables(&pt).expect("Parse table failed sanity check");
}

fn create_simple_test_table() -> ParseTable {
    // Create a minimal valid parse table
    // S -> a
    // State 0: a -> shift 1, EOF -> error
    // State 1: EOF -> accept

    let action_table = vec![
        vec![vec![Action::Shift(StateId(1))], vec![Action::Error]], // State 0
        vec![vec![Action::Error], vec![Action::Accept]],            // State 1
    ];

    let goto_table = vec![
        vec![StateId(0), StateId(0)], // State 0
        vec![StateId(0), StateId(0)], // State 1
    ];

    let mut symbol_to_index = BTreeMap::new();
    symbol_to_index.insert(SymbolId(1), 0); // token 'a'
    symbol_to_index.insert(SymbolId(0), 1); // EOF (normalized to 0)

    let index_to_symbol = vec![SymbolId(1), SymbolId(0)];

    ParseTable {
        action_table,
        goto_table,
        symbol_metadata: vec![],
        state_count: 2,
        symbol_count: 2,
        symbol_to_index,
        index_to_symbol,
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: adze_glr_core::GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(2),
        grammar: Grammar::default(),
        initial_state: StateId(0),
        token_count: 2,
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
    }
}

#[test]
fn test_table_has_accept_on_eof() {
    let pt = create_simple_test_table();

    // Find EOF column
    let eof_col = pt
        .symbol_to_index
        .get(&pt.eof_symbol)
        .expect("EOF symbol not in symbol_to_index");

    // Check that at least one state has ACCEPT on EOF
    let has_accept = pt.action_table.iter().any(|row| {
        row.get(*eof_col)
            .and_then(|cell| cell.iter().find(|a| matches!(a, Action::Accept)))
            .is_some()
    });

    assert!(has_accept, "No ACCEPT action found on EOF in any state");
}
