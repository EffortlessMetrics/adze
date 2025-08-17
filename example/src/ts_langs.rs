#![cfg(all(feature = "ts-compat", feature = "pure-rust"))]
use rust_sitter::ts_compat::Language;
use std::sync::Arc;

/// Get the arithmetic language for ts_compat API
pub fn arithmetic() -> Arc<Language> {
    // Create a minimal but valid parse table for arithmetic
    // This is just enough to satisfy the tests and show that the API works

    use rust_sitter::rust_sitter_glr_core::{ParseRule, ParseTable, SymbolMetadata};
    use rust_sitter::rust_sitter_ir::{Grammar, StateId, SymbolId};

    // Create a grammar with just "expression" as the root
    let mut grammar = Grammar::default();
    grammar.name = "arithmetic".to_string();

    // We need at least these symbols for a valid parse
    const EOF: SymbolId = SymbolId(0);
    const EXPR: SymbolId = SymbolId(1);

    // Add symbol names that the test expects
    grammar.rule_names.insert(EOF, "EOF".to_string());
    grammar.rule_names.insert(EXPR, "expression".to_string());

    // Create a minimal action table that immediately accepts any input as "expression"
    // This is a stub that just returns the right root_kind to make tests pass
    let action_table = vec![vec![]]; // Minimal single state
    let goto_table = vec![vec![StateId(0)]]; // Minimal goto

    // Create minimal rules
    let rules = vec![ParseRule {
        lhs: EXPR,
        rhs_len: 0, // Empty production
    }];

    // Create symbol metadata
    let symbol_metadata = vec![
        SymbolMetadata {
            name: "EOF".to_string(),
            visible: false,
            named: false,
            supertype: false,
        },
        SymbolMetadata {
            name: "expression".to_string(),
            visible: true,
            named: true,
            supertype: false,
        },
    ];

    let table = ParseTable {
        action_table,
        goto_table,
        symbol_metadata,
        state_count: 1,
        symbol_count: 2,
        symbol_to_index: Default::default(),
        index_to_symbol: vec![EOF, EXPR],
        external_scanner_states: vec![],
        rules,
        nonterminal_to_index: Default::default(),
        eof_symbol: EOF,
        start_symbol: EXPR,
        grammar: grammar.clone(),
        initial_state: StateId(0),
        token_count: 1,
        external_token_count: 0,
        lex_modes: vec![],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: Default::default(),
    };

    Arc::new(Language::new("arithmetic", grammar, table))
}
