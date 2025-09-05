//! End-to-end tests for full grammar processing pipeline
//! Tests nullable start (Python-like) and non-nullable (JavaScript-like) grammars

use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};
use rust_sitter_ir::builder::GrammarBuilder;
use rust_sitter_tablegen::helpers::{collect_token_indices, eof_accepts_or_reduces};
use rust_sitter_tablegen::TableCompressor;

#[test]
fn test_python_like_nullable_start() {
    // Create a Python-like grammar with nullable start symbol
    let grammar = GrammarBuilder::python_like();

    // Build parse table
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table =
        build_lr1_automaton(&grammar, &first_follow).expect("Failed to build parse table");

    // Check that the start symbol is nullable
    assert!(
        eof_accepts_or_reduces(&parse_table),
        "Python-like grammar should have nullable start symbol"
    );

    // Compress the table
    let compressor = TableCompressor::new();
    let token_indices = collect_token_indices(&grammar, &parse_table);
    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .expect("Failed to compress table");

    // Validate compressed table has proper structure
    assert!(!compressed.action_table.data.is_empty());
    assert!(!compressed.goto_table.data.is_empty());

    // Check EOF handling
    assert!(token_indices.contains(&0), "EOF column should be included");
}

#[test]
fn test_javascript_like_non_nullable_start() {
    // Create a JavaScript-like grammar with non-nullable start symbol
    let grammar = GrammarBuilder::javascript_like();

    // Build parse table
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table =
        build_lr1_automaton(&grammar, &first_follow).expect("Failed to build parse table");

    // Check that the start symbol is NOT nullable
    assert!(
        !eof_accepts_or_reduces(&parse_table),
        "JavaScript-like grammar should have non-nullable start symbol"
    );

    // Compress the table
    let compressor = TableCompressor::new();
    let token_indices = collect_token_indices(&grammar, &parse_table);
    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .expect("Failed to compress table");

    // Validate compressed table has proper structure
    assert!(!compressed.action_table.data.is_empty());
    assert!(!compressed.goto_table.data.is_empty());

    // Check EOF handling
    assert!(token_indices.contains(&0), "EOF column should be included");
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
        .rule_with_precedence(
            "expr",
            vec!["expr", "+", "expr"],
            1,
            rust_sitter_ir::Associativity::Left,
        )
        .rule_with_precedence(
            "expr",
            vec!["expr", "-", "expr"],
            1,
            rust_sitter_ir::Associativity::Left,
        )
        .rule_with_precedence(
            "expr",
            vec!["expr", "*", "expr"],
            2,
            rust_sitter_ir::Associativity::Left,
        )
        .rule_with_precedence(
            "expr",
            vec!["expr", "/", "expr"],
            2,
            rust_sitter_ir::Associativity::Left,
        )
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    // Build parse table - with precedence, conflicts should be resolved
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table =
        build_lr1_automaton(&grammar, &first_follow).expect("Failed to build parse table");

    // Compress the table
    let compressor = TableCompressor::new();
    let token_indices = collect_token_indices(&grammar, &parse_table);
    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .expect("Failed to compress table");

    // Validate the table was successfully built and compressed
    assert!(!compressed.action_table.data.is_empty());

    // Ensure we have the expected number of symbols (terminals + non-terminals)
    assert!(
        token_indices.len() >= 6,
        "Should have at least 6 token indices"
    );
}

#[test]
fn test_empty_grammar_handling() {
    // Test a minimal grammar with just an empty start rule
    let grammar = GrammarBuilder::new("empty")
        .rule("start", vec![])
        .start("start")
        .build();

    // Build parse table
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table =
        build_lr1_automaton(&grammar, &first_follow).expect("Failed to build parse table");

    // This should be a nullable start
    assert!(
        eof_accepts_or_reduces(&parse_table),
        "Empty grammar should have nullable start"
    );

    // Compress the table
    let compressor = TableCompressor::new();
    let token_indices = collect_token_indices(&grammar, &parse_table);
    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .expect("Failed to compress table");

    // Even an empty grammar should produce some structure
    assert!(!compressed.action_table.data.is_empty());
    assert!(!compressed.goto_table.data.is_empty());
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
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table =
        build_lr1_automaton(&grammar, &first_follow).expect("Failed to build parse table");

    // This should NOT be nullable (must have at least one item)
    assert!(
        !eof_accepts_or_reduces(&parse_table),
        "List grammar should have non-nullable start"
    );

    // Compress the table
    let compressor = TableCompressor::new();
    let token_indices = collect_token_indices(&grammar, &parse_table);
    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .expect("Failed to compress table");

    // Validate compression
    assert!(!compressed.action_table.data.is_empty());
    assert!(
        compressed.action_table.data.len() >= 3,
        "Should have at least 3 actions"
    );
}
