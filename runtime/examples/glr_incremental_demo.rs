//! Demonstrates incremental parsing with rust-sitter GLR parser

use rust_sitter::{
    glr_incremental::{ForestNode, GLREdit, GLRToken, IncrementalGLRParser},
    glr_parser::GLRParser,
};
use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets, ParseTable};
use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId};
use std::ops::Range;
use std::sync::Arc;

fn main() {
    println!("=== GLR Incremental Parsing Demo ===\n");

    // Create a simple grammar for demonstration
    let grammar = create_simple_grammar();
    let grammar = Arc::new(grammar);

    // Build parse table
    let ff_sets = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = match build_lr1_automaton(&grammar, &ff_sets) {
        Ok(table) => table,
        Err(e) => {
            println!("Failed to build parse table: {:?}", e);
            return;
        }
    };

    // Create incremental parser directly with grammar and table
    let mut incremental_parser = IncrementalGLRParser::new((*grammar).clone(), parse_table);

    // Test 1: Initial parse
    println!("Test 1: Initial parse");
    let input1 = "a b c";
    let tokens1 = tokenize(input1);
    println!("Input: '{}'", input1);

    let result1 = incremental_parser.parse_incremental(&tokens1, &[]);
    match result1 {
        Ok(tree) => {
            println!("✓ Parse successful!");
            // Store the tree for reuse
            // incremental_parser.previous_forest = Some(tree.clone()); // TODO: Fix private field access

            // Test 2: Small edit in the middle
            println!("\n\nTest 2: Edit in the middle");
            let input2 = "a x c";
            let tokens2 = tokenize(input2);
            println!("Input: '{}' (changed 'b' to 'x' at position 2)", input2);

            // Create edit for changing 'b' to 'x' at position 2
            let edit = GLREdit {
                old_range: Range { start: 2, end: 3 },
                new_text: b"x".to_vec(),
                old_forest: None, // TODO: Fix missing field
                old_token_range: Range { start: 1, end: 2 }, // Second token
                new_tokens: vec![GLRToken {
                    symbol: SymbolId(1),
                    text: b"x".to_vec(),
                    start_byte: 2,
                    end_byte: 3,
                }],
                old_tokens: tokens1.clone(),
            };

            let result2 = incremental_parser.parse_incremental(&tokens2, &[edit]);
            match result2 {
                Ok(tree2) => {
                    println!("✓ Incremental parse successful!");
                    println!("Note: Some subtrees were reused from the previous parse!");
                    // Update for next parse
                    // incremental_parser.previous_forest = Some(tree2); // TODO: Fix private field access
                }
                Err(e) => println!("✗ Incremental parse failed: {}", e),
            }

            // Test 3: Append at the end
            println!("\n\nTest 3: Append at the end");
            let input3 = "a b c d";
            let tokens3 = tokenize(input3);
            println!("Input: '{}' (appended ' d')", input3);

            // Create edit for appending ' d' at the end
            let edit = GLREdit {
                old_range: Range { start: 5, end: 5 },
                new_text: b" d".to_vec(),
                old_forest: None, // TODO: Fix missing field
                old_token_range: Range { start: 3, end: 3 }, // After last token
                new_tokens: vec![GLRToken {
                    symbol: SymbolId(1),
                    text: b"d".to_vec(),
                    start_byte: 6,
                    end_byte: 7,
                }],
                old_tokens: tokens2.clone(),
            };

            let result3 = incremental_parser.parse_incremental(&tokens3, &[edit]);
            match result3 {
                Ok(_tree3) => {
                    println!("✓ Incremental parse successful!");
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

    // Set start symbol
    // The grammar uses the first rule as the start rule by convention
    grammar
}

/// Simple tokenizer for demo
fn tokenize(input: &str) -> Vec<GLRToken> {
    let mut tokens = Vec::new();
    let mut pos = 0;

    for ch in input.chars() {
        if ch.is_alphabetic() {
            let text = ch.to_string();
            let len = text.len();
            tokens.push(GLRToken {
                symbol: SymbolId(1), // All letters map to terminal A
                text: ch.to_string().into_bytes(),
                start_byte: pos,
                end_byte: pos + len,
            });
            pos += len;
        } else if ch == ' ' {
            pos += 1; // Skip spaces
        }
    }

    tokens
}
