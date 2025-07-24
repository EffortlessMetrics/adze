// Simple integration test for GLR parser
// This demonstrates basic GLR parsing functionality

use rust_sitter_ir::{Grammar, Rule, Symbol, Token, TokenPattern, SymbolId, ProductionId};
use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
use std::sync::Arc;

// Import internal modules for testing
#[path = "../src/subtree.rs"]
mod subtree;
#[path = "../src/glr_lexer.rs"]
mod glr_lexer;
#[path = "../src/glr_parser.rs"]
mod glr_parser;

use glr_lexer::GLRLexer;
use glr_parser::GLRParser;
use subtree::Subtree;

/// Create a simple number grammar for testing
fn create_number_grammar() -> Grammar {
    let mut grammar = Grammar::new("number".to_string());
    
    // Define terminals
    let number_id = SymbolId(0);
    grammar.tokens.insert(number_id, Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    });
    
    let plus_id = SymbolId(1);
    grammar.tokens.insert(plus_id, Token {
        name: "plus".to_string(),
        pattern: TokenPattern::String("+".to_string()),
        fragile: false,
    });
    
    // Define non-terminal
    let expr_id = SymbolId(10);
    grammar.rule_names.insert(expr_id, "expression".to_string());
    
    // Define rules
    // Rule 1: expression → number
    let number_rule_id = SymbolId(20);
    grammar.rules.insert(number_rule_id, Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(number_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
        fields: vec![],
    });
    
    // Rule 2: expression → expression + expression
    let add_rule_id = SymbolId(21);
    grammar.rules.insert(add_rule_id, Rule {
        lhs: expr_id,
        rhs: vec![Symbol::NonTerminal(expr_id), Symbol::Terminal(plus_id), Symbol::NonTerminal(expr_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(1),
        fields: vec![],
    });
    
    grammar
}

#[test]
fn test_simple_number_parsing() {
    let grammar = create_number_grammar();
    
    // Build parse table
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    let mut parser = GLRParser::new(parse_table, grammar.clone());
    
    // Test 1: Parse a single number
    {
        parser.reset();
        let input = "42";
        let mut lexer = GLRLexer::new(&grammar, input.to_string()).unwrap();
        let tokens = lexer.tokenize_all();
        
        println!("Test 1 - Parsing '{}', tokens: {:?}", input, 
            tokens.iter().map(|t| (t.symbol_id, &t.text)).collect::<Vec<_>>());
        
        for token in &tokens {
            parser.process_token(token.symbol_id, &token.text, token.byte_offset);
        }
        parser.process_eof();
        
        let result = parser.finish();
        assert!(result.is_ok(), "Failed to parse single number");
        println!("✓ Successfully parsed single number");
    }
    
    // Test 2: Parse addition
    {
        parser.reset();
        let input = "1+2";
        let mut lexer = GLRLexer::new(&grammar, input.to_string()).unwrap();
        let tokens = lexer.tokenize_all();
        
        println!("\nTest 2 - Parsing '{}', tokens: {:?}", input,
            tokens.iter().map(|t| (t.symbol_id, &t.text)).collect::<Vec<_>>());
        
        for token in &tokens {
            parser.process_token(token.symbol_id, &token.text, token.byte_offset);
        }
        parser.process_eof();
        
        let result = parser.finish();
        assert!(result.is_ok(), "Failed to parse addition");
        println!("✓ Successfully parsed addition");
    }
    
    // Test 3: Parse chained addition
    {
        parser.reset();
        let input = "1+2+3";
        let mut lexer = GLRLexer::new(&grammar, input.to_string()).unwrap();
        let tokens = lexer.tokenize_all();
        
        println!("\nTest 3 - Parsing '{}', tokens: {:?}", input,
            tokens.iter().map(|t| (t.symbol_id, &t.text)).collect::<Vec<_>>());
        
        for token in &tokens {
            parser.process_token(token.symbol_id, &token.text, token.byte_offset);
        }
        parser.process_eof();
        
        let result = parser.finish();
        assert!(result.is_ok(), "Failed to parse chained addition");
        println!("✓ Successfully parsed chained addition");
        
        // Should have handled ambiguity (left vs right associative)
        let stack_count = parser.stack_count();
        println!("  Parser maintained {} stack(s) during parsing", stack_count);
    }
}

#[test]
fn test_glr_ambiguity() {
    // Create a truly ambiguous grammar
    let mut grammar = Grammar::new("ambiguous".to_string());
    
    // Terminal 'a'
    let a_id = SymbolId(0);
    grammar.tokens.insert(a_id, Token {
        name: "a".to_string(),
        pattern: TokenPattern::String("a".to_string()),
        fragile: false,
    });
    
    // Non-terminal E
    let e_id = SymbolId(10);
    grammar.rule_names.insert(e_id, "E".to_string());
    
    // Rule 1: E → a
    let terminal_rule_id = SymbolId(20);
    grammar.rules.insert(terminal_rule_id, Rule {
        lhs: e_id,
        rhs: vec![Symbol::Terminal(a_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
        fields: vec![],
    });
    
    // Rule 2: E → E E (ambiguous concatenation)
    let concat_rule_id = SymbolId(21);
    grammar.rules.insert(concat_rule_id, Rule {
        lhs: e_id,
        rhs: vec![Symbol::NonTerminal(e_id), Symbol::NonTerminal(e_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(1),
        fields: vec![],
    });
    
    // Build parse table
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    let mut parser = GLRParser::new(parse_table, grammar.clone());
    
    // Parse "aaa" - highly ambiguous
    let input = "aaa";
    let mut lexer = GLRLexer::new(&grammar, input.to_string()).unwrap();
    let tokens = lexer.tokenize_all();
    
    println!("\nTesting ambiguous grammar with input '{}'", input);
    
    for token in &tokens {
        parser.process_token(token.symbol_id, &token.text, token.byte_offset);
        println!("  After token '{}': {} active stacks", token.text, parser.stack_count());
    }
    parser.process_eof();
    
    let result = parser.finish();
    assert!(result.is_ok(), "GLR parser should handle ambiguous grammar");
    
    println!("✓ GLR parser successfully handled ambiguous grammar");
    println!("  Final stack count: {}", parser.stack_count());
}

#[test]
fn test_glr_error_handling() {
    let grammar = create_number_grammar();
    
    // Build parse table
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    let mut parser = GLRParser::new(parse_table, grammar.clone());
    
    // Test invalid input: "1 + +"
    let input = "1++";
    let mut lexer = GLRLexer::new(&grammar, input.to_string()).unwrap();
    let tokens = lexer.tokenize_all();
    
    println!("\nTesting error handling with invalid input '{}'", input);
    println!("Tokens: {:?}", tokens.iter().map(|t| (t.symbol_id, &t.text)).collect::<Vec<_>>());
    
    for token in &tokens {
        parser.process_token(token.symbol_id, &token.text, token.byte_offset);
    }
    parser.process_eof();
    
    let result = parser.finish();
    
    // The parser might handle this through error recovery or reject it
    match result {
        Ok(_) => println!("Parser recovered from error"),
        Err(e) => println!("Parser correctly rejected invalid input: {}", e),
    }
}