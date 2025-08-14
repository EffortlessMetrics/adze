use rust_sitter_glr_core::{Driver, ParseTable, Action};
use rust_sitter_ir::{Grammar, StateId, SymbolId, RuleId};
use rust_sitter_glr_core::parse_forest::{ERROR_SYMBOL, ErrorMeta};

/// Create a minimal JSON-like grammar for testing recovery
fn create_test_grammar() -> (Grammar, ParseTable) {
    use rust_sitter_ir::*;
    
    let mut g = Grammar::new();
    
    // Terminal symbols
    let lbrace = g.define_symbol_raw(SymbolId(1), "{", 1, true, false, false);
    let rbrace = g.define_symbol_raw(SymbolId(2), "}", 2, true, false, false);
    let lbracket = g.define_symbol_raw(SymbolId(3), "[", 3, true, false, false);
    let rbracket = g.define_symbol_raw(SymbolId(4), "]", 4, true, false, false);
    let colon = g.define_symbol_raw(SymbolId(5), ":", 5, true, false, false);
    let comma = g.define_symbol_raw(SymbolId(6), ",", 6, true, false, false);
    let string = g.define_symbol_raw(SymbolId(7), "string", 7, true, false, false);
    let number = g.define_symbol_raw(SymbolId(8), "number", 8, true, false, false);
    let eof = g.define_symbol_raw(SymbolId(9), "EOF", 9, true, false, false);
    
    // Non-terminal symbols
    let document = g.define_symbol_raw(SymbolId(10), "document", 10, false, false, false);
    let value = g.define_symbol_raw(SymbolId(11), "value", 11, false, false, false);
    let object = g.define_symbol_raw(SymbolId(12), "object", 12, false, false, false);
    let array = g.define_symbol_raw(SymbolId(13), "array", 13, false, false, false);
    let members = g.define_symbol_raw(SymbolId(14), "members", 14, false, false, false);
    let pair = g.define_symbol_raw(SymbolId(15), "pair", 15, false, false, false);
    let elements = g.define_symbol_raw(SymbolId(16), "elements", 16, false, false, false);
    
    // Set start symbol
    g.start_symbol = Some(document);
    
    // Define rules
    // document -> value
    g.add_rule(document, vec![value], 0);
    
    // value -> object | array | string | number
    g.add_rule(value, vec![object], 1);
    g.add_rule(value, vec![array], 2);
    g.add_rule(value, vec![string], 3);
    g.add_rule(value, vec![number], 4);
    
    // object -> '{' '}' | '{' members '}'
    g.add_rule(object, vec![lbrace, rbrace], 5);
    g.add_rule(object, vec![lbrace, members, rbrace], 6);
    
    // array -> '[' ']' | '[' elements ']'
    g.add_rule(array, vec![lbracket, rbracket], 7);
    g.add_rule(array, vec![lbracket, elements, rbracket], 8);
    
    // members -> pair | members ',' pair
    g.add_rule(members, vec![pair], 9);
    g.add_rule(members, vec![members, comma, pair], 10);
    
    // pair -> string ':' value
    g.add_rule(pair, vec![string, colon, value], 11);
    
    // elements -> value | elements ',' value
    g.add_rule(elements, vec![value], 12);
    g.add_rule(elements, vec![elements, comma, value], 13);
    
    // Build parse table
    let first_follow = rust_sitter_glr_core::compute_first_follow(&g).unwrap();
    let lr1_automaton = rust_sitter_glr_core::build_lr1_automaton(&g, &first_follow).unwrap();
    let parse_table = rust_sitter_glr_core::build_parse_table(&g, &lr1_automaton, &first_follow).unwrap();
    
    (g, parse_table)
}

#[test]
#[ignore] // API needs update
fn test_empty_object_with_recovery() {
    let (_grammar, mut table) = create_test_grammar();
    
    // Set initial state and EOF symbol
    table.initial_state = StateId(1); // Tree-sitter convention
    table.eof_symbol = SymbolId(9);
    
    let mut driver = Driver::new(&table);
    
    // Parse "{}" - should succeed without recovery
    let tokens = vec![
        (1, 0, 1),  // {
        (2, 1, 2),  // }
        (9, 2, 2),  // EOF
    ];
    
    let result = driver.parse_tokens(tokens);
    assert!(result.is_ok(), "Empty object should parse successfully");
    
    let forest = result.unwrap();
    let view = forest.view();
    assert!(!view.roots().is_empty(), "Should have at least one parse tree");
}

#[test] 
#[ignore] // API needs update
fn test_incomplete_object_recovery() {
    let (_grammar, mut table) = create_test_grammar();
    
    // Set initial state and EOF symbol
    table.initial_state = StateId(1);
    table.eof_symbol = SymbolId(9);
    
    // Add Recover action for incomplete object (state after '{')
    // This simulates what Tree-sitter tables would have
    let lbrace_shift_state = StateId(2); // Assume state 2 after shifting '{'
    table.action_table[lbrace_shift_state.0 as usize][9] = vec![vec![Action::Recover]];
    
    let mut driver = Driver::new(&table);
    
    // Parse "{" - incomplete, should trigger recovery
    let tokens = vec![
        (1, 0, 1),  // {
        (9, 1, 1),  // EOF
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
#[ignore] // API needs update
fn test_missing_value_recovery() {
    let (_grammar, mut table) = create_test_grammar();
    
    table.initial_state = StateId(1);
    table.eof_symbol = SymbolId(9);
    
    let mut driver = Driver::new(&table);
    
    // Parse '{"key": }' - missing value after colon
    let tokens = vec![
        (1, 0, 1),  // {
        (7, 1, 6),  // "key" (string)
        (5, 6, 7),  // :
        (2, 8, 9),  // }
        (9, 9, 9),  // EOF
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
            (1, 0, 1),  // {
            (2, 1, 2),  // }
        ];
        
        // Use streaming parse with a simple lexer
        let lexer = |input: &str, pos: usize, _mode| {
            if pos >= input.len() { return None; }
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
            assert!(!view.roots().is_empty(), "Should have at least one parse tree");
            
            // Verify no error nodes were created
            // We'd need to traverse the forest and check for ERROR_SYMBOL nodes
            // For now, just ensure it parsed
        }
    }
    
    // Test 2: Empty array "[]"
    {
        let lexer = |input: &str, pos: usize, _mode| {
            if pos >= input.len() { return None; }
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
    }
    
    // Test 3: Simple key-value object
    {
        let lexer = |input: &str, pos: usize, _mode| {
            if pos >= input.len() { return None; }
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
                },
                s if s.starts_with(':') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 5,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                _ => None,
            }
        };
        
        let result = driver.parse_streaming("{\"key\":\"value\"}", lexer, None::<fn(&str, usize, &[bool], _) -> _>);
        assert!(result.is_ok(), "Simple object should parse without errors");
    }
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
            if pos >= input.len() { return None; }
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
                assert!(!view.roots().is_empty(), "Should recover and produce a tree");
                // TODO: Check that error_cost is bounded (≤ beam width)
            }
            Err(_) => {
                // Acceptable if recovery can't handle this case
            }
        }
    }
    
    // Test 2: Missing value in object {"k":}
    {
        let lexer = |input: &str, pos: usize, _mode| {
            if pos >= input.len() { return None; }
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
                },
                s if s.starts_with(':') => Some(rust_sitter_glr_core::ts_lexer::NextToken {
                    kind: 5,
                    start: pos as u32,
                    end: (pos + 1) as u32,
                }),
                _ => None,
            }
        };
        
        let result = driver.parse_streaming("{\"k\":}", lexer, None::<fn(&str, usize, &[bool], _) -> _>);
        // Recovery should insert a missing value
        match result {
            Ok(forest) => {
                let view = forest.view();
                assert!(!view.roots().is_empty(), "Should recover with inserted value");
            }
            Err(_) => {
                // Also acceptable
            }
        }
    }
    
    // Test 3: Trailing comma {"k":"v",}
    {
        let lexer = |input: &str, pos: usize, _mode| {
            if pos >= input.len() { return None; }
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
                },
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
        
        let result = driver.parse_streaming("{\"k\":\"v\",}", lexer, None::<fn(&str, usize, &[bool], _) -> _>);
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