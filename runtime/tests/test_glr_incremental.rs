// Test incremental parsing functionality
use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};
use rust_sitter_ir::{
    Grammar, Rule, Symbol, Token, TokenPattern, SymbolId, ProductionId,
    PrecedenceEntry, PrecedenceKind, Associativity,
};
use std::sync::Arc;

// NOTE: These tests use internal modules not exported by the public API
// In a real application, you would use the public API through rust_sitter
#[path = "../src/glr_parser.rs"]
mod glr_parser;
#[path = "../src/glr_lexer.rs"]
mod glr_lexer;
#[path = "../src/glr_incremental.rs"]
mod glr_incremental;
#[path = "../src/subtree.rs"]
mod subtree;

use glr_parser::GLRParser;
use glr_lexer::{GLRLexer, TokenWithPosition};
use glr_incremental::{IncrementalGLRParser, Edit, Position};

// Create a simple arithmetic expression grammar
fn create_arithmetic_grammar() -> Arc<Grammar> {
    let mut grammar = Grammar::new("arithmetic".to_string());
    
    // Define terminals
    let number_id = SymbolId(0);
    grammar.tokens.insert(number_id, Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    });
    grammar.rule_names.insert(number_id, "number".to_string());
    
    let plus_id = SymbolId(1);
    grammar.tokens.insert(plus_id, Token {
        name: "plus".to_string(),
        pattern: TokenPattern::String("+".to_string()),
        fragile: false,
    });
    grammar.rule_names.insert(plus_id, "plus".to_string());
    
    let times_id = SymbolId(2);
    grammar.tokens.insert(times_id, Token {
        name: "times".to_string(),
        pattern: TokenPattern::String("*".to_string()),
        fragile: false,
    });
    grammar.rule_names.insert(times_id, "times".to_string());
    
    let lparen_id = SymbolId(3);
    grammar.tokens.insert(lparen_id, Token {
        name: "lparen".to_string(),
        pattern: TokenPattern::String("(".to_string()),
        fragile: false,
    });
    grammar.rule_names.insert(lparen_id, "lparen".to_string());
    
    let rparen_id = SymbolId(4);
    grammar.tokens.insert(rparen_id, Token {
        name: "rparen".to_string(),
        pattern: TokenPattern::String(")".to_string()),
        fragile: false,
    });
    grammar.rule_names.insert(rparen_id, "rparen".to_string());
    
    // Define non-terminals
    let expr_id = SymbolId(5);
    let term_id = SymbolId(6);
    let factor_id = SymbolId(7);
    
    grammar.rule_names.insert(expr_id, "expr".to_string());
    grammar.rule_names.insert(term_id, "term".to_string());
    grammar.rule_names.insert(factor_id, "factor".to_string());
    
    // Set the start symbol
    let start_id = SymbolId(8);
    grammar.rule_names.insert(start_id, "_start".to_string());
    grammar.rules.insert(start_id, vec![
        Rule {
            lhs: start_id,
            rhs: vec![Symbol::NonTerminal(expr_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(6),
            fields: vec![],
        },
    ]);
    
    // Define rules
    // expr → expr + term
    grammar.rules.insert(expr_id, vec![
        Rule {
            lhs: expr_id,
            rhs: vec![Symbol::NonTerminal(expr_id), Symbol::Terminal(plus_id), Symbol::NonTerminal(term_id)],
            precedence: Some(PrecedenceKind::Int(1)),
            associativity: Some(Associativity::Left),
            production_id: ProductionId(0),
            fields: vec![],
        },
        // expr → term
        Rule {
            lhs: expr_id,
            rhs: vec![Symbol::NonTerminal(term_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(1),
            fields: vec![],
        },
    ]);
    
    // term → term * factor, term → factor
    grammar.rules.insert(term_id, vec![
        Rule {
            lhs: term_id,
            rhs: vec![Symbol::NonTerminal(term_id), Symbol::Terminal(times_id), Symbol::NonTerminal(factor_id)],
            precedence: Some(PrecedenceKind::Int(2)),
            associativity: Some(Associativity::Left),
            production_id: ProductionId(2),
            fields: vec![],
        },
        Rule {
            lhs: term_id,
            rhs: vec![Symbol::NonTerminal(factor_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(3),
            fields: vec![],
        },
    ]);
    
    // factor → ( expr ), factor → number
    grammar.rules.insert(factor_id, vec![
        Rule {
            lhs: factor_id,
            rhs: vec![Symbol::Terminal(lparen_id), Symbol::NonTerminal(expr_id), Symbol::Terminal(rparen_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(4),
            fields: vec![],
        },
        Rule {
            lhs: factor_id,
            rhs: vec![Symbol::Terminal(number_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(5),
            fields: vec![],
        },
    ]);
    
    Arc::new(grammar)
}

#[test]
fn test_incremental_simple_edit() {
    let grammar = create_arithmetic_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    
    match build_lr1_automaton(&grammar, &first_follow) {
        Ok(parse_table) => {
            let parser = GLRParser::new(parse_table, (*grammar).clone());
            let mut incremental = IncrementalGLRParser::new(parser, grammar.clone());
            
            // Parse initial expression: "1 + 2"
            let mut lexer = GLRLexer::new(&grammar, "1 + 2".to_string()).unwrap();
            let tokens = lexer.tokenize_all();
            let initial_tree = incremental.parse_incremental(&tokens, &[], None).unwrap();
            
            // Verify initial parse
            assert_eq!(incremental.stats().subtrees_reused, 0);
            
            // Edit: "1 + 2" → "1 + 3"
            let edit = Edit::new(4, 5, 5);
            let mut new_lexer = GLRLexer::new(&grammar, "1 + 3".to_string()).unwrap();
            let new_tokens = new_lexer.tokenize_all();
            let edited_tree = incremental.parse_incremental(&new_tokens, &[edit], Some(initial_tree)).unwrap();
            
            // Should reuse the "1 +" part
            assert!(incremental.stats().subtrees_reused > 0);
            assert!(incremental.stats().bytes_reused > 0);
            
            // Verify the tree is still valid
            assert_eq!(edited_tree.node.byte_range.start, 0);
            assert_eq!(edited_tree.node.byte_range.end, 5);
        }
        Err(e) => panic!("Failed to build parse table: {:?}", e),
    }
}

#[test]
fn test_incremental_complex_edit() {
    let grammar = create_arithmetic_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    
    match build_lr1_automaton(&grammar, &first_follow) {
        Ok(parse_table) => {
            let parser = GLRParser::new(parse_table, (*grammar).clone());
            let mut incremental = IncrementalGLRParser::new(parser, grammar.clone());
            
            // Parse initial expression: "(1 + 2) * 3"
            let mut lexer = GLRLexer::new(&grammar, "(1 + 2) * 3".to_string()).unwrap();
            let tokens = lexer.tokenize_all();
            let initial_tree = incremental.parse_incremental(&tokens, &[], None).unwrap();
            
            // Edit: "(1 + 2) * 3" → "(1 + 2) * 4"
            let edit = Edit::new(10, 11, 11);
            let mut new_lexer = GLRLexer::new(&grammar, "(1 + 2) * 4".to_string()).unwrap();
            let new_tokens = new_lexer.tokenize_all();
            let _edited_tree = incremental.parse_incremental(&new_tokens, &[edit], Some(initial_tree)).unwrap();
            
            // Should reuse the entire "(1 + 2)" subtree
            assert!(incremental.stats().subtrees_reused > 0);
            assert!(incremental.stats().bytes_reused >= 7); // At least "(1 + 2)"
        }
        Err(e) => panic!("Failed to build parse table: {:?}", e),
    }
}

#[test]
fn test_incremental_multiple_edits() {
    let grammar = create_arithmetic_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    
    match build_lr1_automaton(&grammar, &first_follow) {
        Ok(parse_table) => {
            let parser = GLRParser::new(parse_table, (*grammar).clone());
            let mut incremental = IncrementalGLRParser::new(parser, grammar.clone());
            
            // Parse initial expression: "1 + 2 + 3 + 4"
            let mut lexer = GLRLexer::new(&grammar, "1 + 2 + 3 + 4".to_string()).unwrap();
            let tokens = lexer.tokenize_all();
            let initial_tree = incremental.parse_incremental(&tokens, &[], None).unwrap();
            
            // Multiple edits: "1 + 2 + 3 + 4" → "9 + 2 + 3 + 8"
            let edits = vec![
                Edit::new(0, 1, 1),  // 1 → 9
                Edit::new(12, 13, 13), // 4 → 8
            ];
            let mut new_lexer = GLRLexer::new(&grammar, "9 + 2 + 3 + 8".to_string()).unwrap();
            let new_tokens = new_lexer.tokenize_all();
            let _edited_tree = incremental.parse_incremental(&new_tokens, &edits, Some(initial_tree)).unwrap();
            
            // Should reuse the middle "2 + 3" part
            assert!(incremental.stats().subtrees_reused > 0);
            
            // Test reuse percentage
            let reuse_percentage = (incremental.stats().bytes_reused as f64 / incremental.stats().total_bytes as f64) * 100.0;
            println!("Reuse percentage: {:.1}%", reuse_percentage);
            assert!(reuse_percentage > 30.0); // Should reuse a significant portion
        }
        Err(e) => panic!("Failed to build parse table: {:?}", e),
    }
}

#[test]
fn test_incremental_insertion() {
    let grammar = create_arithmetic_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    
    match build_lr1_automaton(&grammar, &first_follow) {
        Ok(parse_table) => {
            let parser = GLRParser::new(parse_table, (*grammar).clone());
            let mut incremental = IncrementalGLRParser::new(parser, grammar.clone());
            
            // Parse initial expression: "1 + 2"
            let mut lexer = GLRLexer::new(&grammar, "1 + 2".to_string()).unwrap();
            let tokens = lexer.tokenize_all();
            let initial_tree = incremental.parse_incremental(&tokens, &[], None).unwrap();
            
            // Insert: "1 + 2" → "1 + 2 * 3"
            let edit = Edit::new(5, 5, 9); // Insert " * 3"
            let mut new_lexer = GLRLexer::new(&grammar, "1 + 2 * 3".to_string()).unwrap();
            let new_tokens = new_lexer.tokenize_all();
            let edited_tree = incremental.parse_incremental(&new_tokens, &[edit], Some(initial_tree)).unwrap();
            
            // Should reuse at least the "1" part
            assert!(incremental.stats().subtrees_reused > 0);
            
            // Clear pool and test deletion
            incremental.clear_pool();
            
            // Delete: "1 + 2 * 3" → "1 + 2"
            let edit = Edit::new(5, 9, 5); // Delete " * 3"
            let mut final_lexer = GLRLexer::new(&grammar, "1 + 2".to_string()).unwrap();
            let new_tokens = final_lexer.tokenize_all();
            let _final_tree = incremental.parse_incremental(&new_tokens, &[edit], Some(edited_tree)).unwrap();
            
            assert!(incremental.stats().subtrees_reused > 0);
        }
        Err(e) => panic!("Failed to build parse table: {:?}", e),
    }
}