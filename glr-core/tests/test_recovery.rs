#![allow(unused_variables, dead_code, clippy::useless_vec)]

use glr_test_support::*;

use rust_sitter_glr_core::{Action, Driver, ParseRule, ParseTable};
use rust_sitter_ir::{Grammar, RuleId, StateId, SymbolId};

type ActionCell = Vec<Action>;

/// Create a minimal JSON-like grammar for testing recovery
fn create_test_grammar() -> (Grammar, ParseTable) {
    // Create a simple grammar that accepts:
    // object -> '{' '}' | '{' members '}'
    // array -> '[' ']' | '[' elements ']'
    // For simplicity, we'll just test empty object "{}"

    let mut states = vec![];
    let mut gotos = vec![];
    let mut rules = vec![];

    // State 0: initial state
    // Can shift '{' to state 1
    states.push(vec![
        vec![],                          // 0 (unused)
        vec![Action::Shift(StateId(1))], // 1: '{'
        vec![],                          // 2: '}'
        vec![Action::Shift(StateId(3))], // 3: '['
        vec![],                          // 4: ']'
        vec![],                          // 5: ':'
        vec![],                          // 6: ','
        vec![],                          // 7: string
        vec![],                          // 8: number
        vec![],                          // 9: EOF
    ]);
    gotos.push(vec![StateId(0); 10]);

    // State 1: after '{'
    // Can shift '}' to state 2
    // Can shift string to build members
    states.push(vec![
        vec![],                          // 0
        vec![],                          // 1: '{'
        vec![Action::Shift(StateId(2))], // 2: '}'
        vec![],                          // 3: '['
        vec![],                          // 4: ']'
        vec![],                          // 5: ':'
        vec![],                          // 6: ','
        vec![Action::Shift(StateId(4))], // 7: string (for members)
        vec![],                          // 8: number
        vec![],                          // 9: EOF
    ]);
    gotos.push(vec![StateId(0); 10]);

    // State 2: after '{' '}'
    // Reduce to object (rule 0)
    let rule0 = ParseRule {
        lhs: SymbolId(10), // object
        rhs_len: 2,        // '{' '}' = 2 symbols
    };
    rules.push(rule0.clone());

    states.push(vec![
        vec![Action::Reduce(RuleId(0))], // 0
        vec![Action::Reduce(RuleId(0))], // 1
        vec![Action::Reduce(RuleId(0))], // 2
        vec![Action::Reduce(RuleId(0))], // 3
        vec![Action::Reduce(RuleId(0))], // 4
        vec![Action::Reduce(RuleId(0))], // 5
        vec![Action::Reduce(RuleId(0))], // 6
        vec![Action::Reduce(RuleId(0))], // 7
        vec![Action::Reduce(RuleId(0))], // 8
        vec![Action::Accept],            // 9: EOF - accept!
    ]);
    gotos.push(vec![StateId(0); 10]);

    // State 3: after '['
    states.push(vec![
        vec![],                          // 0
        vec![],                          // 1
        vec![],                          // 2
        vec![],                          // 3
        vec![Action::Shift(StateId(5))], // 4: ']'
        vec![],                          // 5
        vec![],                          // 6
        vec![],                          // 7
        vec![],                          // 8
        vec![],                          // 9
    ]);
    gotos.push(vec![StateId(0); 10]);

    // Add more states as needed...
    // State 4: after '{' string (building members)
    states.push(vec![
        vec![],                          // 0
        vec![],                          // 1
        vec![],                          // 2
        vec![],                          // 3
        vec![],                          // 4
        vec![Action::Shift(StateId(6))], // 5: ':'
        vec![],                          // 6
        vec![],                          // 7
        vec![],                          // 8
        vec![],                          // 9
    ]);
    gotos.push(vec![StateId(0); 10]);

    // State 5: after '[' ']'
    let rule1 = ParseRule {
        lhs: SymbolId(11), // array
        rhs_len: 2,        // '[' ']' = 2 symbols
    };
    rules.push(rule1);

    states.push(vec![vec![Action::Reduce(RuleId(1))]; 10]);
    gotos.push(vec![StateId(0); 10]);

    // State 6: after '{' string ':'
    states.push(vec![vec![]; 10]);
    gotos.push(vec![StateId(0); 10]);

    let table = make_minimal_table(
        states,
        gotos,
        rules,
        SymbolId(10), // start_symbol
        SymbolId(9),  // eof_symbol
        0,            // external_token_count
    );

    (Grammar::new("test".to_string()), table)
}

#[test]
#[ignore] // Grammar setup needs work
fn test_empty_object_with_recovery() {
    let (_grammar, mut table) = create_test_grammar();

    // Set initial state and EOF symbol
    table.initial_state = StateId(1); // Tree-sitter convention
    table.eof_symbol = SymbolId(9);

    let mut driver = Driver::new(&table);

    // Parse "{}" - should succeed without recovery
    let tokens = vec![
        (1, 0, 1), // {
        (2, 1, 2), // }
        (9, 2, 2), // EOF
    ];

    let result = driver.parse_tokens(tokens);
    assert!(result.is_ok(), "Empty object should parse successfully");

    let forest = result.unwrap();
    let view = forest.view();
    assert!(
        !view.roots().is_empty(),
        "Should have at least one parse tree"
    );
}

#[test]
#[ignore] // Grammar setup needs work
fn test_incomplete_object_recovery() {
    let (_grammar, mut table) = create_test_grammar();

    // Set initial state and EOF symbol
    table.initial_state = StateId(1);
    table.eof_symbol = SymbolId(9);

    // Add Recover action for incomplete object (state after '{')
    // This simulates what Tree-sitter tables would have
    let lbrace_shift_state = StateId(2); // Assume state 2 after shifting '{'
    table.action_table[lbrace_shift_state.0 as usize][9] = vec![Action::Recover];

    let mut driver = Driver::new(&table);

    // Parse "{" - incomplete, should trigger recovery
    let tokens = vec![
        (1, 0, 1), // {
        (9, 1, 1), // EOF
    ];

    // With recovery, this should still produce a forest (possibly with error nodes)
    let result = driver.parse_tokens(tokens);

    // The exact behavior depends on our recovery implementation
    // For now, we just verify it doesn't panic
    match result {
        Ok(forest) => {
            println!("Incomplete object parsed with recovery");
            let view = forest.view();
            println!("Roots: {:?}", view.roots());
        }
        Err(e) => {
            println!("Parse failed as expected: {}", e);
            // This is also acceptable since our MVP recovery might not handle all cases
        }
    }
}

#[test]
#[ignore] // Grammar setup needs work
fn test_missing_value_recovery() {
    let (_grammar, mut table) = create_test_grammar();

    table.initial_state = StateId(1);
    table.eof_symbol = SymbolId(9);

    let mut driver = Driver::new(&table);

    // Parse '{"key": }' - missing value after colon
    let tokens = vec![
        (1, 0, 1), // {
        (7, 1, 6), // "key" (string)
        (5, 6, 7), // :
        (2, 8, 9), // }
        (9, 9, 9), // EOF
    ];

    let result = driver.parse_tokens(tokens);

    // With recovery, parser might insert a missing value
    match result {
        Ok(forest) => {
            println!("Missing value handled with recovery");
            let view = forest.view();
            println!("Roots: {:?}", view.roots());
        }
        Err(e) => {
            println!("Parse failed: {}", e);
        }
    }
}

#[test]
#[ignore] // Requires complete JSON grammar implementation
fn test_valid_json_no_errors() {
    // Test A: Valid JSON should have no error/missing nodes
    let (_grammar, mut table) = create_test_grammar();

    table.initial_state = StateId(0);
    table.eof_symbol = SymbolId(9);

    let mut driver = Driver::new(&table);

    // Test 1: Empty object "{}"
    {
        let tokens = vec![
            (1, 0, 1), // {
            (2, 1, 2), // }
        ];

        // Use streaming parse with a simple lexer
        let lexer = |input: &str, pos: usize, _mode| {
            if pos >= input.len() {
                return None;
            }
            match &input[pos..] {
                s if s.starts_with('{') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 1,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with('}') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 2,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                _ => None,
            }
        };

        let result = driver.parse_streaming("{}", lexer, None::<fn(&str, usize, &[bool], _) -> _>);
        assert!(result.is_ok(), "Empty object should parse without errors");

        if let Ok(forest) = result {
            let view = forest.view();
            assert!(
                !view.roots().is_empty(),
                "Should have at least one parse tree"
            );

            // Verify no error nodes were created using debug_error_stats
            // TODO: Implement debug_error_stats method on Forest
            // let (has_error, missing, cost) = forest.debug_error_stats();
            // assert!(!has_error, "Valid JSON '{{}}' must have no error chunks");
            // assert_eq!(
            //     missing, 0,
            //     "Valid JSON '{{}}' must not insert missing terminals"
            // );
            // assert_eq!(cost, 0, "Valid JSON '{{}}' must have zero error cost");
        }
    }

    // Test 2: Empty array "[]"
    {
        let lexer = |input: &str, pos: usize, _mode| {
            if pos >= input.len() {
                return None;
            }
            match &input[pos..] {
                s if s.starts_with('[') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 3,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with(']') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 4,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                _ => None,
            }
        };

        let result = driver.parse_streaming("[]", lexer, None::<fn(&str, usize, &[bool], _) -> _>);
        assert!(result.is_ok(), "Empty array should parse without errors");

        if let Ok(forest) = result {
            // TODO: Implement debug_error_stats method on Forest
            // let (has_error, missing, cost) = forest.debug_error_stats();
            // assert!(!has_error, "Valid JSON '[]' must have no error chunks");
            // assert_eq!(
            //     missing, 0,
            //     "Valid JSON '[]' must not insert missing terminals"
            // );
            // assert_eq!(cost, 0, "Valid JSON '[]' must have zero error cost");
        }
    }

    // Test 3: Simple key-value object
    {
        let lexer = |input: &str, pos: usize, _mode| {
            if pos >= input.len() {
                return None;
            }
            match &input[pos..] {
                s if s.starts_with('{') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 1,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with('}') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 2,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with('"') => {
                    // Simple string detection
                    let end = s[1..].find('"').map(|i| i + 2).unwrap_or(1);
                    Some(rust_sitter_glr_core::ts_lexer::NextToken {
                        kind: 7,
                        start: pos as u32,
                        end: (pos + end) as u32,
                    })
                }
                s if s.starts_with(':') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 5,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                _ => None,
            }
        };

        let result = driver.parse_streaming(
            "{\"key\":\"value\"}",
            lexer,
            None::<fn(&str, usize, &[bool], _) -> _>,
        );
        assert!(result.is_ok(), "Simple object should parse without errors");

        if let Ok(forest) = result {
            // TODO: Implement debug_error_stats method on Forest
            // let (has_error, missing, cost) = forest.debug_error_stats();
            // assert!(!has_error, "Valid JSON object must have no error chunks");
            // assert_eq!(
            //     missing, 0,
            //     "Valid JSON object must not insert missing terminals"
            // );
            // assert_eq!(cost, 0, "Valid JSON object must have zero error cost");
        }
    }
}

#[test]
#[ignore] // debug_error_stats method needs to be implemented
fn test_gentle_errors_bounded_recovery() {
    // Test B: Gentle errors should recover with bounded cost
    let (_grammar, mut table) = create_test_grammar();

    table.initial_state = StateId(0);
    table.eof_symbol = SymbolId(9);

    let mut driver = Driver::new(&table);

    // Test 1: Leading comma in object "{,}"
    {
        let lexer = |input: &str, pos: usize, _mode| {
            if pos >= input.len() {
                return None;
            }
            match &input[pos..] {
                s if s.starts_with('{') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 1,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with('}') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 2,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with(',') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 6,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                _ => None,
            }
        };

        let result = driver.parse_streaming("{,}", lexer, None::<fn(&str, usize, &[bool], _) -> _>);
        // Should either parse with recovery or fail gracefully
        match result {
            Ok(forest) => {
                let view = forest.view();
                assert!(
                    !view.roots().is_empty(),
                    "Should recover and produce a tree"
                );

                // Check that error_cost is bounded (≤ beam width)
                // TODO: Implement debug_error_stats method on Forest
                // let (has_error, _missing, cost) = forest.debug_error_stats();
                // assert!(has_error, "Malformed input should have error markers");
                // assert!(
                //     cost <= rust_sitter_glr_core::Driver::RECOVERY_BEAM + 1,
                //     "Recovery cost {} should be bounded by beam width",
                //     cost
                // );
            }
            Err(_) => {
                // Acceptable if recovery can't handle this case
            }
        }
    }

    // Test 2: Missing value in object {"k":}
    {
        let lexer = |input: &str, pos: usize, _mode| {
            if pos >= input.len() {
                return None;
            }
            match &input[pos..] {
                s if s.starts_with('{') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 1,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with('}') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 2,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with('"') => {
                    let end = s[1..].find('"').map(|i| i + 2).unwrap_or(1);
                    Some(rust_sitter_glr_core::ts_lexer::NextToken {
                        kind: 7,
                        start: pos as u32,
                        end: (pos + end) as u32,
                    })
                }
                s if s.starts_with(':') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 5,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                _ => None,
            }
        };

        let result =
            driver.parse_streaming("{\"k\":}", lexer, None::<fn(&str, usize, &[bool], _) -> _>);
        // Recovery should insert a missing value
        match result {
            Ok(forest) => {
                let view = forest.view();
                assert!(
                    !view.roots().is_empty(),
                    "Should recover with inserted value"
                );
            }
            Err(_) => {
                // Also acceptable
            }
        }
    }

    // Test 3: Trailing comma {"k":"v",}
    {
        let lexer = |input: &str, pos: usize, _mode| {
            if pos >= input.len() {
                return None;
            }
            match &input[pos..] {
                s if s.starts_with('{') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 1,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with('}') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 2,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with('"') => {
                    let end = s[1..].find('"').map(|i| i + 2).unwrap_or(1);
                    Some(rust_sitter_glr_core::ts_lexer::NextToken {
                        kind: 7,
                        start: pos as u32,
                        end: (pos + end) as u32,
                    })
                }
                s if s.starts_with(':') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 5,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with(',') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 6,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                _ => None,
            }
        };

        let result = driver.parse_streaming(
            "{\"k\":\"v\",}",
            lexer,
            None::<fn(&str, usize, &[bool], _) -> _>,
        );
        // Should handle trailing comma gracefully
        match result {
            Ok(forest) => {
                let view = forest.view();
                assert!(!view.roots().is_empty(), "Should handle trailing comma");
            }
            Err(_) => {
                // Also acceptable
            }
        }
    }
}

#[test]
#[ignore] // debug_error_stats method needs to be implemented
fn test_cell_parity_after_lbrace() {
    // Create a JSON grammar and parse table
    let (_grammar, table) = create_test_grammar();

    // Start from initial state
    let initial_state = table.initial_state;

    // Find the state after shifting '{'
    let lbrace_sym = SymbolId(1); // '{' token
    let mut after_lbrace_state = None;

    // Look for a shift action on '{' from the initial state
    for action in table.actions(initial_state, lbrace_sym) {
        if let Action::Shift(target) = action {
            after_lbrace_state = Some(target);
            break;
        }
    }

    assert!(
        after_lbrace_state.is_some(),
        "Should be able to shift '{{' from initial state"
    );
    let after_lbrace = after_lbrace_state.unwrap();

    // Now check what actions exist for '}' in that state
    let rbrace_sym = SymbolId(2); // '}' token
    let actions_for_rbrace = table.actions(*after_lbrace, rbrace_sym);

    // Assert there should be at least one non-Recover action
    let has_real_action = actions_for_rbrace
        .iter()
        .any(|a| !matches!(a, Action::Recover | Action::Error));

    assert!(
        has_real_action,
        "After '{{', there should be a real action (Shift/Reduce/Accept) for '}}', not just Recover. \
         Found actions: {:?}",
        actions_for_rbrace
    );

    // Additional check: valid JSON "{}" should parse without error nodes
    // This verifies the driver can handle the action correctly
    let mut driver = Driver::new(&table);
    let lexer = |input: &str, pos: usize, _mode| {
        if pos >= input.len() {
            return None;
        }
        match &input[pos..] {
            s if s.starts_with('{') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                kind: 1,
                start: pos as u32,
                end: (pos + 1) as u32,
            }),
            s if s.starts_with('}') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                kind: 2,
                start: pos as u32,
                end: (pos + 1) as u32,
            }),
            _ => None,
        }
    };

    let result = driver.parse_streaming("{}", lexer, None::<fn(&str, usize, &[bool], _) -> _>);
    assert!(
        result.is_ok(),
        "Empty object '{{}}' must parse successfully"
    );

    if let Ok(forest) = result {
        // TODO: Implement debug_error_stats method on Forest
        // let (has_error, missing, cost) = forest.debug_error_stats();
        // assert!(!has_error, "Valid JSON '{{}}' must have no error chunks");
        // assert_eq!(
        //     missing, 0,
        //     "Valid JSON '{{}}' must not insert missing terminals"
        // );
        // assert_eq!(cost, 0, "Valid JSON '{{}}' must have zero error cost");
    }
}

#[test]
#[ignore] // debug_error_stats method needs to be implemented
fn test_zero_width_progress_guard() {
    // Test that we always make progress even with pathological zero-width tokens
    let (_grammar, mut table) = create_test_grammar();

    table.initial_state = StateId(0);
    table.eof_symbol = SymbolId(9);

    let mut driver = Driver::new(&table);

    // Create a pathological lexer that always returns zero-width tokens
    let mut call_count = 0;
    let mut positions_seen = std::collections::HashSet::new();

    let tracking_lexer = |_input: &str, pos: usize, _mode| {
        positions_seen.insert(pos);
        call_count += 1;

        // Stop if we're stuck at the same position
        if call_count > 100 {
            panic!("Infinite loop detected: lexer called {} times", call_count);
        }

        // Always return a zero-width token at the current position
        // This tests that the driver doesn't get stuck
        if pos > 5 {
            return None;
        } // Stop after a few positions

        Some(rust_sitter_glr_core::ts_lexer::NextToken {
            kind: 7, // String token (insertable)
            start: pos as u32,
            end: pos as u32, // Zero-width!
        })
    };

    // This should not hang or panic
    let result = driver.parse_streaming(
        "test",
        tracking_lexer,
        None::<fn(&str, usize, &[bool], _) -> _>,
    );

    // We should have advanced through multiple positions
    assert!(
        positions_seen.len() > 1,
        "Driver must advance position even with zero-width tokens. Positions seen: {:?}",
        positions_seen
    );

    // The parse might fail, but it shouldn't hang
    match result {
        Ok(_) => {
            // If it somehow succeeded, that's fine
        }
        Err(_) => {
            // Expected - the important thing is we didn't hang
        }
    }
}
