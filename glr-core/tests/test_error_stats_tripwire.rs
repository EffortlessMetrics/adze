//! Trip-wire test to prevent regression of debug_error_stats to silent zeros
#![cfg(not(feature = "strict-invariants"))]

use rust_sitter_glr_core::{Action, Driver, LexMode, ParseRule, ParseTable};
use rust_sitter_ir::{RuleId, StateId, SymbolId};
use std::collections::BTreeMap;

#[test]
fn test_error_stats_not_stubbed() {
    // Create a minimal grammar that forces an error recovery
    // Grammar: S -> 'a' 'b' | 'c'
    // Input: 'a' EOF (missing 'b')
    // Expected: error recovery with missing terminal

    let parse_table = ParseTable {
        action_table: vec![
            // State 0: initial
            vec![
                vec![],                          // 0: ERROR
                vec![Action::Shift(StateId(1))], // 1: 'a'
                vec![],                          // 2: 'b'
                vec![Action::Shift(StateId(3))], // 3: 'c'
                vec![],                          // 4: EOF
            ],
            // State 1: after 'a'
            vec![
                vec![],                          // 0: ERROR
                vec![],                          // 1: 'a'
                vec![Action::Shift(StateId(2))], // 2: 'b'
                vec![],                          // 3: 'c'
                vec![],                          // 4: EOF - no action, will force recovery
            ],
            // State 2: after 'a' 'b'
            vec![
                vec![],                          // 0: ERROR
                vec![],                          // 1: 'a'
                vec![],                          // 2: 'b'
                vec![],                          // 3: 'c'
                vec![Action::Reduce(RuleId(0))], // 4: EOF -> reduce S -> 'a' 'b'
            ],
            // State 3: after 'c'
            vec![
                vec![],                          // 0: ERROR
                vec![],                          // 1: 'a'
                vec![],                          // 2: 'b'
                vec![],                          // 3: 'c'
                vec![Action::Reduce(RuleId(1))], // 4: EOF -> reduce S -> 'c'
            ],
        ],
        goto_table: vec![
            vec![StateId(4)], // State 0: goto[S] = 4
            vec![StateId(0)], // State 1: no gotos
            vec![StateId(0)], // State 2: no gotos
            vec![StateId(0)], // State 3: no gotos
        ],
        rules: vec![
            ParseRule {
                lhs: SymbolId(5),
                rhs_len: 2,
            }, // S -> 'a' 'b'
            ParseRule {
                lhs: SymbolId(5),
                rhs_len: 1,
            }, // S -> 'c'
        ],
        state_count: 4,
        symbol_count: 6,
        symbol_to_index: {
            let mut map = BTreeMap::new();
            map.insert(SymbolId(0), 0); // ERROR
            map.insert(SymbolId(1), 1); // 'a'
            map.insert(SymbolId(2), 2); // 'b'
            map.insert(SymbolId(3), 3); // 'c'
            map.insert(SymbolId(4), 4); // EOF
            map.insert(SymbolId(5), 0); // S (nonterminal, reuses index 0)
            map
        },
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        nonterminal_to_index: {
            let mut map = BTreeMap::new();
            map.insert(SymbolId(5), 0); // S
            map
        },
        eof_symbol: SymbolId(4),
        start_symbol: SymbolId(5),
        grammar: rust_sitter_ir::Grammar::new("test".to_string()),
        initial_state: StateId(0),
        token_count: 4, // ERROR, 'a', 'b', 'c'
        external_token_count: 0,
        symbol_metadata: vec![],
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0
            };
            4
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    };

    let mut driver = Driver::new(&parse_table);

    // Parse "a" EOF - missing 'b' should trigger recovery
    let result = driver.parse_tokens([
        (1u32, 0u32, 1u32), // 'a'
        (4u32, 1u32, 1u32), // EOF
    ]);

    match result {
        Ok(forest) => {
            #[cfg(feature = "test-api")]
            {
                let (has_error, missing, cost) = forest.debug_error_stats();

                // THE CRITICAL ASSERTION: If error recovery happened, stats must show it
                // This prevents regression to stub returning (false, 0, 0)
                assert!(
                    has_error || missing > 0 || cost > 0,
                    "ERROR: debug_error_stats returned all zeros for a parse requiring recovery!\n\
                     This indicates the method has regressed to a stub implementation.\n\
                     Got: has_error={}, missing={}, cost={}\n\
                     Expected: At least one non-zero value since 'b' is missing",
                    has_error,
                    missing,
                    cost
                );

                // More specific check: we expect exactly 1 missing terminal ('b')
                assert_eq!(
                    missing, 1,
                    "Expected exactly 1 missing terminal ('b'), got {}",
                    missing
                );

                println!(
                    "✓ Error stats correctly reported: has_error={}, missing={}, cost={}",
                    has_error, missing, cost
                );
            }
            #[cfg(not(feature = "test-api"))]
            println!("✓ Parse with recovery completed");
        }
        Err(_) => {
            panic!("Parse should succeed with recovery, not fail entirely");
        }
    }
}

#[test]
fn test_clean_parse_has_zero_errors() {
    // Complementary test: valid input should have zero error stats
    let parse_table = ParseTable {
        action_table: vec![
            // State 0: initial
            vec![
                vec![],                          // 0: ERROR
                vec![Action::Shift(StateId(1))], // 1: 'a'
                vec![],                          // 2: 'b'
                vec![Action::Shift(StateId(3))], // 3: 'c'
                vec![],                          // 4: EOF
            ],
            // State 1: after 'a'
            vec![
                vec![],                          // 0: ERROR
                vec![],                          // 1: 'a'
                vec![Action::Shift(StateId(2))], // 2: 'b'
                vec![],                          // 3: 'c'
                vec![],                          // 4: EOF
            ],
            // State 2: after 'a' 'b'
            vec![
                vec![],                          // 0: ERROR
                vec![],                          // 1: 'a'
                vec![],                          // 2: 'b'
                vec![],                          // 3: 'c'
                vec![Action::Reduce(RuleId(0))], // 4: EOF -> reduce S -> 'a' 'b'
            ],
            // State 3: after 'c'
            vec![
                vec![],                          // 0: ERROR
                vec![],                          // 1: 'a'
                vec![],                          // 2: 'b'
                vec![],                          // 3: 'c'
                vec![Action::Reduce(RuleId(1))], // 4: EOF -> reduce S -> 'c'
            ],
            // State 4: after S (accept state)
            vec![
                vec![],               // 0: ERROR
                vec![],               // 1: 'a'
                vec![],               // 2: 'b'
                vec![],               // 3: 'c'
                vec![Action::Accept], // 4: EOF -> accept
            ],
        ],
        goto_table: vec![
            vec![StateId(4)], // State 0: goto[S] = 4
            vec![StateId(0)], // State 1: no gotos
            vec![StateId(0)], // State 2: no gotos
            vec![StateId(0)], // State 3: no gotos
            vec![StateId(0)], // State 4: no gotos
        ],
        rules: vec![
            ParseRule {
                lhs: SymbolId(5),
                rhs_len: 2,
            }, // S -> 'a' 'b'
            ParseRule {
                lhs: SymbolId(5),
                rhs_len: 1,
            }, // S -> 'c'
        ],
        state_count: 5,
        symbol_count: 6,
        symbol_to_index: {
            let mut map = BTreeMap::new();
            map.insert(SymbolId(0), 0); // ERROR
            map.insert(SymbolId(1), 1); // 'a'
            map.insert(SymbolId(2), 2); // 'b'
            map.insert(SymbolId(3), 3); // 'c'
            map.insert(SymbolId(4), 4); // EOF
            map.insert(SymbolId(5), 0); // S (nonterminal, reuses index 0)
            map
        },
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        nonterminal_to_index: {
            let mut map = BTreeMap::new();
            map.insert(SymbolId(5), 0); // S
            map
        },
        eof_symbol: SymbolId(4),
        start_symbol: SymbolId(5),
        grammar: rust_sitter_ir::Grammar::new("test".to_string()),
        initial_state: StateId(0),
        token_count: 4, // ERROR, 'a', 'b', 'c'
        external_token_count: 0,
        symbol_metadata: vec![],
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0
            };
            5
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    };

    let mut driver = Driver::new(&parse_table);

    // Parse "a b" EOF - complete valid input
    let result = driver.parse_tokens([
        (1u32, 0u32, 1u32), // 'a'
        (2u32, 1u32, 1u32), // 'b'
        (4u32, 2u32, 1u32), // EOF
    ]);

    match result {
        Ok(forest) => {
            #[cfg(feature = "test-api")]
            {
                let (has_error, missing, cost) = forest.debug_error_stats();

                // Clean parse should have all zeros
                assert_eq!(
                    (has_error, missing, cost),
                    (false, 0, 0),
                    "Clean parse should have zero error stats"
                );

                println!(
                    "✓ Clean parse correctly has zero errors: has_error={}, missing={}, cost={}",
                    has_error, missing, cost
                );
            }
            #[cfg(not(feature = "test-api"))]
            println!("✓ Clean parse completed");
        }
        Err(e) => {
            panic!("Valid input should parse successfully: {}", e);
        }
    }
}
