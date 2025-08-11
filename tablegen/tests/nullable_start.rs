// Test for nullable start symbol handling in table compression
use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets, Action};
use rust_sitter_ir::*;
use rust_sitter_tablegen::{helpers::collect_token_indices, compress::TableCompressor};

#[test]
fn nullable_start_allows_eof_accept_or_reduce() {
    // Grammar: Start -> ε | 'a'
    // This tests that a nullable start symbol is properly handled
    let mut grammar = Grammar::new("nullable_test".to_string());
    
    // Create start symbol and token
    let start = SymbolId(1);
    let tok_a = SymbolId(2);
    
    // Add token 'a'
    grammar.tokens.insert(
        tok_a,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(tok_a, "a".to_string());
    
    // Add start nonterminal name
    grammar.rule_names.insert(start, "Start".to_string());
    
    // Rule 1: Start -> ε (empty)
    grammar.rules.entry(start).or_default().push(Rule {
        lhs: start,
        rhs: vec![],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    
    // Rule 2: Start -> 'a'
    grammar.rules.entry(start).or_default().push(Rule {
        lhs: start,
        rhs: vec![Symbol::Terminal(tok_a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    
    // Build FIRST/FOLLOW sets and parse table
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    // Collect token indices using helper (includes EOF)
    let token_indices = collect_token_indices(&grammar, &parse_table);
    
    // Derive start_can_be_empty from EOF cell in state 0
    use rust_sitter_ir::SymbolId;
    let eof_idx = *parse_table.symbol_to_index.get(&SymbolId(0)).unwrap();
    let state0 = &parse_table.action_table[0];
    let start_can_be_empty = state0[eof_idx].iter().any(|a| matches!(a, Action::Accept | Action::Reduce(_)));
    
    // This should be true because Start -> ε makes it nullable
    assert!(start_can_be_empty, "Start should be nullable due to empty rule");
    
    // Compression should succeed with nullable start
    let compressed = TableCompressor::new()
        .compress(&parse_table, &token_indices, start_can_be_empty)
        .expect("compression should accept nullable Start via EOF cell");
    
    // Validate the compressed result
    assert!(compressed.validate(&parse_table).is_ok());
}

#[test]
fn non_nullable_start_has_no_eof_reduce() {
    // Grammar: Start -> 'a' | 'a' 'b'
    // This tests a non-nullable start symbol
    let mut grammar = Grammar::new("non_nullable_test".to_string());
    
    let start = SymbolId(1);
    let tok_a = SymbolId(2);
    let tok_b = SymbolId(3);
    
    // Add tokens
    grammar.tokens.insert(
        tok_a,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(tok_a, "a".to_string());
    
    grammar.tokens.insert(
        tok_b,
        Token {
            name: "b".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(tok_b, "b".to_string());
    
    // Add start nonterminal name
    grammar.rule_names.insert(start, "Start".to_string());
    
    // Rule 1: Start -> 'a'
    grammar.rules.entry(start).or_default().push(Rule {
        lhs: start,
        rhs: vec![Symbol::Terminal(tok_a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    
    // Rule 2: Start -> 'a' 'b'
    grammar.rules.entry(start).or_default().push(Rule {
        lhs: start,
        rhs: vec![Symbol::Terminal(tok_a), Symbol::Terminal(tok_b)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    let token_indices = collect_token_indices(&grammar, &parse_table);
    
    // Check EOF cell in state 0
    use rust_sitter_ir::SymbolId;
    let eof_idx = *parse_table.symbol_to_index.get(&SymbolId(0)).unwrap();
    let state0 = &parse_table.action_table[0];
    let start_can_be_empty = state0[eof_idx].iter().any(|a| matches!(a, Action::Accept | Action::Reduce(_)));
    
    // Should be false because Start requires at least 'a'
    assert!(!start_can_be_empty, "Start should not be nullable - requires at least token 'a'");
    
    // Compression should still succeed
    let compressed = TableCompressor::new()
        .compress(&parse_table, &token_indices, start_can_be_empty)
        .expect("compression should work for non-nullable Start");
    
    assert!(compressed.validate(&parse_table).is_ok());
}