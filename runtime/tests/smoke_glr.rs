use rust_sitter_glr_core::{Action, LexMode, ParseRule, ParseTable};
use rust_sitter_ir::{Grammar, RuleId, StateId, SymbolId};
use std::collections::BTreeMap;

#[test]
fn glr_smoke_table_construction() {
    // Test that we can construct a basic parse table without panic
    // EOF(0), 'x'(1), S(2) - EOF must be SymbolId(0) by convention
    let mut action = vec![vec![vec![]; 3]; 2];
    action[0][1].push(Action::Shift(StateId(1))); // on 'x' shift to 1
    action[1][0].push(Action::Reduce(RuleId(0))); // on EOF reduce S -> 'x'

    let mut gotos = vec![vec![StateId(65535); 3]; 2];
    gotos[0][2] = StateId(1); // goto S after reduce (accept state)

    // Map terminals to ACTION table columns
    let mut sym2idx = BTreeMap::new();
    for i in 0..3 {
        sym2idx.insert(SymbolId(i), i as usize);
    }

    let table = ParseTable {
        action_table: action,
        goto_table: gotos,
        rules: vec![ParseRule {
            lhs: SymbolId(2),
            rhs_len: 1,
        }],
        state_count: 2,
        symbol_count: 3,
        symbol_to_index: sym2idx,
        index_to_symbol: vec![SymbolId(0), SymbolId(1), SymbolId(2)],
        token_count: 1, // 'x'
        external_token_count: 0,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(2),
        extras: vec![],
        external_scanner_states: vec![vec![false; 0]; 2],
        grammar: Grammar::default(),
        initial_state: StateId(0),
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0
            };
            2
        ],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        nonterminal_to_index: BTreeMap::from([(SymbolId(2), 2)]),
        goto_indexing: rust_sitter_glr_core::GotoIndexing::NonterminalMap,
        symbol_metadata: vec![],
    }
    .normalize_eof_to_zero();

    // Basic sanity checks
    assert_eq!(table.state_count, 2);
    assert_eq!(table.symbol_count, 3);
    assert_eq!(table.token_count, 1);
    assert_eq!(table.eof_symbol, SymbolId(0));
    assert_eq!(table.start_symbol, SymbolId(2));

    // Verify we can create a driver (doesn't parse anything, just checks construction)
    let _driver = rust_sitter_glr_core::Driver::new(&table);
}
