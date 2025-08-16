use rust_sitter_glr_core::{ParseRule, ParseTable};
use rust_sitter_ir::{StateId, SymbolId};

mod common;

/// Assert that a ParseTable satisfies all structural invariants
fn assert_parse_table_invariants(t: &ParseTable) {
    // action/goto height == state_count
    assert_eq!(
        t.action_table.len(),
        t.state_count,
        "action_table height must equal state_count"
    );
    assert_eq!(
        t.goto_table.len(),
        t.state_count,
        "goto_table height must equal state_count"
    );

    // EOF column index is token_count + external_token_count
    let eof_col = (t.token_count + t.external_token_count) as usize;
    assert_eq!(
        t.symbol_to_index.get(&t.eof_symbol),
        Some(&eof_col),
        "EOF symbol must be at column token_count + external_token_count"
    );

    // EOF, start present in maps
    assert!(
        t.symbol_to_index.contains_key(&t.eof_symbol),
        "EOF must be in symbol_to_index"
    );
    assert!(
        t.nonterminal_to_index.contains_key(&t.start_symbol),
        "start symbol must be in nonterminal_to_index"
    );

    // initial_state exists
    assert!(
        (t.initial_state.0 as usize) < t.state_count,
        "initial_state must be within valid state range"
    );

    // lex_modes has one entry per state
    assert_eq!(
        t.lex_modes.len(),
        t.state_count,
        "lex_modes must have one entry per state"
    );

    // All rules have valid lhs symbols
    for (i, rule) in t.rules.iter().enumerate() {
        assert!(
            t.nonterminal_to_index.contains_key(&rule.lhs),
            "Rule {} lhs {:?} must be a valid nonterminal",
            i,
            rule.lhs
        );
    }
}

#[test]
fn parse_table_invariants_minimal_table() {
    // Create a minimal table with 2 states, 2 terminals, 1 nonterminal
    let start_symbol = SymbolId(2);
    let eof_symbol = SymbolId(1);

    let actions = vec![
        vec![vec![], vec![]], // State 0 actions
        vec![vec![], vec![]], // State 1 actions
    ];

    let gotos = vec![
        vec![StateId(1)], // State 0 gotos
        vec![],           // State 1 gotos
    ];

    let rules = vec![ParseRule {
        lhs: start_symbol,
        rhs_len: 0,
    }];

    let table = common::make_minimal_table(
        actions,
        gotos,
        rules,
        start_symbol,
        eof_symbol,
        0, // no external tokens
    );
    assert_parse_table_invariants(&table);
}

#[test]
fn parse_table_invariants_empty_table() {
    // Create a truly minimal empty table
    // Need at least ERROR (0) and EOF (1) columns
    let start_symbol = SymbolId(2);
    let eof_symbol = SymbolId(1);

    let actions = vec![
        vec![vec![], vec![]], // State 0 with ERROR and EOF columns
    ];

    let gotos = vec![
        vec![], // State 0 has no gotos
    ];

    let rules = vec![];

    let table = common::make_minimal_table(
        actions,
        gotos,
        rules,
        start_symbol,
        eof_symbol,
        0, // no external tokens
    );
    assert_parse_table_invariants(&table);
}

#[test]
fn parse_table_invariants_with_externals() {
    // Test with external tokens - create a fresh table with externals
    let start_symbol = SymbolId(3);
    let token_count = 2;
    let external_token_count = 2;
    let eof_symbol = SymbolId((token_count + external_token_count) as u16);

    let actions = vec![
        vec![vec![], vec![], vec![], vec![], vec![]], // State 0 actions (5 columns)
        vec![vec![], vec![], vec![], vec![], vec![]], // State 1 actions
    ];

    let gotos = vec![
        vec![StateId(1)], // State 0 gotos
        vec![],           // State 1 gotos
    ];

    let rules = vec![ParseRule {
        lhs: start_symbol,
        rhs_len: 0,
    }];

    let table = common::make_minimal_table(
        actions,
        gotos,
        rules,
        start_symbol,
        eof_symbol,
        external_token_count,
    );

    assert_parse_table_invariants(&table);
}

#[test]
fn parse_table_invariants_custom_states() {
    // Test with more states
    let start_symbol = SymbolId(2);
    let eof_symbol = SymbolId(1);

    // Create table with 10 states
    let mut actions = Vec::new();
    let mut gotos = Vec::new();
    for i in 0..10 {
        actions.push(vec![vec![], vec![]]); // 2 columns for each state
        if i == 0 {
            gotos.push(vec![StateId(1)]);
        } else {
            gotos.push(vec![]);
        }
    }

    let rules = vec![ParseRule {
        lhs: start_symbol,
        rhs_len: 0,
    }];

    let table = common::make_minimal_table(actions, gotos, rules, start_symbol, eof_symbol, 0);

    assert_parse_table_invariants(&table);
}
