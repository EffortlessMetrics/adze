use rust_sitter_glr_core::{Driver, ParseTable, Action, LexMode};
use rust_sitter_ir::{StateId, SymbolId};
use std::collections::BTreeSet;

/// Create a simple grammar with whitespace extras
/// Grammar: S -> 'a'
/// Extras: WS (whitespace)
/// 
/// Symbol layout:
/// 0: start symbol (non-terminal)
/// 1: 'a' (terminal)
/// 2: WS (terminal, extra)
/// 3: EOF
fn create_grammar_with_extras() -> ParseTable {
    let mut action_table = vec![];
    let mut goto_table = vec![];
    
    // State 0: initial state
    // Can shift 'a' to state 1
    // WS is an extra - should never be inserted during recovery
    action_table.push(vec![
        vec![],  // 0: start symbol
        vec![Action::Shift(StateId(1))],  // 1: 'a'
        vec![],  // 2: WS (extra - no action)
        vec![],  // 3: EOF
    ]);
    goto_table.push(vec![StateId(0); 4]);
    
    // State 1: after 'a'
    // Can accept on EOF
    action_table.push(vec![
        vec![],  // 0: start symbol
        vec![],  // 1: 'a'
        vec![],  // 2: WS (extra)
        vec![Action::Accept],  // 3: EOF
    ]);
    goto_table.push(vec![StateId(0); 4]);
    
    // Create symbol_to_index mapping
    let mut symbol_to_index = std::collections::BTreeMap::new();
    symbol_to_index.insert(SymbolId(0), 0);
    symbol_to_index.insert(SymbolId(1), 1);
    symbol_to_index.insert(SymbolId(2), 2);
    symbol_to_index.insert(SymbolId(3), 3);
    
    // Create extras set - WS is an extra
    let mut extras = BTreeSet::new();
    extras.insert(SymbolId(2));
    
    ParseTable {
        action_table,
        goto_table,
        symbol_metadata: vec![],
        state_count: 2,
        symbol_count: 4,
        symbol_to_index,
        token_count: 3,  // terminals: 'a', WS, EOF
        external_token_count: 0,
        eof_symbol: SymbolId(3),
        extras,
        reduce_actions: vec![],
        combined_tables: None,
    }
}

#[test]
fn test_extras_never_inserted_during_recovery() {
    let table = create_grammar_with_extras();
    let driver = Driver::new(table, LexMode::default());
    
    // Empty input - missing 'a'
    // The driver should insert 'a' (SymbolId(1)), never WS (SymbolId(2))
    let tokens: Vec<(u32, u32, u32)> = vec![];
    
    // Parse with error recovery
    let result = driver.parse_tokens(tokens.into_iter()).unwrap();
    
    // Check error stats using the test helper
    #[cfg(any(test, feature = "test-helpers"))]
    {
        let (has_error, missing, cost) = result.debug_error_stats();
        
        // Should have error due to missing terminal
        assert!(has_error, "Should have error due to missing terminal");
        
        // Should have inserted exactly one missing terminal ('a')
        assert_eq!(missing, 1, "Should insert exactly one missing terminal");
        assert_eq!(cost, 1, "Cost should be 1 for single insertion");
    }
    
    // The important invariant: WS was not inserted because it's an extra
    // The insertion loop in try_insertion() should skip SymbolId(2) entirely
}

#[test]
fn test_insertion_skips_all_extras() {
    // Create a grammar with multiple extras
    let mut action_table = vec![];
    let mut goto_table = vec![];
    
    // State 0: initial state
    // Can shift 'a' to state 1, 'b' to state 2
    action_table.push(vec![
        vec![],  // 0: start
        vec![Action::Shift(StateId(1))],  // 1: 'a'
        vec![Action::Shift(StateId(2))],  // 2: 'b'
        vec![],  // 3: WS (extra)
        vec![],  // 4: comment (extra)
        vec![],  // 5: EOF
    ]);
    goto_table.push(vec![StateId(0); 6]);
    
    // State 1: after 'a'
    // Can shift 'b' to state 2
    action_table.push(vec![
        vec![],  // 0
        vec![],  // 1
        vec![Action::Shift(StateId(2))],  // 2: 'b'
        vec![],  // 3: WS
        vec![],  // 4: comment
        vec![],  // 5: EOF
    ]);
    goto_table.push(vec![StateId(0); 6]);
    
    // State 2: after 'a' 'b' or just 'b'
    // Can accept on EOF
    action_table.push(vec![
        vec![],  // 0
        vec![],  // 1
        vec![],  // 2
        vec![],  // 3
        vec![],  // 4
        vec![Action::Accept],  // 5: EOF
    ]);
    goto_table.push(vec![StateId(0); 6]);
    
    let mut symbol_to_index = std::collections::BTreeMap::new();
    for i in 0..6 {
        symbol_to_index.insert(SymbolId(i), i as usize);
    }
    
    // Mark WS and comment as extras
    let mut extras = BTreeSet::new();
    extras.insert(SymbolId(3));  // WS
    extras.insert(SymbolId(4));  // comment
    
    let table = ParseTable {
        action_table,
        goto_table,
        symbol_metadata: vec![],
        state_count: 3,
        symbol_count: 6,
        symbol_to_index,
        token_count: 5,  // 'a', 'b', WS, comment, EOF
        external_token_count: 0,
        eof_symbol: SymbolId(5),
        extras,
        reduce_actions: vec![],
        combined_tables: None,
    };
    
    let driver = Driver::new(table, LexMode::default());
    
    // Input with only 'b' - missing 'a'
    let tokens = vec![
        (2, 0, 1),  // 'b' at position 0-1
    ];
    
    let result = driver.parse_tokens(tokens.into_iter()).unwrap();
    
    #[cfg(any(test, feature = "test-helpers"))]
    {
        let (has_error, missing, _cost) = result.debug_error_stats();
        
        assert!(has_error, "Should have error");
        // Should insert 'a', never WS or comment
        assert_eq!(missing, 1, "Should insert exactly one missing terminal");
    }
}

#[test]
fn test_extras_in_terminal_boundary() {
    // Verify that extras are properly within the terminal boundary
    let table = create_grammar_with_extras();
    
    // WS (SymbolId(2)) is an extra
    assert!(table.extras.contains(&SymbolId(2)), "WS should be marked as extra");
    
    // All extras should be less than token_count
    for &extra in &table.extras {
        assert!((extra.0 as usize) < table.token_count, 
                "Extra {} should be < token_count {}", extra.0, table.token_count);
    }
    
    // Terminal boundary should include all terminals
    let tb = table.terminal_boundary();
    assert_eq!(tb, table.token_count + table.external_token_count);
    
    // EOF should equal terminal boundary
    assert_eq!(table.eof_symbol.0 as usize, tb, 
              "EOF should equal terminal_boundary");
}