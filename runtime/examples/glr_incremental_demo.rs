//! Demonstrates incremental parsing with rust-sitter GLR parser

use rust_sitter::{
    glr_incremental::{Edit, IncrementalGLRParser, Position},
    glr_parser::GLRParser,
    glr_lexer::TokenWithPosition,
};
use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets, ParseTable};
use rust_sitter_ir::{Grammar, Rule, Symbol, SymbolId, ProductionId};
use std::sync::Arc;

fn main() {
    println!("=== GLR Incremental Parsing Demo ===\n");
    
    // Create a simple grammar for demonstration
    let grammar = create_simple_grammar();
    let grammar = Arc::new(grammar);
    
    // Build parse table
    let ff_sets = FirstFollowSets::compute(&grammar);
    let parse_table = match build_lr1_automaton(&grammar, &ff_sets) {
        Ok(table) => table,
        Err(e) => {
            println!("Failed to build parse table: {:?}", e);
            return;
        }
    };
    
    // Create GLR parser - need to pass grammar too
    let glr_parser = GLRParser::new(parse_table, (*grammar).clone());
    
    // Create incremental parser
    let mut incremental_parser = IncrementalGLRParser::new(glr_parser, grammar.clone());
    
    // Test 1: Initial parse
    println!("Test 1: Initial parse");
    let input1 = "a b c";
    let tokens1 = tokenize(input1);
    println!("Input: '{}'", input1);
    
    let result1 = incremental_parser.parse_incremental(&tokens1, &[], None);
    match result1 {
        Ok(tree) => {
            println!("✓ Parse successful!");
            let stats = incremental_parser.stats();
            print_stats(&stats);
            
            // Test 2: Small edit in the middle
            println!("\n\nTest 2: Edit in the middle");
            let input2 = "a x c";
            let tokens2 = tokenize(input2);
            println!("Input: '{}' (changed 'b' to 'x' at position 2)", input2);
            
            let edit = Edit {
                start_byte: 2,
                old_end_byte: 3,
                new_end_byte: 3,
                start_position: Position { line: 0, column: 2 },
                old_end_position: Position { line: 0, column: 3 },
                new_end_position: Position { line: 0, column: 3 },
            };
            
            let result2 = incremental_parser.parse_incremental(&tokens2, &[edit], Some(tree.clone()));
            match result2 {
                Ok(_tree2) => {
                    println!("✓ Incremental parse successful!");
                    let stats = incremental_parser.stats();
                    print_stats(&stats);
                    println!("Note: Some subtrees were reused from the previous parse!");
                }
                Err(e) => println!("✗ Incremental parse failed: {}", e),
            }
            
            // Test 3: Append at the end
            println!("\n\nTest 3: Append at the end");
            let input3 = "a b c d";
            let tokens3 = tokenize(input3);
            println!("Input: '{}' (appended ' d')", input3);
            
            let edit = Edit {
                start_byte: 5,
                old_end_byte: 5,
                new_end_byte: 7,
                start_position: Position { line: 0, column: 5 },
                old_end_position: Position { line: 0, column: 5 },
                new_end_position: Position { line: 0, column: 7 },
            };
            
            let result3 = incremental_parser.parse_incremental(&tokens3, &[edit], Some(tree));
            match result3 {
                Ok(_tree3) => {
                    println!("✓ Incremental parse successful!");
                    let stats = incremental_parser.stats();
                    print_stats(&stats);
                    println!("Note: The beginning of the tree was reused!");
                }
                Err(e) => println!("✗ Incremental parse failed: {}", e),
            }
        }
        Err(e) => println!("✗ Initial parse failed: {}", e),
    }
    
    println!("\n=== Key Benefits of Incremental Parsing ===");
    println!("1. Small edits reuse most of the parse tree");
    println!("2. Performance scales with edit size, not document size");
    println!("3. Essential for real-time IDE features");
    println!("4. Enables efficient parsing of large files");
}

/// Create a simple grammar: S -> A+
fn create_simple_grammar() -> Grammar {
    let mut grammar = Grammar::new("simple".to_string());
    
    // Define symbol IDs
    let s_sym = SymbolId(0); // Start symbol
    let a_sym = SymbolId(1); // Terminal 'a', 'b', 'c', etc.
    
    // Add rule: S -> A
    grammar.add_rule(Rule {
        lhs: s_sym,
        rhs: vec![Symbol::Terminal(a_sym)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
        fields: Default::default(),
    });
    
    // Add rule: S -> S A (for repetition)
    grammar.add_rule(Rule {
        lhs: s_sym,
        rhs: vec![Symbol::NonTerminal(s_sym), Symbol::Terminal(a_sym)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(1),
        fields: Default::default(),
    });
    
    // Set start symbol by adding initial rule
    grammar.set_start_symbol(s_sym);
    grammar
}

/// Simple tokenizer for demo
fn tokenize(input: &str) -> Vec<TokenWithPosition> {
    let mut tokens = Vec::new();
    let mut pos = 0;
    
    for ch in input.chars() {
        if ch.is_alphabetic() {
            let text = ch.to_string();
            let len = text.len();
            tokens.push(TokenWithPosition {
                symbol_id: SymbolId(1), // All letters map to terminal A
                text,
                byte_offset: pos,
                byte_length: len,
            });
            pos += len;
        } else if ch == ' ' {
            pos += 1; // Skip spaces
        }
    }
    
    tokens
}

/// Print reuse statistics
fn print_stats(stats: &rust_sitter::glr_incremental::ReuseStats) {
    println!("  Subtrees reused: {}", stats.subtrees_reused);
    println!("  Bytes reused: {} / {} ({:.1}%)", 
        stats.bytes_reused, 
        stats.total_bytes,
        if stats.total_bytes > 0 {
            (stats.bytes_reused as f64 / stats.total_bytes as f64) * 100.0
        } else {
            0.0
        }
    );
}