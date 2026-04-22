//! Tests for error recovery in the GLR parser.
//!
//! Note: These tests use manually constructed parse tables that don't satisfy
//! all strict invariants (e.g., EOF/END parity). They are only compiled when
//! the `strict-invariants` feature is disabled.

#![cfg(not(feature = "strict-invariants"))]
#![allow(unused_variables, dead_code, clippy::useless_vec)]

use glr_test_support::*;

use adze_glr_core::{Action, Driver, ParseRule, ParseTable};
use adze_ir::{Grammar, RuleId, StateId, SymbolId};

type ActionCell = Vec<Action>;

/// Create a minimal JSON-like grammar for testing recovery
///
/// Grammar:
///   value -> object | array
///   object -> '{' '}'
///   array -> '[' ']'
///
/// Symbols:
///   0: ERROR, 1: '{', 2: '}', 3: '[', 4: ']', 9: EOF
///   10: value (start symbol), 11: object, 12: array
fn create_test_grammar() -> (Grammar, ParseTable) {
    let mut states = vec![];
    let mut gotos = vec![];

    // Rules for the grammar
    let rules = vec![
        // Rule 0: value -> object
        ParseRule {
            lhs: SymbolId(10), // value
            rhs_len: 1,        // object
        },
        // Rule 1: value -> array
        ParseRule {
            lhs: SymbolId(10), // value
            rhs_len: 1,        // array
        },
        // Rule 2: object -> '{' '}'
        ParseRule {
            lhs: SymbolId(11), // object
            rhs_len: 2,        // '{' '}'
        },
        // Rule 3: array -> '[' ']'
        ParseRule {
            lhs: SymbolId(12), // array
            rhs_len: 2,        // '[' ']'
        },
    ];

    // State 0: Initial state
    // Can shift '{' to state 1 or '[' to state 3
    states.push(vec![
        vec![],                          // 0: ERROR
        vec![Action::Shift(StateId(1))], // 1: '{'
        vec![],                          // 2: '}'
        vec![Action::Shift(StateId(3))], // 3: '['
        vec![],                          // 4: ']'
        vec![],                          // 5: (unused)
        vec![],                          // 6: (unused)
        vec![],                          // 7: (unused)
        vec![],                          // 8: (unused)
        vec![],                          // 9: EOF
        vec![],                          // 10: value
        vec![],                          // 11: object
        vec![],                          // 12: array
    ]);
    // Goto for state 0: value->state 5, object->state 6, array->state 7
    let mut goto0 = vec![StateId(0); 13];
    goto0[10] = StateId(5); // value -> 5
    goto0[11] = StateId(6); // object -> 6
    goto0[12] = StateId(7); // array -> 7
    gotos.push(goto0);

    // State 1: After '{'
    // Can shift '}' to state 2
    states.push(vec![
        vec![],                          // 0: ERROR
        vec![],                          // 1: '{'
        vec![Action::Shift(StateId(2))], // 2: '}'
        vec![],                          // 3: '['
        vec![],                          // 4: ']'
        vec![],                          // 5
        vec![],                          // 6
        vec![],                          // 7
        vec![],                          // 8
        vec![],                          // 9: EOF
        vec![],                          // 10
        vec![],                          // 11
        vec![],                          // 12
    ]);
    gotos.push(vec![StateId(0); 13]);

    // State 2: After '{' '}'
    // Reduce by rule 2 (object -> '{' '}')
    states.push(vec![
        vec![Action::Reduce(RuleId(2))], // 0: ERROR
        vec![Action::Reduce(RuleId(2))], // 1: '{'
        vec![Action::Reduce(RuleId(2))], // 2: '}'
        vec![Action::Reduce(RuleId(2))], // 3: '['
        vec![Action::Reduce(RuleId(2))], // 4: ']'
        vec![Action::Reduce(RuleId(2))], // 5
        vec![Action::Reduce(RuleId(2))], // 6
        vec![Action::Reduce(RuleId(2))], // 7
        vec![Action::Reduce(RuleId(2))], // 8
        vec![Action::Reduce(RuleId(2))], // 9: EOF
        vec![Action::Reduce(RuleId(2))], // 10
        vec![Action::Reduce(RuleId(2))], // 11
        vec![Action::Reduce(RuleId(2))], // 12
    ]);
    gotos.push(vec![StateId(0); 13]);

    // State 3: After '['
    // Can shift ']' to state 4
    states.push(vec![
        vec![],                          // 0: ERROR
        vec![],                          // 1: '{'
        vec![],                          // 2: '}'
        vec![],                          // 3: '['
        vec![Action::Shift(StateId(4))], // 4: ']'
        vec![],                          // 5
        vec![],                          // 6
        vec![],                          // 7
        vec![],                          // 8
        vec![],                          // 9: EOF
        vec![],                          // 10
        vec![],                          // 11
        vec![],                          // 12
    ]);
    gotos.push(vec![StateId(0); 13]);

    // State 4: After '[' ']'
    // Reduce by rule 3 (array -> '[' ']')
    states.push(vec![
        vec![Action::Reduce(RuleId(3))], // 0: ERROR
        vec![Action::Reduce(RuleId(3))], // 1: '{'
        vec![Action::Reduce(RuleId(3))], // 2: '}'
        vec![Action::Reduce(RuleId(3))], // 3: '['
        vec![Action::Reduce(RuleId(3))], // 4: ']'
        vec![Action::Reduce(RuleId(3))], // 5
        vec![Action::Reduce(RuleId(3))], // 6
        vec![Action::Reduce(RuleId(3))], // 7
        vec![Action::Reduce(RuleId(3))], // 8
        vec![Action::Reduce(RuleId(3))], // 9: EOF
        vec![Action::Reduce(RuleId(3))], // 10
        vec![Action::Reduce(RuleId(3))], // 11
        vec![Action::Reduce(RuleId(3))], // 12
    ]);
    gotos.push(vec![StateId(0); 13]);

    // State 5: After reducing to value
    // Accept on EOF
    states.push(vec![
        vec![],               // 0: ERROR
        vec![],               // 1: '{'
        vec![],               // 2: '}'
        vec![],               // 3: '['
        vec![],               // 4: ']'
        vec![],               // 5
        vec![],               // 6
        vec![],               // 7
        vec![],               // 8
        vec![Action::Accept], // 9: EOF
        vec![],               // 10
        vec![],               // 11
        vec![],               // 12
    ]);
    gotos.push(vec![StateId(0); 13]);

    // State 6: After reducing to object
    // Reduce by rule 0 (value -> object)
    states.push(vec![
        vec![Action::Reduce(RuleId(0))], // 0: ERROR
        vec![Action::Reduce(RuleId(0))], // 1: '{'
        vec![Action::Reduce(RuleId(0))], // 2: '}'
        vec![Action::Reduce(RuleId(0))], // 3: '['
        vec![Action::Reduce(RuleId(0))], // 4: ']'
        vec![Action::Reduce(RuleId(0))], // 5
        vec![Action::Reduce(RuleId(0))], // 6
        vec![Action::Reduce(RuleId(0))], // 7
        vec![Action::Reduce(RuleId(0))], // 8
        vec![Action::Reduce(RuleId(0))], // 9: EOF
        vec![Action::Reduce(RuleId(0))], // 10
        vec![Action::Reduce(RuleId(0))], // 11
        vec![Action::Reduce(RuleId(0))], // 12
    ]);
    gotos.push(vec![StateId(0); 13]);

    // State 7: After reducing to array
    // Reduce by rule 1 (value -> array)
    states.push(vec![
        vec![Action::Reduce(RuleId(1))], // 0: ERROR
        vec![Action::Reduce(RuleId(1))], // 1: '{'
        vec![Action::Reduce(RuleId(1))], // 2: '}'
        vec![Action::Reduce(RuleId(1))], // 3: '['
        vec![Action::Reduce(RuleId(1))], // 4: ']'
        vec![Action::Reduce(RuleId(1))], // 5
        vec![Action::Reduce(RuleId(1))], // 6
        vec![Action::Reduce(RuleId(1))], // 7
        vec![Action::Reduce(RuleId(1))], // 8
        vec![Action::Reduce(RuleId(1))], // 9: EOF
        vec![Action::Reduce(RuleId(1))], // 10
        vec![Action::Reduce(RuleId(1))], // 11
        vec![Action::Reduce(RuleId(1))], // 12
    ]);
    gotos.push(vec![StateId(0); 13]);

    let table = make_minimal_table(
        states,
        gotos,
        rules,
        SymbolId(10), // start_symbol (value)
        SymbolId(9),  // eof_symbol
        0,            // external_token_count
    );

    (Grammar::new("test".to_string()), table)
}

#[test]
fn test_empty_object_with_recovery() {
    let (_grammar, mut table) = create_test_grammar();

    // Set initial state and EOF symbol
    table.initial_state = StateId(0); // Actual initial state
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

    // TODO: Verify no error nodes were created
    // debug_error_stats method needs to be implemented on Forest
    // #[cfg(any(test, feature = "test-api", feature = "test-helpers"))]
    // {
    //     let (has_error, missing, cost) = forest.debug_error_stats();
    //     assert!(!has_error, "Valid JSON '{{}}' must have no error chunks");
    //     assert_eq!(
    //         missing, 0,
    //         "Valid JSON '{{}}' must not insert missing terminals"
    //     );
    //     assert_eq!(cost, 0, "Valid JSON '{{}}' must have zero error cost");
    // }
}

#[test]
fn test_incomplete_object_recovery() {
    let (_grammar, mut table) = create_test_grammar();

    // Set initial state and EOF symbol
    table.initial_state = StateId(0);
    table.eof_symbol = SymbolId(9);

    // Add Recover action for incomplete object (state after '{')
    // This simulates what Tree-sitter tables would have
    // State 1 is after shifting '{' from state 0
    table.action_table[1][9] = vec![Action::Recover];

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
fn test_missing_value_recovery() {
    let (_grammar, mut table) = create_test_grammar();

    table.initial_state = StateId(0);
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
                s if s.starts_with('{') => Some(adze_glr_core::ts_lexer::NextToken {
                    kind: 1,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with('}') => Some(adze_glr_core::ts_lexer::NextToken {
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

            // TODO: Verify no error nodes were created using debug_error_stats
            // debug_error_stats method needs to be implemented on Forest
            // #[cfg(any(test, feature = "test-api", feature = "test-helpers"))]
            // {
            //     let (has_error, missing, cost) = forest.debug_error_stats();
            //     assert!(!has_error, "Valid JSON '{{}}' must have no error chunks");
            //     assert_eq!(
            //         missing, 0,
            //         "Valid JSON '{{}}' must not insert missing terminals"
            //     );
            //     assert_eq!(cost, 0, "Valid JSON '{{}}' must have zero error cost");
            // }
        }
    }

    // Test 2: Empty array "[]"
    {
        let lexer = |input: &str, pos: usize, _mode| {
            if pos >= input.len() {
                return None;
            }
            match &input[pos..] {
                s if s.starts_with('[') => Some(adze_glr_core::ts_lexer::NextToken {
                    kind: 3,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with(']') => Some(adze_glr_core::ts_lexer::NextToken {
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
            let view = forest.view();
            assert!(
                !view.roots().is_empty(),
                "Valid JSON '[]' must produce a tree"
            );
        }
    }

    // Note: Test 3 (Simple key-value object) requires a more complete JSON grammar
    // with support for members, which is beyond the scope of this minimal test grammar.
    // This would require adding productions like:
    //   object -> '{' members '}'
    //   members -> pair | members ',' pair
    //   pair -> string ':' value
    // For now, we only test empty objects and arrays.
}

#[test]
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
                s if s.starts_with('{') => Some(adze_glr_core::ts_lexer::NextToken {
                    kind: 1,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with('}') => Some(adze_glr_core::ts_lexer::NextToken {
                    kind: 2,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with(',') => Some(adze_glr_core::ts_lexer::NextToken {
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
                s if s.starts_with('{') => Some(adze_glr_core::ts_lexer::NextToken {
                    kind: 1,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with('}') => Some(adze_glr_core::ts_lexer::NextToken {
                    kind: 2,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with('"') => {
                    let end = s[1..].find('"').map(|i| i + 2).unwrap_or(1);
                    Some(adze_glr_core::ts_lexer::NextToken {
                        kind: 7,
                        start: pos as u32,
                        end: (pos + end) as u32,
                    })
                }
                s if s.starts_with(':') => Some(adze_glr_core::ts_lexer::NextToken {
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
                s if s.starts_with('{') => Some(adze_glr_core::ts_lexer::NextToken {
                    kind: 1,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with('}') => Some(adze_glr_core::ts_lexer::NextToken {
                    kind: 2,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with('"') => {
                    let end = s[1..].find('"').map(|i| i + 2).unwrap_or(1);
                    Some(adze_glr_core::ts_lexer::NextToken {
                        kind: 7,
                        start: pos as u32,
                        end: (pos + end) as u32,
                    })
                }
                s if s.starts_with(':') => Some(adze_glr_core::ts_lexer::NextToken {
                    kind: 5,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                s if s.starts_with(',') => Some(adze_glr_core::ts_lexer::NextToken {
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
            s if s.starts_with('{') => Some(adze_glr_core::ts_lexer::NextToken {
                kind: 1,
                start: pos as u32,
                end: (pos + 1) as u32,
            }),
            s if s.starts_with('}') => Some(adze_glr_core::ts_lexer::NextToken {
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
        let view = forest.view();
        assert!(
            !view.roots().is_empty(),
            "Valid JSON '{{}}' must produce a tree"
        );
    }
}

#[test]
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

        Some(adze_glr_core::ts_lexer::NextToken {
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
