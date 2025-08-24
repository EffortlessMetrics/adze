// Test GLR fork/merge functionality
use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};
use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use std::sync::Arc;

// Import internal modules for testing
#[path = "../src/error_recovery.rs"]
mod error_recovery;
#[path = "../src/glr_lexer.rs"]
mod glr_lexer;
#[path = "../src/glr_parser.rs"]
mod glr_parser;
#[path = "../src/subtree.rs"]
mod subtree;

use glr_lexer::GLRLexer;
use glr_parser::GLRParser;

/// Create an ambiguous grammar: E → E E | a
/// This grammar is ambiguous for strings like "aaa"
fn create_ambiguous_grammar() -> Grammar {
    let mut grammar = Grammar::new("ambiguous".to_string());

    // Terminal 'a'
    let a_id = SymbolId(1);
    grammar.tokens.insert(
        a_id,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );

    // Non-terminal E
    let e_id = SymbolId(10);
    grammar.rule_names.insert(e_id, "E".to_string());

    // Rule 1: E → a
    grammar.rules.entry(e_id).or_default().push(Rule {
        lhs: e_id,
        rhs: vec![Symbol::Terminal(a_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
        fields: vec![],
    });

    // Rule 2: E → E E (ambiguous concatenation)
    grammar.rules.entry(e_id).or_default().push(Rule {
        lhs: e_id,
        rhs: vec![Symbol::NonTerminal(e_id), Symbol::NonTerminal(e_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(1),
        fields: vec![],
    });

    grammar
}

/// Create ambiguous arithmetic grammar with precedence
fn create_arithmetic_grammar() -> Grammar {
    let mut grammar = Grammar::new("arithmetic".to_string());

    // Terminals
    let num_id = SymbolId(1);
    let plus_id = SymbolId(2);
    let times_id = SymbolId(3);

    grammar.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        plus_id,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        times_id,
        Token {
            name: "times".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );

    // Non-terminal E
    let e_id = SymbolId(10);
    grammar.rule_names.insert(e_id, "E".to_string());

    // Rules with precedence
    // E → E + E (lower precedence)
    grammar.rules.entry(e_id).or_default().push(Rule {
        lhs: e_id,
        rhs: vec![
            Symbol::NonTerminal(e_id),
            Symbol::Terminal(plus_id),
            Symbol::NonTerminal(e_id),
        ],
        precedence: Some(rust_sitter_ir::PrecedenceKind::Static(1)),
        associativity: Some(rust_sitter_ir::Associativity::Left),
        production_id: ProductionId(0),
        fields: vec![],
    });

    // E → E * E (higher precedence)
    grammar.rules.entry(e_id).or_default().push(Rule {
        lhs: e_id,
        rhs: vec![
            Symbol::NonTerminal(e_id),
            Symbol::Terminal(times_id),
            Symbol::NonTerminal(e_id),
        ],
        precedence: Some(rust_sitter_ir::PrecedenceKind::Static(2)),
        associativity: Some(rust_sitter_ir::Associativity::Left),
        production_id: ProductionId(1),
        fields: vec![],
    });

    // E → number
    grammar.rules.entry(e_id).or_default().push(Rule {
        lhs: e_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(2),
        fields: vec![],
    });

    grammar
}

#[test]
fn test_simple_fork_merge() {
    let grammar = create_ambiguous_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    let mut parser = GLRParser::new(parse_table, grammar.clone());

    // Parse "aaa" - should create forks (ambiguity surfaces at length >= 3 in LR(1))
    let input = "aaa";
    let mut lexer = GLRLexer::new(&grammar, input.to_string()).unwrap();
    let tokens = lexer.tokenize_all();

    println!("\n=== Testing fork/merge with input '{}' ===", input);

    let mut stack_counts = Vec::new();
    for token in &tokens {
        parser.process_token(token.symbol_id, &token.text, token.byte_offset);
        let count = parser.stack_count();
        stack_counts.push(count);
        println!("After token '{}': {} active stacks", token.text, count);
    }

    parser.process_eof(input.len());
    println!("After EOF: {} active stacks", parser.stack_count());

    let result = parser.finish();
    assert!(result.is_ok(), "Parser should handle ambiguous input");

    // We should have seen multiple stacks during parsing
    assert!(
        stack_counts.iter().any(|&c| c > 1),
        "Expected multiple stacks during parsing of ambiguous input, but got {:?}",
        stack_counts
    );

    println!("✓ Fork/merge working correctly");
}

#[test]
fn test_complex_ambiguity() {
    let grammar = create_ambiguous_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    let mut parser = GLRParser::new(parse_table, grammar.clone());

    // Parse "aaaa" - highly ambiguous
    let input = "aaaa";
    let mut lexer = GLRLexer::new(&grammar, input.to_string()).unwrap();
    let tokens = lexer.tokenize_all();

    println!("\n=== Testing complex ambiguity with input '{}' ===", input);

    let mut max_stacks = 0;
    for token in &tokens {
        parser.process_token(token.symbol_id, &token.text, token.byte_offset);
        let count = parser.stack_count();
        max_stacks = max_stacks.max(count);
        println!("After token '{}': {} active stacks", token.text, count);
    }

    parser.process_eof(input.len());
    let final_count = parser.stack_count();
    println!("After EOF: {} active stacks", final_count);
    println!("Maximum stacks during parsing: {}", max_stacks);

    let result = parser.finish();
    assert!(
        result.is_ok(),
        "Parser should handle highly ambiguous input"
    );

    // For "aaaa", we expect many possible parse trees
    assert!(
        max_stacks > 2,
        "Expected many stacks for highly ambiguous input, but got max {}",
        max_stacks
    );

    println!("✓ Complex ambiguity handled correctly");
}

#[test]
fn test_precedence_disambiguation() {
    let grammar = create_arithmetic_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    let mut parser = GLRParser::new(parse_table, grammar.clone());

    // Parse "1+2*3" - should disambiguate using precedence
    let input = "1+2*3";
    let mut lexer = GLRLexer::new(&grammar, input.to_string()).unwrap();
    let tokens = lexer.tokenize_all();

    println!(
        "\n=== Testing precedence disambiguation with input '{}' ===",
        input
    );

    for token in &tokens {
        parser.process_token(token.symbol_id, &token.text, token.byte_offset);
        println!(
            "After token '{}': {} active stacks",
            token.text,
            parser.stack_count()
        );
    }

    parser.process_eof(input.len());

    let result = parser.finish();
    assert!(result.is_ok(), "Parser should handle arithmetic expression");

    let tree = result.unwrap();

    // Verify the parse tree structure
    // With correct precedence, should parse as 1+(2*3), not (1+2)*3
    fn find_operator(tree: &Arc<subtree::Subtree>) -> Option<SymbolId> {
        if tree.children.len() == 3 {
            // Middle child should be operator
            Some(tree.children[1].subtree.node.symbol_id)
        } else {
            None
        }
    }

    // The root should be addition (lower precedence)
    let root_op = find_operator(&tree);
    assert_eq!(
        root_op,
        Some(SymbolId(2)),
        "Root should be addition operator"
    );

    println!("✓ Precedence disambiguation working correctly");
}

#[test]
fn test_merge_identical_stacks() {
    let grammar = create_ambiguous_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    let mut parser = GLRParser::new(parse_table, grammar.clone());

    // Parse "aaa" and track stack counts
    let input = "aaa";
    let mut lexer = GLRLexer::new(&grammar, input.to_string()).unwrap();
    let tokens = lexer.tokenize_all();

    println!("\n=== Testing stack merging with input '{}' ===", input);

    let mut stack_history = Vec::new();
    for token in &tokens {
        parser.process_token(token.symbol_id, &token.text, token.byte_offset);
        let count = parser.stack_count();
        stack_history.push(count);
        println!("After token '{}': {} active stacks", token.text, count);
    }

    parser.process_eof(input.len());
    let final_count = parser.stack_count();
    stack_history.push(final_count);
    println!("After EOF: {} active stacks", final_count);

    // Verify that merging is happening
    // Without merging, the number of stacks would grow exponentially
    // With merging, it should be more controlled
    let max_stacks = *stack_history.iter().max().unwrap();
    println!("Stack count history: {:?}", stack_history);
    println!("Maximum stacks: {}", max_stacks);

    // For "aaa", without merging we'd have many more stacks
    assert!(
        max_stacks < 20,
        "Stack count suggests merging may not be working properly: {}",
        max_stacks
    );

    let result = parser.finish();
    assert!(result.is_ok(), "Parser should complete successfully");

    println!("✓ Stack merging working correctly");
}
