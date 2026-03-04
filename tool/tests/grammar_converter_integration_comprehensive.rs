//! Comprehensive integration tests for GrammarConverter and the full pipeline.
//!
//! Tests GrammarConverter construction, full pipeline (build → normalize → FIRST/FOLLOW → LR1 → tablegen),
//! various grammar patterns, error handling, ToolError display/conversion, determinism,
//! thread safety, and edge cases.

use adze_glr_core::{FirstFollowSets, GLRError, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tablegen::{StaticLanguageGenerator, TableCompressor, collect_token_indices};
use adze_tool::{GrammarConverter, ToolError};

// ────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────

fn run_pipeline(g: &Grammar) -> adze_glr_core::ParseTable {
    let ff = FirstFollowSets::compute(g).expect("FIRST/FOLLOW failed");
    build_lr1_automaton(g, &ff).expect("LR1 build failed")
}

fn simple_expr_grammar() -> Grammar {
    GrammarBuilder::new("expr")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["expr", "+", "NUM"])
        .start("expr")
        .build()
}

// ────────────────────────────────────────────────────────
// 1. GrammarConverter construction
// ────────────────────────────────────────────────────────

#[test]
fn converter_new_is_unit_struct() {
    let _converter = GrammarConverter;
    // GrammarConverter is a unit struct — construction succeeds
}

#[test]
fn converter_create_sample_grammar_returns_nonempty() {
    let grammar = GrammarConverter::create_sample_grammar();
    assert!(!grammar.tokens.is_empty());
    assert!(!grammar.rules.is_empty());
}

#[test]
fn converter_sample_grammar_has_name() {
    let grammar = GrammarConverter::create_sample_grammar();
    assert_eq!(grammar.name, "sample");
}

#[test]
fn converter_sample_grammar_has_tokens() {
    let grammar = GrammarConverter::create_sample_grammar();
    assert!(grammar.tokens.len() >= 3);
}

#[test]
fn converter_sample_grammar_has_rules() {
    let grammar = GrammarConverter::create_sample_grammar();
    assert!(!grammar.rules.is_empty());
}

#[test]
fn converter_sample_grammar_has_fields() {
    let grammar = GrammarConverter::create_sample_grammar();
    assert!(!grammar.fields.is_empty());
}

// ────────────────────────────────────────────────────────
// 2. Full pipeline: builder → normalize → FIRST/FOLLOW → LR1 → tablegen
// ────────────────────────────────────────────────────────

#[test]
fn full_pipeline_simple_grammar() {
    let g = simple_expr_grammar();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 2);
    assert!(pt.symbol_count >= 2);
}

#[test]
fn full_pipeline_through_tablegen() {
    let g = simple_expr_grammar();
    let pt = run_pipeline(&g);
    let mut generator = StaticLanguageGenerator::new(g, pt);
    // Should not panic
    let _code = generator.generate_language_code();
}

#[test]
fn full_pipeline_node_types_generation() {
    let g = simple_expr_grammar();
    let pt = run_pipeline(&g);
    let generator = StaticLanguageGenerator::new(g, pt);
    let node_types = generator.generate_node_types();
    assert!(!node_types.is_empty());
}

#[test]
fn full_pipeline_compress_tables() {
    let g = simple_expr_grammar();
    let pt = run_pipeline(&g);
    let mut generator = StaticLanguageGenerator::new(g, pt);
    let result = generator.compress_tables();
    assert!(result.is_ok());
}

#[test]
fn full_pipeline_table_compressor() {
    let g = simple_expr_grammar();
    let pt = run_pipeline(&g);
    let token_indices = collect_token_indices(&g, &pt);
    let compressor = TableCompressor::new();
    let result = compressor.compress(&pt, &token_indices, false);
    assert!(result.is_ok());
}

// ────────────────────────────────────────────────────────
// 3. Various grammar patterns: arithmetic, list, recursive, chain
// ────────────────────────────────────────────────────────

#[test]
fn pattern_arithmetic_add_only() {
    let g = GrammarBuilder::new("add")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("e", vec!["NUM"])
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .start("e")
        .build();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 4);
}

#[test]
fn pattern_arithmetic_add_mul() {
    let g = GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("e", vec!["NUM"])
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "*", "e"], 2, Associativity::Left)
        .start("e")
        .build();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 5);
}

#[test]
fn pattern_arithmetic_four_ops() {
    let g = GrammarBuilder::new("four_ops")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .rule("e", vec!["NUM"])
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "-", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "*", "e"], 2, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "/", "e"], 2, Associativity::Left)
        .start("e")
        .build();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 5);
}

#[test]
fn pattern_list_comma_separated() {
    let g = GrammarBuilder::new("csv")
        .token("ITEM", r"[a-z]+")
        .token(",", ",")
        .rule("list", vec!["ITEM"])
        .rule("list", vec!["list", ",", "ITEM"])
        .start("list")
        .build();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 3);
}

#[test]
fn pattern_right_recursive_list() {
    let g = GrammarBuilder::new("rlist")
        .token("ITEM", r"[a-z]+")
        .token(":", ":")
        .rule("list", vec!["ITEM"])
        .rule("list", vec!["ITEM", ":", "list"])
        .start("list")
        .build();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 3);
}

#[test]
fn pattern_chain_two_nonterminals() {
    let g = GrammarBuilder::new("chain2")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .start("b")
        .build();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn pattern_chain_four_nonterminals() {
    let g = GrammarBuilder::new("chain4")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("c", vec!["b"])
        .rule("d", vec!["c"])
        .start("d")
        .build();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn pattern_nested_parentheses() {
    let g = GrammarBuilder::new("paren")
        .token("NUM", r"\d+")
        .token("(", "(")
        .token(")", ")")
        .rule("e", vec!["NUM"])
        .rule("e", vec!["(", "e", ")"])
        .start("e")
        .build();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 4);
}

#[test]
fn pattern_statement_list() {
    let g = GrammarBuilder::new("stmts")
        .token("ID", r"[a-z]+")
        .token(";", ";")
        .rule("stmt", vec!["ID", ";"])
        .rule("stmts", vec!["stmt"])
        .rule("stmts", vec!["stmts", "stmt"])
        .start("stmts")
        .build();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 3);
}

#[test]
fn pattern_binary_with_right_assoc() {
    let g = GrammarBuilder::new("rassoc")
        .token("n", "n")
        .token("^", "^")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "^", "e"], 1, Associativity::Right)
        .start("e")
        .build();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 4);
}

// ────────────────────────────────────────────────────────
// 4. Error handling for malformed grammars
// ────────────────────────────────────────────────────────

#[test]
fn error_empty_grammar_no_rules() {
    let g = GrammarBuilder::new("empty").build();
    let result = FirstFollowSets::compute(&g);
    // Empty grammar may succeed or fail — just don't panic
    let _ = result;
}

#[test]
fn error_tokens_only_no_rules() {
    let g = GrammarBuilder::new("tokens_only").token("a", "a").build();
    let result = FirstFollowSets::compute(&g);
    let _ = result;
}

#[test]
fn error_no_start_symbol() {
    let g = GrammarBuilder::new("no_start")
        .token("a", "a")
        .rule("s", vec!["a"])
        .build();
    // Should still work (start derived from first rule)
    let ff = FirstFollowSets::compute(&g);
    assert!(ff.is_ok());
}

#[test]
fn error_unreachable_rules() {
    let g = GrammarBuilder::new("unreachable")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("orphan", vec!["b"])
        .start("s")
        .build();
    // Unreachable rules are allowed
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 2);
}

// ────────────────────────────────────────────────────────
// 5. ToolError display and conversion
// ────────────────────────────────────────────────────────

#[test]
fn tool_error_multiple_word_rules_display() {
    let err = ToolError::MultipleWordRules;
    let msg = format!("{err}");
    assert!(msg.contains("word rule"));
}

#[test]
fn tool_error_multiple_prec_display() {
    let err = ToolError::MultiplePrecedenceAttributes;
    let msg = format!("{err}");
    assert!(msg.contains("prec"));
}

#[test]
fn tool_error_expected_string_literal_display() {
    let err = ToolError::ExpectedStringLiteral {
        context: "token".into(),
        actual: "42".into(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("token"));
    assert!(msg.contains("42"));
}

#[test]
fn tool_error_expected_integer_literal_display() {
    let err = ToolError::ExpectedIntegerLiteral {
        actual: "abc".into(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("abc"));
}

#[test]
fn tool_error_expected_path_type_display() {
    let err = ToolError::ExpectedPathType {
        actual: "int*".into(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("int*"));
}

#[test]
fn tool_error_expected_single_segment_display() {
    let err = ToolError::ExpectedSingleSegmentPath {
        actual: "a::b::c".into(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("a::b::c"));
}

#[test]
fn tool_error_nested_option_display() {
    let err = ToolError::NestedOptionType;
    let msg = format!("{err}");
    assert!(msg.contains("Option"));
}

#[test]
fn tool_error_struct_no_fields_display() {
    let err = ToolError::StructHasNoFields { name: "Foo".into() };
    let msg = format!("{err}");
    assert!(msg.contains("Foo"));
}

#[test]
fn tool_error_complex_symbols_display() {
    let err = ToolError::complex_symbols_not_normalized("parsing");
    let msg = format!("{err}");
    assert!(msg.contains("parsing"));
}

#[test]
fn tool_error_expected_symbol_type_display() {
    let err = ToolError::expected_symbol_type("terminal");
    let msg = format!("{err}");
    assert!(msg.contains("terminal"));
}

#[test]
fn tool_error_expected_action_type_display() {
    let err = ToolError::expected_action_type("shift");
    let msg = format!("{err}");
    assert!(msg.contains("shift"));
}

#[test]
fn tool_error_expected_error_type_display() {
    let err = ToolError::expected_error_type("syntax");
    let msg = format!("{err}");
    assert!(msg.contains("syntax"));
}

#[test]
fn tool_error_string_too_long_display() {
    let err = ToolError::string_too_long("extraction", 99999);
    let msg = format!("{err}");
    assert!(msg.contains("99999"));
}

#[test]
fn tool_error_invalid_production_display() {
    let err = ToolError::InvalidProduction {
        details: "missing LHS".into(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("missing LHS"));
}

#[test]
fn tool_error_grammar_validation_display() {
    let err = ToolError::grammar_validation("no start symbol");
    let msg = format!("{err}");
    assert!(msg.contains("no start symbol"));
}

#[test]
fn tool_error_other_display() {
    let err = ToolError::Other("custom msg".into());
    let msg = format!("{err}");
    assert!(msg.contains("custom msg"));
}

#[test]
fn tool_error_from_string() {
    let err: ToolError = String::from("oops").into();
    assert!(matches!(err, ToolError::Other(s) if s == "oops"));
}

#[test]
fn tool_error_from_str() {
    let err: ToolError = "bad".into();
    assert!(matches!(err, ToolError::Other(s) if s == "bad"));
}

#[test]
fn tool_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
    let err: ToolError = io_err.into();
    assert!(matches!(err, ToolError::Io(_)));
}

#[test]
fn tool_error_from_glr_error() {
    let glr_err = GLRError::StateMachine("boom".into());
    let err: ToolError = glr_err.into();
    assert!(matches!(err, ToolError::Glr(_)));
}

#[test]
fn tool_error_is_debug() {
    let err = ToolError::MultipleWordRules;
    let dbg = format!("{err:?}");
    assert!(!dbg.is_empty());
}

// ────────────────────────────────────────────────────────
// 6. Determinism of the full pipeline
// ────────────────────────────────────────────────────────

#[test]
fn determinism_same_grammar_same_state_count() {
    let g1 = simple_expr_grammar();
    let g2 = simple_expr_grammar();
    let pt1 = run_pipeline(&g1);
    let pt2 = run_pipeline(&g2);
    assert_eq!(pt1.state_count, pt2.state_count);
}

#[test]
fn determinism_same_symbol_count() {
    let g1 = simple_expr_grammar();
    let g2 = simple_expr_grammar();
    let pt1 = run_pipeline(&g1);
    let pt2 = run_pipeline(&g2);
    assert_eq!(pt1.symbol_count, pt2.symbol_count);
}

#[test]
fn determinism_same_rule_count() {
    let g1 = simple_expr_grammar();
    let g2 = simple_expr_grammar();
    let pt1 = run_pipeline(&g1);
    let pt2 = run_pipeline(&g2);
    assert_eq!(pt1.rules.len(), pt2.rules.len());
}

#[test]
fn determinism_action_table_rows() {
    let g1 = simple_expr_grammar();
    let g2 = simple_expr_grammar();
    let pt1 = run_pipeline(&g1);
    let pt2 = run_pipeline(&g2);
    assert_eq!(pt1.action_table.len(), pt2.action_table.len());
}

#[test]
fn determinism_goto_table_rows() {
    let g1 = simple_expr_grammar();
    let g2 = simple_expr_grammar();
    let pt1 = run_pipeline(&g1);
    let pt2 = run_pipeline(&g2);
    assert_eq!(pt1.goto_table.len(), pt2.goto_table.len());
}

#[test]
fn determinism_node_types_output() {
    let g1 = simple_expr_grammar();
    let g2 = simple_expr_grammar();
    let pt1 = run_pipeline(&g1);
    let pt2 = run_pipeline(&g2);
    let gen1 = StaticLanguageGenerator::new(g1, pt1);
    let gen2 = StaticLanguageGenerator::new(g2, pt2);
    assert_eq!(gen1.generate_node_types(), gen2.generate_node_types());
}

// ────────────────────────────────────────────────────────
// 7. Thread safety traits (Send, Sync)
// ────────────────────────────────────────────────────────

#[test]
fn grammar_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<Grammar>();
}

#[test]
fn grammar_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<Grammar>();
}

#[test]
fn tool_error_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<ToolError>();
}

#[test]
fn first_follow_sets_are_send() {
    fn assert_send<T: Send>() {}
    assert_send::<FirstFollowSets>();
}

// ────────────────────────────────────────────────────────
// 8. Empty and minimal grammars
// ────────────────────────────────────────────────────────

#[test]
fn minimal_single_token_single_rule() {
    let g = GrammarBuilder::new("min")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn minimal_grammar_normalize_is_safe() {
    let mut g = GrammarBuilder::new("min_norm")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn minimal_grammar_table_compress() {
    let g = GrammarBuilder::new("min_comp")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let pt = run_pipeline(&g);
    let token_indices = collect_token_indices(&g, &pt);
    let compressor = TableCompressor::new();
    let compressed = compressor.compress(&pt, &token_indices, false);
    assert!(compressed.is_ok());
}

#[test]
fn empty_builder_builds_grammar() {
    let g = GrammarBuilder::new("empty_builder").build();
    assert_eq!(g.name, "empty_builder");
    assert!(g.tokens.is_empty());
    assert!(g.rules.is_empty());
}

#[test]
fn empty_grammar_normalize_is_safe() {
    let mut g = GrammarBuilder::new("e").build();
    g.normalize();
    assert!(g.rules.is_empty());
}

// ────────────────────────────────────────────────────────
// 9. Grammars with many tokens (10+)
// ────────────────────────────────────────────────────────

#[test]
fn many_tokens_twelve() {
    let g = GrammarBuilder::new("many_tok")
        .token("T0", "a")
        .token("T1", "b")
        .token("T2", "c")
        .token("T3", "d")
        .token("T4", "e")
        .token("T5", "f")
        .token("T6", "g")
        .token("T7", "h")
        .token("T8", "i")
        .token("T9", "j")
        .token("T10", "k")
        .token("T11", "l")
        .rule("s", vec!["T0"])
        .rule("s", vec!["T1"])
        .rule("s", vec!["T2"])
        .rule("s", vec!["T3"])
        .rule("s", vec!["T4"])
        .rule("s", vec!["T5"])
        .rule("s", vec!["T6"])
        .rule("s", vec!["T7"])
        .rule("s", vec!["T8"])
        .rule("s", vec!["T9"])
        .rule("s", vec!["T10"])
        .rule("s", vec!["T11"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 12);
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn many_tokens_pipeline_through_compress() {
    let g = GrammarBuilder::new("many_comp")
        .token("T0", "a")
        .token("T1", "b")
        .token("T2", "c")
        .token("T3", "d")
        .token("T4", "e")
        .token("T5", "f")
        .token("T6", "g")
        .token("T7", "h")
        .token("T8", "i")
        .token("T9", "j")
        .rule("s", vec!["T0"])
        .rule("s", vec!["T1"])
        .rule("s", vec!["T2"])
        .rule("s", vec!["T3"])
        .rule("s", vec!["T4"])
        .rule("s", vec!["T5"])
        .rule("s", vec!["T6"])
        .rule("s", vec!["T7"])
        .rule("s", vec!["T8"])
        .rule("s", vec!["T9"])
        .start("s")
        .build();
    let pt = run_pipeline(&g);
    let token_indices = collect_token_indices(&g, &pt);
    let compressor = TableCompressor::new();
    assert!(compressor.compress(&pt, &token_indices, false).is_ok());
}

#[test]
fn many_tokens_sequence() {
    let g = GrammarBuilder::new("tok_seq")
        .token("T0", "a")
        .token("T1", "b")
        .token("T2", "c")
        .token("T3", "d")
        .token("T4", "e")
        .token("T5", "f")
        .token("T6", "g")
        .token("T7", "h")
        .token("T8", "i")
        .token("T9", "j")
        .rule(
            "s",
            vec!["T0", "T1", "T2", "T3", "T4", "T5", "T6", "T7", "T8", "T9"],
        )
        .start("s")
        .build();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 10);
}

// ────────────────────────────────────────────────────────
// 10. Grammars with many rules (10+)
// ────────────────────────────────────────────────────────

#[test]
fn many_rules_alternatives() {
    let builder = GrammarBuilder::new("many_rules")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .token("^", "^")
        .rule("e", vec!["a"])
        .rule("e", vec!["b"])
        .rule("e", vec!["c"])
        .rule("e", vec!["d"])
        .rule("e", vec!["e"])
        .rule_with_precedence("expr", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["e", "-", "e"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["e", "*", "e"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["e", "/", "e"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["e", "^", "e"], 3, Associativity::Right)
        .rule("expr", vec!["e"])
        .start("expr")
        .build();
    let total_rules: usize = builder.rules.values().map(|v| v.len()).sum();
    assert!(total_rules >= 10);
}

#[test]
fn many_rules_nonterminal_chain() {
    let g = GrammarBuilder::new("chain10")
        .token("x", "x")
        .rule("r0", vec!["x"])
        .rule("r1", vec!["r0"])
        .rule("r2", vec!["r1"])
        .rule("r3", vec!["r2"])
        .rule("r4", vec!["r3"])
        .rule("r5", vec!["r4"])
        .rule("r6", vec!["r5"])
        .rule("r7", vec!["r6"])
        .rule("r8", vec!["r7"])
        .rule("r9", vec!["r8"])
        .start("r9")
        .build();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn many_rules_pipeline_succeeds() {
    let g = GrammarBuilder::new("mr_pipe")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .rule("atom", vec!["a"])
        .rule("atom", vec!["b"])
        .rule("atom", vec!["c"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["atom"])
        .rule("stmt", vec!["expr"])
        .rule("stmts", vec!["stmt"])
        .rule("stmts", vec!["stmts", "stmt"])
        .rule("prog", vec!["stmts"])
        .start("prog")
        .build();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 2);
}

// ────────────────────────────────────────────────────────
// Additional: Grammar patterns and edge cases
// ────────────────────────────────────────────────────────

#[test]
fn grammar_clone_produces_equal() {
    let g = simple_expr_grammar();
    let g2 = g.clone();
    assert_eq!(g, g2);
}

#[test]
fn grammar_normalize_idempotent() {
    let mut g = simple_expr_grammar();
    g.normalize();
    let count1: usize = g.rules.values().map(|v| v.len()).sum();
    g.normalize();
    let count2: usize = g.rules.values().map(|v| v.len()).sum();
    assert_eq!(count1, count2);
}

#[test]
fn grammar_start_symbol_exists() {
    let g = simple_expr_grammar();
    assert!(g.start_symbol().is_some());
}

#[test]
fn grammar_builder_python_like() {
    let g = GrammarBuilder::python_like();
    assert_eq!(g.name, "python_like");
    assert!(!g.externals.is_empty());
}

#[test]
fn grammar_builder_javascript_like() {
    let g = GrammarBuilder::javascript_like();
    assert_eq!(g.name, "javascript_like");
    assert!(g.tokens.len() >= 10);
}

#[test]
fn nullable_start_pipeline() {
    let g = GrammarBuilder::new("nullable")
        .token("a", "a")
        .rule("s", vec![])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g);
    assert!(ff.is_ok());
}

#[test]
fn multiple_start_alternatives() {
    let g = GrammarBuilder::new("multi_alt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .rule("s", vec!["d"])
        .rule("s", vec!["e"])
        .start("s")
        .build();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn table_compressor_default() {
    let compressor = TableCompressor::default();
    let g = simple_expr_grammar();
    let pt = run_pipeline(&g);
    let token_indices = collect_token_indices(&g, &pt);
    assert!(compressor.compress(&pt, &token_indices, false).is_ok());
}

#[test]
fn static_language_generator_set_start_empty() {
    let g = simple_expr_grammar();
    let pt = run_pipeline(&g);
    let mut generator = StaticLanguageGenerator::new(g, pt);
    generator.set_start_can_be_empty(true);
    assert!(generator.start_can_be_empty);
}

#[test]
fn pipeline_preserves_eof_symbol() {
    let g = simple_expr_grammar();
    let pt = run_pipeline(&g);
    // EOF symbol should be set
    assert!(pt.eof_symbol.0 > 0 || pt.symbol_to_index.contains_key(&pt.eof_symbol));
}

#[test]
fn pipeline_preserves_start_symbol() {
    let g = simple_expr_grammar();
    let pt = run_pipeline(&g);
    assert!(pt.start_symbol.0 > 0);
}

#[test]
fn pipeline_action_table_nonempty() {
    let g = simple_expr_grammar();
    let pt = run_pipeline(&g);
    assert!(!pt.action_table.is_empty());
}

#[test]
fn pipeline_goto_table_nonempty() {
    let g = simple_expr_grammar();
    let pt = run_pipeline(&g);
    assert!(!pt.goto_table.is_empty());
}

#[test]
fn pipeline_rules_nonempty() {
    let g = simple_expr_grammar();
    let pt = run_pipeline(&g);
    assert!(!pt.rules.is_empty());
}

#[test]
fn pattern_single_char_tokens() {
    let g = GrammarBuilder::new("chars")
        .token("(", "(")
        .token(")", ")")
        .token("a", "a")
        .rule("s", vec!["(", "a", ")"])
        .start("s")
        .build();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 3);
}

#[test]
fn pattern_mixed_prec_and_no_prec() {
    let g = GrammarBuilder::new("mixed")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("e", vec!["NUM"])
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule("e", vec!["e", "*", "e"])
        .start("e")
        .build();
    let pt = run_pipeline(&g);
    assert!(pt.state_count >= 4);
}

#[test]
fn compress_start_can_be_empty_true() {
    let g = simple_expr_grammar();
    let pt = run_pipeline(&g);
    let token_indices = collect_token_indices(&g, &pt);
    let compressor = TableCompressor::new();
    assert!(compressor.compress(&pt, &token_indices, true).is_ok());
}

#[test]
fn compress_start_can_be_empty_false() {
    let g = simple_expr_grammar();
    let pt = run_pipeline(&g);
    let token_indices = collect_token_indices(&g, &pt);
    let compressor = TableCompressor::new();
    assert!(compressor.compress(&pt, &token_indices, false).is_ok());
}

#[test]
fn first_follow_multiple_nonterminals() {
    let g = GrammarBuilder::new("ff_multi")
        .token("a", "a")
        .token("b", "b")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("s", vec!["x", "y"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = g.start_symbol().unwrap();
    assert!(ff.first(start).is_some());
}

#[test]
fn grammar_with_extras() {
    let g = GrammarBuilder::new("extras")
        .token("a", "a")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(!g.extras.is_empty());
}

#[test]
fn grammar_with_external_scanner() {
    let g = GrammarBuilder::new("ext")
        .token("a", "a")
        .token("IND", "INDENT")
        .external("IND")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(!g.externals.is_empty());
}

#[test]
fn static_generator_language_code_nonempty() {
    let g = simple_expr_grammar();
    let pt = run_pipeline(&g);
    let generator = StaticLanguageGenerator::new(g, pt);
    let code = generator.generate_language_code();
    assert!(!code.is_empty());
}
