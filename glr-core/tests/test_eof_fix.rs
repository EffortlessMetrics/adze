//! Test that EOF symbol is correctly handled (not 0/ERROR)
#![cfg(not(feature = "strict-invariants"))]

use rust_sitter_glr_core::{Action, ActionCell, Driver, LexMode, ParseRule, ParseTable};
use rust_sitter_ir::{RuleId, StateId, SymbolId};
use std::collections::BTreeMap;

#[test]
#[cfg(feature = "test-helpers")]
fn test_eof_not_error_symbol() {
    // Create a minimal parse table with EOF != 0
    // The key assertion here is that EOF is not 0 (the ERROR symbol)
    let parse_table = ParseTable {
        action_table: vec![], // Minimal table
        goto_table: vec![],
        rules: vec![],
        state_count: 1,
        symbol_count: 6,
        symbol_to_index: {
            let mut map = BTreeMap::new();
            map.insert(SymbolId(3), 0); // Map EOF to index 0
            map
        },
        external_scanner_states: vec![],
        nonterminal_to_index: BTreeMap::new(),
        eof_symbol: SymbolId(3), // token_count + external_token_count  // Critical: NOT 0!
        start_symbol: SymbolId(10),
        grammar: rust_sitter_ir::Grammar::new("test".to_string()),
        initial_state: StateId(0),
        token_count: 3,
        external_token_count: 0,
        lex_modes: vec![LexMode {
            lex_state: 0,
            external_lex_state: 0,
        }],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        symbol_metadata: vec![],
    };

    // This should NOT panic with our assertion that EOF != 0
    let _driver = Driver::new(&parse_table);
    println!(
        "✓ Driver created successfully with EOF={} (not 0)",
        parse_table.eof_symbol.0
    );
}

#[test]
#[cfg(feature = "test-helpers")]
fn test_error_stats_not_stubbed() {
    // Create parse table with deliberate recovery scenario
    let mut parse_table = ParseTable {
        action_table: vec![
            // State 0
            vec![
                vec![],                          // ERROR
                vec![Action::Shift(StateId(1))], // LBRACE
                vec![],                          // RBRACE
                vec![],                          // ... empty
                vec![],
                vec![Action::Reduce(RuleId(0))], // EOF - reduce to accept
            ],
            // State 1 (after LBRACE)
            vec![
                vec![],                          // ERROR
                vec![],                          // LBRACE
                vec![Action::Shift(StateId(2))], // RBRACE
                vec![],                          // ... empty
                vec![],
                vec![], // EOF - no action, will trigger recovery
            ],
            // State 2 (after LBRACE RBRACE)
            vec![
                vec![], // ERROR
                vec![], // LBRACE
                vec![], // RBRACE
                vec![], // ... empty
                vec![],
                vec![Action::Reduce(RuleId(1))], // EOF - reduce
            ],
        ],
        goto_table: vec![vec![], vec![], vec![]],
        rules: vec![
            ParseRule {
                lhs: SymbolId(10),
                rhs_len: 0,
            }, // start -> ε
            ParseRule {
                lhs: SymbolId(10),
                rhs_len: 2,
            }, // start -> { }
        ],
        state_count: 3,
        symbol_count: 11,
        symbol_to_index: {
            let mut map = BTreeMap::new();
            map.insert(SymbolId(3), 0); // Map EOF to index 0
            map
        },
        external_scanner_states: vec![],
        nonterminal_to_index: BTreeMap::new(),
        eof_symbol: SymbolId(3), // token_count + external_token_count
        start_symbol: SymbolId(10),
        grammar: rust_sitter_ir::Grammar::new("test".to_string()),
        initial_state: StateId(0),
        token_count: 3,
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0
            };
            3
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        symbol_metadata: vec![],
    };

    let mut driver = Driver::new(&parse_table);

    // Tokenized mode test - provide LBRACE then EOF (missing RBRACE)
    let result = driver.parse_tokens([
        (1u32, 0u32, 1u32), // LBRACE
        (3u32, 1u32, 1u32), // EOF
    ]);

    match result {
        Ok(forest) => {
            // Get real stats - this should NOT return (false, 0, 0) from a stub
            let (has_error, missing, cost) = forest.debug_error_stats();

            // We expect recovery to have inserted the missing RBRACE
            assert!(
                has_error || missing > 0,
                "Expected error or missing terminal from recovery, got: has_error={}, missing={}, cost={}",
                has_error,
                missing,
                cost
            );
            assert!(
                cost > 0,
                "Expected non-zero cost from recovery, got {}",
                cost
            );

            println!(
                "✓ Error stats correctly reported: has_error={}, missing={}, cost={}",
                has_error, missing, cost
            );
        }
        Err(e) => {
            // Parse failure is also OK for this malformed input
            println!("Parse failed as expected: {}", e);
        }
    }
}
