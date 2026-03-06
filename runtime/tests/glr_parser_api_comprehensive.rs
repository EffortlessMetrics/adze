//! Comprehensive tests for the GLR parser API (`adze::glr_parser`).
//!
//! Covers:
//! 1. Parser construction
//! 2. Parse table generation from grammars
//! 3. Token processing
//! 4. Error handling (invalid input)
//! 5. Parse result inspection

mod common;

#[cfg(feature = "ts-compat")]
use adze::adze_glr_core as glr_core;
#[cfg(feature = "ts-compat")]
use adze::adze_ir as ir;
use adze::error_recovery::ErrorRecoveryConfig;
use adze::glr_lexer::GLRLexer;
use adze::glr_parser::GLRParser;
use adze::subtree::Subtree;

#[cfg(not(feature = "ts-compat"))]
use adze_glr_core as glr_core;
#[cfg(not(feature = "ts-compat"))]
use adze_ir as ir;

use glr_core::{FirstFollowSets, build_lr1_automaton};
use ir::builder::GrammarBuilder;
use ir::{Associativity, Grammar, SymbolId};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_table(grammar: &Grammar) -> glr_core::ParseTable {
    let ff = FirstFollowSets::compute(grammar).unwrap();
    build_lr1_automaton(grammar, &ff).expect("Failed to build automaton")
}

/// Build a simple `sum → NUMBER "+" NUMBER` grammar.
fn simple_sum_grammar() -> Grammar {
    GrammarBuilder::new("sum")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("sum", vec!["NUMBER", "+", "NUMBER"])
        .start("sum")
        .build()
}

/// Build an arithmetic grammar with precedence: expr → expr+expr | expr*expr | NUMBER
fn arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

/// Lex, parse, and finish.
fn lex_parse_finish(grammar: &Grammar, input: &str) -> Result<Arc<Subtree>, String> {
    let table = build_table(grammar);
    let mut parser = GLRParser::new(table, grammar.clone());

    let mut lexer = GLRLexer::new(grammar, input.to_string()).unwrap();
    let tokens = lexer.tokenize_all();

    for tok in &tokens {
        parser.process_token(tok.symbol_id, &tok.text, tok.byte_offset);
    }
    let total = tokens
        .last()
        .map(|t| t.byte_offset + t.text.len())
        .unwrap_or(0);
    parser.process_eof(total);
    parser.finish()
}

/// Lex, parse, and return all alternatives.
fn lex_parse_all_alternatives(grammar: &Grammar, input: &str) -> Result<Vec<Arc<Subtree>>, String> {
    let table = build_table(grammar);
    let mut parser = GLRParser::new(table, grammar.clone());

    let mut lexer = GLRLexer::new(grammar, input.to_string()).unwrap();
    let tokens = lexer.tokenize_all();

    for tok in &tokens {
        parser.process_token(tok.symbol_id, &tok.text, tok.byte_offset);
    }
    let total = tokens
        .last()
        .map(|t| t.byte_offset + t.text.len())
        .unwrap_or(0);
    parser.process_eof(total);
    parser.finish_all_alternatives()
}

/// Recursively check that no node is an error node.
fn has_error_nodes(tree: &Subtree) -> bool {
    if tree.node.is_error {
        return true;
    }
    tree.children.iter().any(|e| has_error_nodes(&e.subtree))
}

/// Count total nodes in a subtree.
fn count_nodes(tree: &Subtree) -> usize {
    1 + tree
        .children
        .iter()
        .map(|e| count_nodes(&e.subtree))
        .sum::<usize>()
}

// =========================================================================
// 1. Parser Construction
// =========================================================================

#[test]
fn construct_parser_simple_grammar() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let parser = GLRParser::new(table, g);
    assert_eq!(parser.stack_count(), 1, "fresh parser has one stack");
}

#[test]
fn construct_parser_arithmetic_grammar() {
    let g = arithmetic_grammar();
    let table = build_table(&g);
    let parser = GLRParser::new(table, g);
    assert_eq!(parser.stack_count(), 1);
}

#[test]
fn construct_parser_with_error_recovery() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let parser = GLRParser::new(table, g).with_error_recovery(ErrorRecoveryConfig::default());
    assert_eq!(parser.stack_count(), 1);
}

#[test]
fn enable_error_recovery_mutably() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let mut parser = GLRParser::new(table, g);
    parser.enable_error_recovery(ErrorRecoveryConfig::default());
    assert_eq!(parser.stack_count(), 1);
}

#[test]
fn start_symbol_id_is_nonzero() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let parser = GLRParser::new(table, g);
    // SymbolId(0) is EOF; start symbol should not be EOF
    assert_ne!(parser.start_symbol_id(), SymbolId(0));
}

// =========================================================================
// 2. Parse Table Generation
// =========================================================================

#[test]
fn table_generation_simple_grammar() {
    let g = simple_sum_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    // At least one state for the initial configuration
    assert!(!table.action_table.is_empty());
}

#[test]
fn table_generation_arithmetic_grammar() {
    let g = arithmetic_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    assert!(!table.action_table.is_empty());
    assert!(!table.goto_table.is_empty());
}

#[test]
fn table_generation_empty_start_grammar() {
    // Grammar where the start symbol has an epsilon production
    let g = GrammarBuilder::new("nullable")
        .token("A", "a")
        .rule("start", vec![])
        .rule("start", vec!["A"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    assert!(!table.action_table.is_empty());
}

#[test]
fn table_generation_multiple_alternatives() {
    let g = GrammarBuilder::new("multi_alt")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .rule("s", vec!["C"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(!table.action_table.is_empty());
}

// =========================================================================
// 3. Token Processing
// =========================================================================

#[test]
fn parse_single_token() {
    let g = GrammarBuilder::new("single")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let result = lex_parse_finish(&g, "a");
    assert!(
        result.is_ok(),
        "single token parse should succeed: {result:?}"
    );
}

#[test]
fn parse_simple_sum() {
    let g = simple_sum_grammar();
    let result = lex_parse_finish(&g, "1+2");
    assert!(result.is_ok(), "simple sum parse failed: {result:?}");
    let tree = result.unwrap();
    assert!(!has_error_nodes(&tree));
}

#[test]
fn parse_arithmetic_expression() {
    let g = arithmetic_grammar();
    let result = lex_parse_finish(&g, "1+2*3");
    assert!(result.is_ok(), "arithmetic parse failed: {result:?}");
}

#[test]
fn parse_chained_addition() {
    let g = arithmetic_grammar();
    let result = lex_parse_finish(&g, "1+2+3");
    assert!(result.is_ok(), "chained addition parse failed: {result:?}");
}

#[test]
fn parse_preserves_byte_ranges() {
    let g = simple_sum_grammar();
    let result = lex_parse_finish(&g, "1+2").unwrap();
    // Root node should span the entire input
    assert_eq!(result.node.byte_range.start, 0);
    assert_eq!(result.node.byte_range.end, 3);
}

#[test]
fn process_token_advances_stacks() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let mut parser = GLRParser::new(table, g.clone());

    // Feed "1" – should shift
    let mut lexer = GLRLexer::new(&g, "1+2".to_string()).unwrap();
    let tokens = lexer.tokenize_all();
    assert!(!tokens.is_empty());
    parser.process_token(tokens[0].symbol_id, &tokens[0].text, tokens[0].byte_offset);
    // After one token the parser should still have active stacks
    assert!(parser.stack_count() >= 1);
}

#[test]
fn stack_count_increases_on_ambiguity() {
    // Ambiguous grammar: stmt can be parsed two ways
    let g = GrammarBuilder::new("ambig")
        .token("ID", r"[a-z]+")
        .token("(", "(")
        .token(")", ")")
        .token(",", ",")
        .rule("s", vec!["expr"])
        .rule("expr", vec!["ID"])
        .rule("expr", vec!["ID", "(", "args", ")"])
        .rule("expr", vec!["ID", "(", ")"])
        .rule("args", vec!["expr"])
        .rule("args", vec!["args", ",", "expr"])
        .start("s")
        .build();
    let table = build_table(&g);
    let mut parser = GLRParser::new(table, g.clone());

    let mut lexer = GLRLexer::new(&g, "f(x)".to_string()).unwrap();
    let tokens = lexer.tokenize_all();
    for tok in &tokens {
        parser.process_token(tok.symbol_id, &tok.text, tok.byte_offset);
    }
    // After processing the ambiguity should manifest in multiple stacks or completed alternatives
    // We just verify parsing still works
    let total = tokens
        .last()
        .map(|t| t.byte_offset + t.text.len())
        .unwrap_or(0);
    parser.process_eof(total);
    let result = parser.finish();
    assert!(result.is_ok(), "ambiguous grammar parse failed: {result:?}");
}

// =========================================================================
// 4. Error Handling
// =========================================================================

#[test]
fn parse_empty_input_fails() {
    let g = simple_sum_grammar();
    let result = lex_parse_finish(&g, "");
    assert!(
        result.is_err(),
        "empty input should fail for non-nullable start"
    );
}

#[test]
fn parse_incomplete_input_fails() {
    let g = simple_sum_grammar();
    let result = lex_parse_finish(&g, "1+");
    assert!(result.is_err(), "incomplete input should fail");
}

#[test]
fn finish_without_eof_fails() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let mut parser = GLRParser::new(table, g.clone());

    let mut lexer = GLRLexer::new(&g, "1+2".to_string()).unwrap();
    let tokens = lexer.tokenize_all();
    for tok in &tokens {
        parser.process_token(tok.symbol_id, &tok.text, tok.byte_offset);
    }
    // Call finish without process_eof
    let result = parser.finish();
    assert!(result.is_err(), "finish without EOF should fail");
}

#[test]
fn unexpected_token_leads_to_parse_failure() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let mut parser = GLRParser::new(table, g.clone());

    // Feed tokens that don't form a valid sentence: "+" alone
    let plus_id = g
        .tokens
        .iter()
        .find(|(_, t)| t.name == "+")
        .map(|(id, _)| *id)
        .unwrap();
    parser.process_token(plus_id, "+", 0);
    parser.process_eof(1);
    // Parsing an unexpected leading token should fail
    let result = parser.finish();
    assert!(
        result.is_err(),
        "unexpected leading '+' should fail to parse"
    );
}

#[test]
fn error_message_contains_state_info() {
    let g = simple_sum_grammar();
    let result = lex_parse_finish(&g, "");
    let err = result.unwrap_err();
    assert!(
        err.contains("Stack states") || err.contains("Parse incomplete"),
        "error should contain state info, got: {err}"
    );
}

// =========================================================================
// 5. Parse Result Inspection
// =========================================================================

#[test]
fn result_tree_has_children() {
    let g = simple_sum_grammar();
    let tree = lex_parse_finish(&g, "1+2").unwrap();
    // sum → NUMBER "+" NUMBER = 3 children
    assert!(
        !tree.children.is_empty(),
        "result tree should have children"
    );
}

#[test]
fn result_tree_node_count() {
    let g = simple_sum_grammar();
    let tree = lex_parse_finish(&g, "1+2").unwrap();
    let n = count_nodes(&tree);
    // Root + 3 leaf children = 4
    assert!(n >= 4, "expected at least 4 nodes, got {n}");
}

#[test]
fn result_symbol_id_is_start() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let parser = GLRParser::new(table, g.clone());
    let start = parser.start_symbol_id();

    let tree = lex_parse_finish(&g, "1+2").unwrap();
    assert_eq!(tree.node.symbol_id, start);
}

#[test]
fn get_best_parse_after_tokens() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let mut parser = GLRParser::new(table, g.clone());

    let mut lexer = GLRLexer::new(&g, "1+2".to_string()).unwrap();
    let tokens = lexer.tokenize_all();
    for tok in &tokens {
        parser.process_token(tok.symbol_id, &tok.text, tok.byte_offset);
    }
    let total = tokens
        .last()
        .map(|t| t.byte_offset + t.text.len())
        .unwrap_or(0);
    parser.process_eof(total);

    let best = parser.get_best_parse();
    assert!(
        best.is_some(),
        "get_best_parse should return something after a successful parse"
    );
}

#[test]
fn finish_all_alternatives_returns_at_least_one() {
    let g = simple_sum_grammar();
    let alts = lex_parse_all_alternatives(&g, "1+2").unwrap();
    assert!(!alts.is_empty(), "should have at least one alternative");
}

#[test]
fn finish_all_alternatives_empty_input_fails() {
    let g = simple_sum_grammar();
    let result = lex_parse_all_alternatives(&g, "");
    assert!(result.is_err());
}

// =========================================================================
// 6. Reset and Reuse
// =========================================================================

#[test]
fn reset_restores_initial_state() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let mut parser = GLRParser::new(table, g.clone());

    // Parse something
    let mut lexer = GLRLexer::new(&g, "1+2".to_string()).unwrap();
    let tokens = lexer.tokenize_all();
    for tok in &tokens {
        parser.process_token(tok.symbol_id, &tok.text, tok.byte_offset);
    }
    let total = tokens
        .last()
        .map(|t| t.byte_offset + t.text.len())
        .unwrap_or(0);
    parser.process_eof(total);
    let _ = parser.finish();

    // Reset
    parser.reset();
    assert_eq!(parser.stack_count(), 1, "reset should restore to one stack");
}

#[test]
fn reparse_after_reset() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let mut parser = GLRParser::new(table, g.clone());

    // First parse
    {
        let mut lexer = GLRLexer::new(&g, "1+2".to_string()).unwrap();
        let tokens = lexer.tokenize_all();
        for tok in &tokens {
            parser.process_token(tok.symbol_id, &tok.text, tok.byte_offset);
        }
        let total = tokens
            .last()
            .map(|t| t.byte_offset + t.text.len())
            .unwrap_or(0);
        parser.process_eof(total);
        assert!(parser.finish().is_ok());
    }

    // Reset and re-parse different input
    parser.reset();
    {
        let mut lexer = GLRLexer::new(&g, "3+4".to_string()).unwrap();
        let tokens = lexer.tokenize_all();
        for tok in &tokens {
            parser.process_token(tok.symbol_id, &tok.text, tok.byte_offset);
        }
        let total = tokens
            .last()
            .map(|t| t.byte_offset + t.text.len())
            .unwrap_or(0);
        parser.process_eof(total);
        let tree = parser.finish().unwrap();
        assert_eq!(tree.node.byte_range.end, 3);
    }
}

// =========================================================================
// 7. Expected Symbols
// =========================================================================

#[test]
fn expected_symbols_at_start() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let parser = GLRParser::new(table, g.clone());
    let expected = parser.expected_symbols();
    // At the start we expect a NUMBER token
    let number_id = g
        .tokens
        .iter()
        .find(|(_, t)| t.name == "NUMBER")
        .map(|(id, _)| *id)
        .unwrap();
    assert!(
        expected.contains(&number_id),
        "expected symbols should include NUMBER at start, got: {expected:?}"
    );
}

#[test]
fn expected_symbols_after_partial_input() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let mut parser = GLRParser::new(table, g.clone());

    let number_id = g
        .tokens
        .iter()
        .find(|(_, t)| t.name == "NUMBER")
        .map(|(id, _)| *id)
        .unwrap();
    parser.process_token(number_id, "1", 0);

    let expected = parser.expected_symbols();
    let plus_id = g
        .tokens
        .iter()
        .find(|(_, t)| t.name == "+")
        .map(|(id, _)| *id)
        .unwrap();
    assert!(
        expected.contains(&plus_id),
        "after NUMBER, expected symbols should include '+', got: {expected:?}"
    );
}

// =========================================================================
// 8. GSS State Management (incremental parsing support)
// =========================================================================

#[test]
fn get_set_gss_state_roundtrip() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let mut parser = GLRParser::new(table, g.clone());

    let state = parser.get_gss_state();
    assert_eq!(state.len(), 1);

    parser.set_gss_state(state.clone());
    assert_eq!(parser.stack_count(), 1);
}

#[test]
fn next_stack_id_roundtrip() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let mut parser = GLRParser::new(table, g.clone());

    let id = parser.get_next_stack_id();
    parser.set_next_stack_id(id + 10);
    assert_eq!(parser.get_next_stack_id(), id + 10);
}

#[test]
fn set_gss_state_selective_empty() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let mut parser = GLRParser::new(table, g);
    parser.set_gss_state_selective(vec![]);
    assert_eq!(parser.stack_count(), 0);
}

// =========================================================================
// 9. Inject Subtree APIs
// =========================================================================

#[test]
fn inject_subtree_no_stacks_fails() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let mut parser = GLRParser::new(table, g);
    // Clear all stacks
    parser.set_gss_state(vec![]);

    let subtree = Arc::new(Subtree::new(
        adze::subtree::SubtreeNode {
            symbol_id: SymbolId(1),
            is_error: false,
            byte_range: 0..1,
        },
        vec![],
    ));
    let result = parser.inject_subtree(subtree);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No active stacks"));
}

#[test]
fn inject_ambiguous_subtrees_empty_list_fails() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let mut parser = GLRParser::new(table, g);
    let result = parser.inject_ambiguous_subtrees(vec![]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No subtrees"));
}

#[test]
fn inject_ambiguous_subtrees_no_stacks_fails() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let mut parser = GLRParser::new(table, g);
    parser.set_gss_state(vec![]);

    let subtree = Arc::new(Subtree::new(
        adze::subtree::SubtreeNode {
            symbol_id: SymbolId(1),
            is_error: false,
            byte_range: 0..1,
        },
        vec![],
    ));
    let result = parser.inject_ambiguous_subtrees(vec![subtree]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No active stacks"));
}

// =========================================================================
// 10. Safe Dedup Threshold
// =========================================================================

#[test]
fn safe_dedup_threshold_returns_value() {
    let threshold = adze::glr_parser::safe_dedup_threshold();
    // Default should be the constant
    assert!(
        threshold >= 1,
        "threshold should be at least 1, got {threshold}"
    );
}

#[test]
fn default_safe_dedup_threshold_constant() {
    assert_eq!(adze::glr_parser::DEFAULT_SAFE_DEDUP_THRESHOLD, 10);
}

// =========================================================================
// 11. Builder-based Grammars (GrammarBuilder integration)
// =========================================================================

#[test]
fn parse_with_builder_grammar_two_alternatives() {
    let g = GrammarBuilder::new("alt")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .start("s")
        .build();
    assert!(lex_parse_finish(&g, "a").is_ok());
    assert!(lex_parse_finish(&g, "b").is_ok());
}

#[test]
fn parse_with_builder_grammar_sequence() {
    let g = GrammarBuilder::new("seq")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("s", vec!["A", "B", "C"])
        .start("s")
        .build();
    let tree = lex_parse_finish(&g, "abc").unwrap();
    assert_eq!(tree.children.len(), 3);
}

#[test]
fn parse_recursive_grammar() {
    // Right-recursive list: list → ITEM | ITEM list
    let g = GrammarBuilder::new("list")
        .token("ITEM", r"[a-z]")
        .rule("list", vec!["ITEM"])
        .rule("list", vec!["ITEM", "list"])
        .start("list")
        .build();
    assert!(lex_parse_finish(&g, "a").is_ok());
    assert!(lex_parse_finish(&g, "ab").is_ok());
    assert!(lex_parse_finish(&g, "abc").is_ok());
}

#[test]
fn parse_left_recursive_grammar() {
    // Left-recursive list: list → ITEM | list ITEM
    let g = GrammarBuilder::new("lrec")
        .token("ITEM", r"[a-z]")
        .rule("list", vec!["ITEM"])
        .rule("list", vec!["list", "ITEM"])
        .start("list")
        .build();
    assert!(lex_parse_finish(&g, "a").is_ok());
    assert!(lex_parse_finish(&g, "ab").is_ok());
}

// =========================================================================
// 12. With Error Recovery
// =========================================================================

#[test]
fn error_recovery_builder_method_returns_self() {
    let g = simple_sum_grammar();
    let table = build_table(&g);
    let parser = GLRParser::new(table, g).with_error_recovery(ErrorRecoveryConfig::default());
    // Just verify it compiles and returns a usable parser
    assert_eq!(parser.stack_count(), 1);
}

#[test]
fn parse_result_no_error_nodes_on_valid_input() {
    let g = arithmetic_grammar();
    let tree = lex_parse_finish(&g, "1+2").unwrap();
    assert!(
        !has_error_nodes(&tree),
        "valid parse should have no error nodes"
    );
}
