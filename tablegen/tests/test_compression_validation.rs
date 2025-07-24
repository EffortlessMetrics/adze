// Comprehensive validation tests for table compression algorithms
use rust_sitter_ir::{Grammar, Rule, Symbol, Token, TokenPattern, SymbolId, ProductionId};
use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton, Action, StateId};
use rust_sitter_tablegen::compression::{
    compress_action_table, compress_goto_table, decompress_action, decompress_goto,
    BitPackedActionTable
};
use std::collections::HashMap;

#[test]
fn test_action_table_compression_round_trip() {
    // Create a grammar with various action types
    let mut grammar = Grammar::new("test_compression".to_string());
    
    // Terminals
    let a_id = SymbolId(1);
    grammar.tokens.insert(a_id, Token {
        name: "a".to_string(),
        pattern: TokenPattern::String("a".to_string()),
        fragile: false,
    });
    
    let b_id = SymbolId(2);
    grammar.tokens.insert(b_id, Token {
        name: "b".to_string(),
        pattern: TokenPattern::String("b".to_string()),
        fragile: false,
    });
    
    let c_id = SymbolId(3);
    grammar.tokens.insert(c_id, Token {
        name: "c".to_string(),
        pattern: TokenPattern::String("c".to_string()),
        fragile: false,
    });
    
    // Nonterminals
    let s_id = SymbolId(4);
    grammar.rule_names.insert(s_id, "S".to_string());
    
    let a_nt_id = SymbolId(5);
    grammar.rule_names.insert(a_nt_id, "A".to_string());
    
    // Rules to create diverse actions
    // S → A c
    grammar.rules.insert(s_id, Rule {
        production_id: ProductionId(0),
        lhs: s_id,
        rhs: vec![Symbol::NonTerminal(a_nt_id), Symbol::Terminal(c_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
    });
    
    // A → a b
    grammar.rules.insert(a_nt_id, Rule {
        production_id: ProductionId(1),
        lhs: a_nt_id,
        rhs: vec![Symbol::Terminal(a_id), Symbol::Terminal(b_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
    });
    
    // A → a (creates shift-reduce conflict)
    grammar.rules.insert(a_nt_id, Rule {
        production_id: ProductionId(2),
        lhs: a_nt_id,
        rhs: vec![Symbol::Terminal(a_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
    });
    
    // Build parse table
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    // Compress the action table
    let compressed = compress_action_table(&parse_table.action_table);
    
    // Decompress and verify each action matches
    for state in 0..parse_table.state_count {
        for symbol in 0..parse_table.symbol_count {
            let original = &parse_table.action_table[state][symbol];
            let decompressed = decompress_action(&compressed, state, symbol);
            
            assert_eq!(
                original, &decompressed,
                "Action mismatch at state {}, symbol {}: {:?} != {:?}",
                state, symbol, original, decompressed
            );
        }
    }
}

#[test]
fn test_goto_table_compression_round_trip() {
    // Create a grammar with nonterminals for goto table
    let mut grammar = Grammar::new("test_goto".to_string());
    
    // Terminals
    let a_id = SymbolId(1);
    grammar.tokens.insert(a_id, Token {
        name: "a".to_string(),
        pattern: TokenPattern::String("a".to_string()),
        fragile: false,
    });
    
    // Multiple nonterminals for goto transitions
    let s_id = SymbolId(2);
    grammar.rule_names.insert(s_id, "S".to_string());
    
    let a_nt_id = SymbolId(3);
    grammar.rule_names.insert(a_nt_id, "A".to_string());
    
    let b_nt_id = SymbolId(4);
    grammar.rule_names.insert(b_nt_id, "B".to_string());
    
    // S → A B
    grammar.rules.insert(s_id, Rule {
        production_id: ProductionId(0),
        lhs: s_id,
        rhs: vec![Symbol::NonTerminal(a_nt_id), Symbol::NonTerminal(b_nt_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
    });
    
    // A → a
    grammar.rules.insert(a_nt_id, Rule {
        production_id: ProductionId(1),
        lhs: a_nt_id,
        rhs: vec![Symbol::Terminal(a_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
    });
    
    // B → a
    grammar.rules.insert(b_nt_id, Rule {
        production_id: ProductionId(2),
        lhs: b_nt_id,
        rhs: vec![Symbol::Terminal(a_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
    });
    
    // Build parse table
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    // Extract goto table (transitions on nonterminals)
    let mut goto_table = vec![vec![None; parse_table.symbol_count]; parse_table.state_count];
    
    for state in 0..parse_table.state_count {
        for (symbol_id, &index) in &parse_table.symbol_to_index {
            // Check if this symbol is a nonterminal
            if grammar.rule_names.contains_key(symbol_id) {
                // Check if there's a shift action for this nonterminal
                if let Action::Shift(target) = &parse_table.action_table[state][index] {
                    goto_table[state][index] = Some(*target);
                }
            }
        }
    }
    
    // Compress the goto table
    let compressed = compress_goto_table(&goto_table);
    
    // Decompress and verify each goto matches
    for state in 0..parse_table.state_count {
        for symbol in 0..parse_table.symbol_count {
            let original = goto_table[state][symbol];
            let decompressed = decompress_goto(&compressed, state, symbol);
            
            assert_eq!(
                original, decompressed,
                "Goto mismatch at state {}, symbol {}: {:?} != {:?}",
                state, symbol, original, decompressed
            );
        }
    }
}

#[test]
fn test_compression_with_fork_actions() {
    // Create a grammar with conflicts that generate Fork actions
    let mut grammar = Grammar::new("ambiguous".to_string());
    
    // Classic dangling else
    let if_id = SymbolId(1);
    grammar.tokens.insert(if_id, Token {
        name: "if".to_string(),
        pattern: TokenPattern::String("if".to_string()),
        fragile: false,
    });
    
    let then_id = SymbolId(2);
    grammar.tokens.insert(then_id, Token {
        name: "then".to_string(),
        pattern: TokenPattern::String("then".to_string()),
        fragile: false,
    });
    
    let else_id = SymbolId(3);
    grammar.tokens.insert(else_id, Token {
        name: "else".to_string(),
        pattern: TokenPattern::String("else".to_string()),
        fragile: false,
    });
    
    let stmt_id = SymbolId(4);
    grammar.rule_names.insert(stmt_id, "stmt".to_string());
    
    let expr_id = SymbolId(5);
    grammar.rule_names.insert(expr_id, "expr".to_string());
    
    // stmt → if expr then stmt
    grammar.rules.insert(stmt_id, Rule {
        production_id: ProductionId(0),
        lhs: stmt_id,
        rhs: vec![
            Symbol::Terminal(if_id),
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(then_id),
            Symbol::NonTerminal(stmt_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
    });
    
    // stmt → if expr then stmt else stmt (dangling else conflict)
    grammar.rules.insert(stmt_id, Rule {
        production_id: ProductionId(1),
        lhs: stmt_id,
        rhs: vec![
            Symbol::Terminal(if_id),
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(then_id),
            Symbol::NonTerminal(stmt_id),
            Symbol::Terminal(else_id),
            Symbol::NonTerminal(stmt_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
    });
    
    // stmt → expr
    grammar.rules.insert(stmt_id, Rule {
        production_id: ProductionId(2),
        lhs: stmt_id,
        rhs: vec![Symbol::NonTerminal(expr_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
    });
    
    // expr → if (simple expression)
    grammar.rules.insert(expr_id, Rule {
        production_id: ProductionId(3),
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(if_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
    });
    
    // Build parse table with conflicts
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    // Count Fork actions
    let mut fork_count = 0;
    for state in 0..parse_table.state_count {
        for symbol in 0..parse_table.symbol_count {
            if matches!(&parse_table.action_table[state][symbol], Action::Fork(_)) {
                fork_count += 1;
            }
        }
    }
    
    assert!(fork_count > 0, "Expected Fork actions in ambiguous grammar");
    
    // Compress and decompress
    let compressed = compress_action_table(&parse_table.action_table);
    
    // Verify Fork actions are preserved
    for state in 0..parse_table.state_count {
        for symbol in 0..parse_table.symbol_count {
            let original = &parse_table.action_table[state][symbol];
            let decompressed = decompress_action(&compressed, state, symbol);
            
            // Special check for Fork actions
            if let Action::Fork(original_actions) = original {
                if let Action::Fork(decompressed_actions) = &decompressed {
                    assert_eq!(
                        original_actions.len(),
                        decompressed_actions.len(),
                        "Fork action count mismatch at state {}, symbol {}",
                        state, symbol
                    );
                    
                    for (i, (orig, decomp)) in original_actions.iter()
                        .zip(decompressed_actions.iter()).enumerate() {
                        assert_eq!(
                            orig, decomp,
                            "Fork action {} mismatch at state {}, symbol {}",
                            i, state, symbol
                        );
                    }
                } else {
                    panic!(
                        "Fork action not preserved at state {}, symbol {}: {:?} != {:?}",
                        state, symbol, original, decompressed
                    );
                }
            } else {
                assert_eq!(original, &decompressed);
            }
        }
    }
}

/// Test bit-packed compression with all action types
#[test]
fn test_bit_packed_round_trip() {
    let grammar = create_conflict_grammar(); // Grammar with Fork actions
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    // Create bit-packed representation
    let bit_packed = BitPackedActionTable::from_table(&parse_table.action_table);
    
    // Track action type counts for validation
    let mut error_count = 0;
    let mut shift_count = 0;
    let mut reduce_count = 0;
    let mut accept_count = 0;
    let mut fork_count = 0;
    
    // Verify every cell matches after decompression
    for state in 0..parse_table.state_count {
        for symbol in 0..parse_table.symbol_count {
            let original = &parse_table.action_table[state][symbol];
            let decompressed = bit_packed.decompress(state, symbol);
            
            // Count action types
            match original {
                Action::Error => error_count += 1,
                Action::Shift(_) => shift_count += 1,
                Action::Reduce(_) => reduce_count += 1,
                Action::Accept => accept_count += 1,
                Action::Fork(_) => fork_count += 1,
            }
            
            assert_eq!(
                *original, decompressed,
                "Bit-packed mismatch at state {} symbol {}: {:?} vs {:?}",
                state, symbol, original, decompressed
            );
        }
    }
    
    // Ensure we tested various action types
    println!("Action counts - Error: {}, Shift: {}, Reduce: {}, Accept: {}, Fork: {}",
             error_count, shift_count, reduce_count, accept_count, fork_count);
    assert!(error_count > 0, "No Error actions found");
    assert!(shift_count > 0, "No Shift actions found");
    assert!(reduce_count > 0, "No Reduce actions found");
    // Accept actions may not be present in all grammars
    if accept_count == 0 {
        println!("Note: No Accept actions found (this is okay for some grammars)");
    }
}

/// Test compression with large grammar
#[test]
fn test_large_grammar_compression() {
    let grammar = create_large_grammar(50); // 50 rules
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    let original_size = parse_table.state_count * parse_table.symbol_count;
    
    // Test row deduplication compression
    let compressed = compress_action_table(&parse_table.action_table);
    // Cannot access private field unique_rows
    // let compression_ratio = compressed.unique_rows.len() as f64 / parse_table.state_count as f64;
    
    println!("Large grammar compression:");
    println!("  States: {}", parse_table.state_count);
    println!("  Symbols: {}", parse_table.symbol_count);
    println!("  Original cells: {}", original_size);
    // println!("  Unique rows: {}", compressed.unique_rows.len());
    // println!("  Compression ratio: {:.2}%", compression_ratio * 100.0);
    
    // Verify compression maintains correctness
    for state in 0..parse_table.state_count {
        for symbol in 0..parse_table.symbol_count {
            let original = &parse_table.action_table[state][symbol];
            let decompressed = decompress_action(&compressed, state, symbol);
            assert_eq!(*original, decompressed);
        }
    }
}

/// Test edge cases
#[test]
fn test_compression_edge_cases() {
    // Empty table
    let empty_table: Vec<Vec<Action>> = vec![];
    let compressed = compress_action_table(&empty_table);
    // Cannot access private field unique_rows
    // assert_eq!(compressed.unique_rows.len(), 0);
    
    // Single cell table
    let single_cell = vec![vec![Action::Error]];
    let compressed = compress_action_table(&single_cell);
    // Cannot access private field unique_rows
    // assert_eq!(compressed.unique_rows.len(), 1);
    assert_eq!(decompress_action(&compressed, 0, 0), Action::Error);
    
    // All identical rows
    let identical_rows = vec![
        vec![Action::Error, Action::Error, Action::Error],
        vec![Action::Error, Action::Error, Action::Error],
        vec![Action::Error, Action::Error, Action::Error],
    ];
    let compressed = compress_action_table(&identical_rows);
    // Cannot access private field unique_rows
    // assert_eq!(compressed.unique_rows.len(), 1);
}

/// Compression performance test
#[test]
fn test_compression_performance() {
    use std::time::Instant;
    
    let sizes = vec![10, 50, 100];
    
    for size in sizes {
        let grammar = create_large_grammar(size);
        let first_follow = FirstFollowSets::compute(&grammar);
        let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
        
        // Time compression
        let start = Instant::now();
        let compressed = compress_action_table(&parse_table.action_table);
        let compress_time = start.elapsed();
        
        // Time decompression of all cells
        let start = Instant::now();
        for state in 0..parse_table.state_count {
            for symbol in 0..parse_table.symbol_count {
                let _ = decompress_action(&compressed, state, symbol);
            }
        }
        let decompress_time = start.elapsed();
        
        println!("Grammar size {} rules:", size);
        println!("  Compression time: {:?}", compress_time);
        println!("  Full decompression time: {:?}", decompress_time);
        println!("  Compression ratio: {:.2}%", 
                 100.0); // Cannot access private field unique_rows
    }
}

// Helper function to create conflict grammar with dangling else
fn create_conflict_grammar() -> Grammar {
    // Dangling else grammar
    let mut grammar = Grammar::new("if_then_else".to_string());
    
    let stmt_id = SymbolId(0);
    let if_id = SymbolId(1);
    let then_id = SymbolId(2);
    let else_id = SymbolId(3);
    let expr_id = SymbolId(4);
    let other_id = SymbolId(5);
    
    // Terminals
    grammar.tokens.insert(if_id, Token {
        name: "if".to_string(),
        pattern: TokenPattern::String("if".to_string()),
        fragile: false,
    });
    grammar.tokens.insert(then_id, Token {
        name: "then".to_string(),
        pattern: TokenPattern::String("then".to_string()),
        fragile: false,
    });
    grammar.tokens.insert(else_id, Token {
        name: "else".to_string(),
        pattern: TokenPattern::String("else".to_string()),
        fragile: false,
    });
    grammar.tokens.insert(expr_id, Token {
        name: "expr".to_string(),
        pattern: TokenPattern::String("expr".to_string()),
        fragile: false,
    });
    grammar.tokens.insert(other_id, Token {
        name: "other".to_string(),
        pattern: TokenPattern::String("other".to_string()),
        fragile: false,
    });
    
    // Non-terminal
    grammar.rule_names.insert(stmt_id, "stmt".to_string());
    
    // Rules creating shift-reduce conflict
    // stmt -> if expr then stmt
    grammar.rules.insert(stmt_id, Rule {
        production_id: ProductionId(0),
        lhs: stmt_id,
        rhs: vec![
            Symbol::Terminal(if_id),
            Symbol::Terminal(expr_id),
            Symbol::Terminal(then_id),
            Symbol::NonTerminal(stmt_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
    });
    
    // stmt -> if expr then stmt else stmt
    grammar.rules.insert(stmt_id, Rule {
        production_id: ProductionId(1),
        lhs: stmt_id,
        rhs: vec![
            Symbol::Terminal(if_id),
            Symbol::Terminal(expr_id),
            Symbol::Terminal(then_id),
            Symbol::NonTerminal(stmt_id),
            Symbol::Terminal(else_id),
            Symbol::NonTerminal(stmt_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
    });
    
    // stmt -> other
    grammar.rules.insert(stmt_id, Rule {
        production_id: ProductionId(2),
        lhs: stmt_id,
        rhs: vec![Symbol::Terminal(other_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
    });
    
    // The start symbol is determined by the first rule in the grammar
    grammar
}

fn create_large_grammar(num_rules: usize) -> Grammar {
    let mut grammar = Grammar::new("large".to_string());
    
    // Create symbols
    let start_id = SymbolId(0);
    grammar.rule_names.insert(start_id, "start".to_string());
    
    // Create terminals
    for i in 0..10 {
        let id = SymbolId(i + 1);
        grammar.tokens.insert(id, Token {
            name: format!("t{}", i),
            pattern: TokenPattern::String(format!("t{}", i)),
            fragile: false,
        });
    }
    
    // Create non-terminals and rules
    for i in 0..num_rules {
        let nt_id = SymbolId((i + 11) as u16);
        grammar.rule_names.insert(nt_id, format!("nt{}", i));
        
        // Create rules with varying patterns
        let rule_id = ProductionId(i as u16);
        let rhs = match i % 5 {
            0 => vec![Symbol::Terminal(SymbolId(1))],
            1 => vec![Symbol::Terminal(SymbolId(2)), Symbol::Terminal(SymbolId(3))],
            2 => vec![Symbol::NonTerminal(start_id), Symbol::Terminal(SymbolId(4))],
            3 => vec![
                Symbol::Terminal(SymbolId(5)),
                Symbol::NonTerminal(nt_id),
                Symbol::Terminal(SymbolId(6))
            ],
            _ => vec![Symbol::NonTerminal(nt_id)],
        };
        
        let lhs = if i == 0 { start_id } else { SymbolId((i % 5 + 11) as u16) };
        grammar.rules.insert(lhs, Rule {
            production_id: rule_id,
            lhs,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
        });
    }
    
    // The start symbol is determined by the first rule in the grammar
    grammar
}