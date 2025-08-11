//! End-to-end tests for full grammar processing pipeline
//! Tests nullable start (Python-like) and non-nullable (JavaScript-like) grammars

use rust_sitter_ir::builder::GrammarBuilder;
use rust_sitter_glr_core::ParseTable;
use rust_sitter_tablegen::{TableCompressor, CompressOptions};
use rust_sitter_tablegen::helpers::{collect_token_indices, eof_accepts_or_reduces};

#[test]
fn test_python_like_nullable_start() {
    // Create a Python-like grammar with nullable start symbol
    let grammar = GrammarBuilder::python_like();
    
    // Build parse table
    let parse_table = ParseTable::from_grammar(&grammar).expect("Failed to build parse table");
    
    // Check that the start symbol is nullable
    assert!(eof_accepts_or_reduces(&parse_table), 
            "Python-like grammar should have nullable start symbol");
    
    // Compress the table
    let compressor = TableCompressor::new(&grammar, &parse_table);
    let options = CompressOptions::default();
    let compressed = compressor.compress(&options).expect("Failed to compress table");
    
    // Validate compressed table has proper structure
    assert!(!compressed.parse_table.is_empty());
    assert!(!compressed.symbols.is_empty());
    
    // Check EOF handling
    let token_indices = collect_token_indices(&grammar, &parse_table);
    assert!(token_indices.contains(&0), "EOF column should be included");
    
    // Check that we have the expected symbols
    let symbol_names: Vec<&str> = compressed.symbols.iter()
        .filter_map(|s| s.name.as_deref())
        .collect();
    
    assert!(symbol_names.contains(&"module"));
    assert!(symbol_names.contains(&"statement"));
    assert!(symbol_names.contains(&"function_def"));
}

#[test]
fn test_javascript_like_non_nullable_start() {
    // Create a JavaScript-like grammar with non-nullable start symbol
    let grammar = GrammarBuilder::javascript_like();
    
    // Build parse table
    let parse_table = ParseTable::from_grammar(&grammar).expect("Failed to build parse table");
    
    // Check that the start symbol is NOT nullable
    assert!(!eof_accepts_or_reduces(&parse_table), 
            "JavaScript-like grammar should have non-nullable start symbol");
    
    // Compress the table
    let compressor = TableCompressor::new(&grammar, &parse_table);
    let options = CompressOptions::default();
    let compressed = compressor.compress(&options).expect("Failed to compress table");
    
    // Validate compressed table has proper structure
    assert!(!compressed.parse_table.is_empty());
    assert!(!compressed.symbols.is_empty());
    
    // Check EOF handling
    let token_indices = collect_token_indices(&grammar, &parse_table);
    assert!(token_indices.contains(&0), "EOF column should be included");
    
    // Check that we have the expected symbols
    let symbol_names: Vec<&str> = compressed.symbols.iter()
        .filter_map(|s| s.name.as_deref())
        .collect();
    
    assert!(symbol_names.contains(&"program"));
    assert!(symbol_names.contains(&"statement"));
    assert!(symbol_names.contains(&"expression"));
}

#[test]
fn test_precedence_handling() {
    // Create a grammar with precedence rules
    let grammar = GrammarBuilder::new("calc")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .token("(", "(")
        .token(")", ")")
        .extra("WHITESPACE")
        .token("WHITESPACE", r"[ \t\n]+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, rust_sitter_ir::Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, rust_sitter_ir::Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, rust_sitter_ir::Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "/", "expr"], 2, rust_sitter_ir::Associativity::Left)
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    
    // Build parse table - with precedence, conflicts should be resolved
    let parse_table = ParseTable::from_grammar(&grammar).expect("Failed to build parse table");
    
    // Compress the table
    let compressor = TableCompressor::new(&grammar, &parse_table);
    let options = CompressOptions::default();
    let compressed = compressor.compress(&options).expect("Failed to compress table");
    
    // Validate the table was successfully built and compressed
    assert!(!compressed.parse_table.is_empty());
    
    // Ensure we have the expected number of symbols (terminals + non-terminals)
    let terminal_count = compressed.symbols.iter()
        .filter(|s| s.symbol_type == rust_sitter_tablegen::SymbolType::Regular)
        .count();
    assert!(terminal_count >= 6, "Should have at least 6 terminals");
}

#[test]
fn test_empty_grammar_handling() {
    // Test a minimal grammar with just an empty start rule
    let grammar = GrammarBuilder::new("empty")
        .rule("start", vec![])
        .start("start")
        .build();
    
    // Build parse table
    let parse_table = ParseTable::from_grammar(&grammar).expect("Failed to build parse table");
    
    // This should be a nullable start
    assert!(eof_accepts_or_reduces(&parse_table), 
            "Empty grammar should have nullable start");
    
    // Compress the table
    let compressor = TableCompressor::new(&grammar, &parse_table);
    let options = CompressOptions::default();
    let compressed = compressor.compress(&options).expect("Failed to compress table");
    
    // Even an empty grammar should produce some structure
    assert!(!compressed.parse_table.is_empty());
    assert!(!compressed.symbols.is_empty());
}

#[test]
fn test_recursive_grammar() {
    // Test a simple recursive grammar
    let grammar = GrammarBuilder::new("list")
        .token("ITEM", "item")
        .token(",", ",")
        .rule("list", vec!["ITEM"])
        .rule("list", vec!["list", ",", "ITEM"])
        .start("list")
        .build();
    
    // Build parse table
    let parse_table = ParseTable::from_grammar(&grammar).expect("Failed to build parse table");
    
    // This should NOT be nullable (must have at least one item)
    assert!(!eof_accepts_or_reduces(&parse_table), 
            "List grammar should have non-nullable start");
    
    // Compress the table
    let compressor = TableCompressor::new(&grammar, &parse_table);
    let options = CompressOptions::default();
    let compressed = compressor.compress(&options).expect("Failed to compress table");
    
    // Validate compression
    assert!(!compressed.parse_table.is_empty());
    assert!(compressed.symbols.len() >= 3, "Should have at least 3 symbols");
}