#![cfg(test)]

mod support {
    // we already use these in golden tests; reuse the same support tree
    pub mod language_builder; // encode_actions
}

use rust_sitter::pure_parser::TSParseAction;
use rust_sitter_glr_core::{Action, ParseRule, ParseTable, SymbolMetadata};
use rust_sitter_ir::{RuleId, StateId, SymbolId};
use support::language_builder::encode_actions;

#[test]
fn encode_actions_minimal() {
    // Build a tiny table: 2 states × 3 symbols
    // (0,0) -> Shift to state 1
    // (0,1) -> Reduce by rule 0 (lhs=S1, rhs_len=2)
    // (1,2) -> Accept
    let grammar = rust_sitter_ir::Grammar::new("toy".into());

    // Symbols: 3 total (S0..S2). We'll mark S1 as the LHS for the rule.
    let mut symbol_to_index = std::collections::BTreeMap::new();
    symbol_to_index.insert(SymbolId(0), 0);
    symbol_to_index.insert(SymbolId(1), 1);
    symbol_to_index.insert(SymbolId(2), 2);

    let table = ParseTable {
        grammar,
        state_count: 2,
        index_to_symbol: vec![SymbolId(0), SymbolId(1), SymbolId(2)],
        symbol_count: 3,
        symbol_to_index,
        symbol_metadata: vec![
            // name/visibility flags are not read by the encoder; just fill enough
            SymbolMetadata {
                name: "s0".into(),
                visible: true,
                named: false,
                supertype: false,
            },
            SymbolMetadata {
                name: "s1".into(),
                visible: true,
                named: false,
                supertype: false,
            },
            SymbolMetadata {
                name: "s2".into(),
                visible: true,
                named: false,
                supertype: false,
            },
        ],
        rules: vec![ParseRule {
            lhs: SymbolId(1),
            rhs_len: 2,
        }],
        action_table: vec![
            vec![
                vec![Action::Shift(StateId(1))],
                vec![Action::Reduce(RuleId(0))],
                vec![],
            ],
            vec![vec![], vec![], vec![Action::Accept]],
        ],
        eof_symbol: SymbolId(2),
        start_symbol: SymbolId(0),
        goto_table: vec![vec![], vec![]],
        external_scanner_states: vec![vec![], vec![]],
        nonterminal_to_index: std::collections::BTreeMap::new(),
        initial_state: StateId(0),
        token_count: 2,
        external_token_count: 0,
        lex_modes: vec![
            rust_sitter_glr_core::LexMode {
                lex_state: 0,
                external_lex_state: 0
            };
            2
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![0],
        alias_sequences: vec![vec![]],
        field_names: vec![],
        field_map: std::collections::BTreeMap::new(),
    };

    // Encode → TS actions + flat table of indices
    let (ts_actions, flat) = encode_actions(&table);

    // 0 = error, others are interned, so just assert the shape & the key cases:
    // (0,0) Shift, (0,1) Reduce(lhs=S1, rhs_len=2), (1,2) Accept
    let idx_00 = flat[0 * table.symbol_count + 0] as usize;
    let idx_01 = flat[0 * table.symbol_count + 1] as usize;
    let idx_12 = flat[1 * table.symbol_count + 2] as usize;

    let a00 = ts_actions[idx_00];
    let a01 = ts_actions[idx_01];
    let a12 = ts_actions[idx_12];

    // action_type: 1=Shift, 2=Reduce, 3=Accept (per your encoder)
    assert_eq!(a00.action_type, 1, "expected Shift at (0,0)");
    assert_eq!(a01.action_type, 2, "expected Reduce at (0,1)");
    assert_eq!(
        a01.child_count, 2,
        "rhs_len should be encoded in child_count"
    );
    assert_eq!(a01.symbol, 1, "reduce lhs must be S1");
    assert_eq!(a12.action_type, 3, "expected Accept at (1,2)");
}
