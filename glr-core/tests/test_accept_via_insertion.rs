//! Test that accept-via-insertion at EOF has cost of exactly 1

// This test requires a parse table that violates the EOF/END parity invariant
// to trigger the specific recovery scenario we're testing
#[cfg(all(feature = "test-helpers", not(feature = "strict-invariants")))]
#[test]
fn accept_via_insertion_at_eof_cost_is_one() {
    use adze_glr_core::{Action, Driver, LexMode, ParseRule, ParseTable};
    use adze_ir::{RuleId, StateId, SymbolId};
    use std::collections::BTreeMap;

    // Minimal grammar: start -> LBRACE RBRACE
    let mut parse_table = ParseTable {
        action_table: vec![
            // State 0: shift LBRACE to state 1
            vec![
                vec![],                          // ERROR column
                vec![Action::Shift(StateId(1))], // LBRACE column
                vec![],                          // RBRACE column (END - last terminal)
                vec![],                          // EOF column
            ],
            // State 1: shift RBRACE to state 2, no action on EOF (triggers recovery)
            vec![
                vec![],                          // ERROR column
                vec![],                          // LBRACE column
                vec![Action::Shift(StateId(2))], // RBRACE column (END - last terminal)
                vec![], // EOF column - empty to trigger recovery, violates parity but test needs it
            ],
            // State 2: reduce by rule 0 (accept)
            vec![
                vec![],                          // ERROR column
                vec![],                          // LBRACE column
                vec![],                          // RBRACE column (END - last terminal)
                vec![Action::Reduce(RuleId(0))], // EOF column - reduce by rule 0
            ],
            // State 3: accept state
            vec![
                vec![],               // ERROR column
                vec![],               // LBRACE column
                vec![],               // RBRACE column (END - last terminal)
                vec![Action::Accept], // EOF column - accept
            ],
        ],
        goto_table: vec![
            // State 0 gotos (indexed by nonterminal_to_index)
            vec![StateId(3)], // Symbol 4 (start) at index 0 -> go to state 3 (accept state)
            // State 1 gotos
            vec![],
            // State 2 gotos
            vec![],
            // State 3 (accept state)
            vec![],
        ],
        rules: vec![
            ParseRule {
                lhs: SymbolId(4),
                rhs_len: 2,
            }, // Rule 0: start -> LBRACE RBRACE
        ],
        state_count: 4, // States 0, 1, 2, 3
        symbol_count: 5,
        symbol_to_index: {
            let mut map = BTreeMap::new();
            map.insert(SymbolId(0), 3); // EOF at index 3 (normalized to 0)
            map.insert(SymbolId(1), 1); // LBRACE at index 1
            map.insert(SymbolId(2), 2); // RBRACE at index 2
            map
        },
        index_to_symbol: vec![SymbolId(0), SymbolId(1), SymbolId(2), SymbolId(0)], // Reverse mapping
        external_scanner_states: vec![],
        nonterminal_to_index: {
            let mut map = BTreeMap::new();
            map.insert(SymbolId(4), 0); // start symbol at index 0
            map
        },
        goto_indexing: adze_glr_core::GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0), // EOF must be 0 by convention
        start_symbol: SymbolId(4),
        grammar: adze_ir::Grammar::new("test".to_string()),
        initial_state: StateId(0),
        token_count: 3, // ERROR, LBRACE, RBRACE
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0
            };
            4
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![0],
        rule_assoc_by_rule: vec![0], // No associativity for the single rule
        alias_sequences: vec![],
        symbol_metadata: vec![],
        field_map: BTreeMap::new(),
        field_names: vec![],
    };

    let mut driver = Driver::new(&parse_table);

    // Feed tokens: LBRACE then EOF (missing RBRACE)
    let result = driver.parse_tokens([
        (1u32, 0u32, 1u32), // LBRACE at position 0-1
        (3u32, 1u32, 1u32), // EOF at position 1-1
    ]);

    match result {
        Ok(forest) => {
            // Get error stats
            let (has_error, missing, cost) = forest.debug_error_stats();

            // Should have exactly 1 missing terminal (RBRACE) with cost = 1
            assert!(
                !has_error,
                "Accept-via-insertion should not produce error chunks"
            );
            assert_eq!(
                missing, 1,
                "Should have exactly 1 missing terminal (RBRACE), got {}",
                missing
            );
            assert_eq!(cost, 1, "Single insertion should have cost=1, got {}", cost);

            println!("✓ Accept-via-insertion at EOF correctly has cost=1");
        }
        Err(e) => {
            panic!("Parse should succeed via recovery: {}", e);
        }
    }
}
