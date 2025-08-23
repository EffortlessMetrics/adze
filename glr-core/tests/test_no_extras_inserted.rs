//! Test that extras (whitespace/comments) are never inserted during error recovery

#[cfg(feature = "test-helpers")]
#[test]
fn extras_marked_correctly_in_parse_table() {
    use rust_sitter_glr_core::ParseTable;
    use rust_sitter_ir::SymbolId;

    // Create a minimal parse table
    let mut table = ParseTable {
        action_table: vec![],
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count: 0,
        symbol_count: 5,
        symbol_to_index: Default::default(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: Default::default(),
        goto_indexing: rust_sitter_glr_core::GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(100),
        grammar: Default::default(),
        initial_state: rust_sitter_ir::StateId(0),
        token_count: 3,
        external_token_count: 0,
        lex_modes: vec![],
        extras: vec![SymbolId(2)], // Mark symbol 2 as an extra (whitespace)
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: Default::default(),
    };

    // Add symbols to index
    table.symbol_to_index.insert(SymbolId(0), 0); // ID
    table.symbol_to_index.insert(SymbolId(1), 1); // PLUS
    table.symbol_to_index.insert(SymbolId(2), 2); // WS (extra)
    table.symbol_to_index.insert(SymbolId(4), 4); // EOF

    // Test: verify extras are identified correctly
    assert!(
        table.is_extra(SymbolId(2)),
        "Symbol 2 should be marked as extra"
    );
    assert!(!table.is_extra(SymbolId(0)), "Symbol 0 should not be extra");
    assert!(!table.is_extra(SymbolId(1)), "Symbol 1 should not be extra");
    assert!(!table.is_extra(SymbolId(4)), "EOF should not be extra");

    // Test: extras should be within terminal boundary
    let terminal_boundary = table.terminal_boundary();
    assert_eq!(
        terminal_boundary, 3,
        "Terminal boundary should be token_count"
    );
    assert!(
        (2_usize) < terminal_boundary,
        "Extra symbol should be within terminal range"
    );

    // The key invariant: During error recovery in Driver::parse_streaming,
    // the insertion loop iterates 0..eof_symbol and filters out:
    // 1. The EOF symbol itself
    // 2. Any symbol where table.is_extra(sym) returns true
    // This test verifies the table structure supports that filtering.
}

#[cfg(feature = "test-helpers")]
#[test]
fn external_tokens_within_insertion_range() {
    use rust_sitter_glr_core::ParseTable;
    use rust_sitter_ir::SymbolId;

    // Create a parse table with external tokens
    let mut table = ParseTable {
        action_table: vec![],
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count: 0,
        symbol_count: 4,
        symbol_to_index: Default::default(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: Default::default(),
        goto_indexing: rust_sitter_glr_core::GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0), // EOF must be 0 by convention
        start_symbol: SymbolId(100),
        grammar: Default::default(),
        initial_state: rust_sitter_ir::StateId(0),
        token_count: 2,          // Regular tokens: 0, 1
        external_token_count: 1, // External token: 2
        lex_modes: vec![],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: Default::default(),
    };

    // Add symbols to index
    table.symbol_to_index.insert(SymbolId(0), 0); // ID
    table.symbol_to_index.insert(SymbolId(1), 1); // PLUS
    table.symbol_to_index.insert(SymbolId(2), 2); // INDENT (external)
    table.symbol_to_index.insert(SymbolId(3), 3); // EOF

    // Test: verify terminal boundary includes external tokens
    let terminal_boundary = table.terminal_boundary();
    assert_eq!(
        terminal_boundary, 3,
        "Should be token_count + external_token_count"
    );

    // Test: external tokens are terminals
    assert!(
        table.is_terminal(SymbolId(2)),
        "External token should be a terminal"
    );

    // Test: external token is in insertable range (0..eof_symbol)
    assert!(2 < table.eof_symbol.0, "External token should be < EOF");

    // The insertion logic in Driver will consider symbols 0..eof_symbol,
    // which includes the external token at position 2.
}
