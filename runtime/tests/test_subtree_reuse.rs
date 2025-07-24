// Test subtree reuse in incremental parsing
use rust_sitter_ir::{Grammar, Rule, Symbol, Token, TokenPattern, SymbolId, ProductionId};
use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};

// Import internal modules for testing
#[path = "../src/subtree.rs"]
mod subtree;
#[path = "../src/glr_lexer.rs"]
mod glr_lexer;
#[path = "../src/glr_parser.rs"]
mod glr_parser;
#[path = "../src/glr_incremental.rs"]
mod glr_incremental;

use glr_incremental::{IncrementalGLRParser, Edit, Position};

fn create_simple_grammar() -> Grammar {
    let mut grammar = Grammar::new("simple".to_string());
    
    // Terminals
    let a_id = SymbolId(1);
    let b_id = SymbolId(2);
    let c_id = SymbolId(3);
    
    grammar.tokens.insert(a_id, Token {
        name: "a".to_string(),
        pattern: TokenPattern::String("a".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(b_id, Token {
        name: "b".to_string(),
        pattern: TokenPattern::String("b".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(c_id, Token {
        name: "c".to_string(),
        pattern: TokenPattern::String("c".to_string()),
        fragile: false,
    });
    
    // Non-terminals
    let s_id = SymbolId(10);
    let a_id_nt = SymbolId(11);
    
    grammar.rule_names.insert(s_id, "S".to_string());
    grammar.rule_names.insert(a_id_nt, "A".to_string());
    
    // Rules
    // S -> A A A
    grammar.rules.insert(SymbolId(20), Rule {
        lhs: s_id,
        rhs: vec![Symbol::NonTerminal(a_id_nt), Symbol::NonTerminal(a_id_nt), Symbol::NonTerminal(a_id_nt)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
        fields: vec![],
    });
    
    // A -> a | b | c
    grammar.rules.insert(SymbolId(21), Rule {
        lhs: a_id_nt,
        rhs: vec![Symbol::Terminal(a_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(1),
        fields: vec![],
    });
    
    grammar.rules.insert(SymbolId(22), Rule {
        lhs: a_id_nt,
        rhs: vec![Symbol::Terminal(b_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(2),
        fields: vec![],
    });
    
    grammar.rules.insert(SymbolId(23), Rule {
        lhs: a_id_nt,
        rhs: vec![Symbol::Terminal(c_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(3),
        fields: vec![],
    });
    
    grammar
}

#[test]
fn test_subtree_reuse_basic() {
    let grammar = create_simple_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    let mut parser = GLRIncrementalParser::new(parse_table, grammar.clone());
    
    // Initial parse: "abc"
    let tree1 = parser.parse("abc").unwrap();
    println!("Initial parse complete");
    
    // Apply edit: change middle 'b' to 'a' -> "aac"
    parser.apply_edit(&Edit {
        start_byte: 1,
        old_end_byte: 2,
        new_end_byte: 2,
        start_position: Position { row: 0, column: 1 },
        old_end_position: Position { row: 0, column: 2 },
        new_end_position: Position { row: 0, column: 2 },
    });
    
    let tree2 = parser.parse("aac").unwrap();
    println!("Second parse complete");
    
    // Check statistics
    let stats = parser.get_stats();
    println!("Parse stats: {:?}", stats);
    
    // We should have reused at least one subtree (the first 'a' and/or last 'c')
    assert!(stats.subtrees_reused > 0, "Expected some subtree reuse, but got none");
    assert!(stats.bytes_reused > 0, "Expected some bytes reused, but got none");
    
    // The trees should be different
    assert_ne!(tree1.node.byte_range, tree2.node.byte_range);
}

#[test]
fn test_subtree_reuse_multiple_edits() {
    let grammar = create_simple_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    let mut parser = GLRIncrementalParser::new(parse_table, grammar.clone());
    
    // Initial parse: "aaa"
    let _tree1 = parser.parse("aaa").unwrap();
    parser.reset_stats();
    
    // Apply edit: change first 'a' to 'b' -> "baa"
    parser.apply_edit(&Edit {
        start_byte: 0,
        old_end_byte: 1,
        new_end_byte: 1,
        start_position: Position { row: 0, column: 0 },
        old_end_position: Position { row: 0, column: 1 },
        new_end_position: Position { row: 0, column: 1 },
    });
    
    let _tree2 = parser.parse("baa").unwrap();
    
    let stats = parser.get_stats();
    println!("Stats after first edit: {:?}", stats);
    
    // We should reuse the last two 'a' subtrees
    assert!(stats.subtrees_reused >= 1, "Expected at least 1 subtree reused, got {}", stats.subtrees_reused);
    assert!(stats.bytes_reused >= 2, "Expected at least 2 bytes reused, got {}", stats.bytes_reused);
}