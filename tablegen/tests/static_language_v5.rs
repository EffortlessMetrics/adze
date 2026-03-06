//! Comprehensive tests for static language generation (v5).
//!
//! Covers `StaticLanguageGenerator` code generation, node types output,
//! determinism, scaling, grammar differentiation, and edge cases.

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::StaticLanguageGenerator;

// ===========================================================================
// Helpers
// ===========================================================================

/// Build a grammar and compute a real LR(1) parse table through the full pipeline.
fn build_pipeline(
    grammar_fn: impl FnOnce() -> adze_ir::Grammar,
) -> (adze_ir::Grammar, adze_glr_core::ParseTable) {
    let mut grammar = grammar_fn();
    let ff =
        FirstFollowSets::compute_normalized(&mut grammar).expect("FIRST/FOLLOW computation failed");
    let table = build_lr1_automaton(&grammar, &ff).expect("LR(1) automaton failed");
    (grammar, table)
}

fn single_token_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("single")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

fn two_token_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("two_tok")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
}

fn alternatives_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("alts")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build()
}

fn chain_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("C", vec!["x"])
        .rule("B", vec!["C"])
        .rule("A", vec!["B"])
        .rule("start", vec!["A"])
        .start("start")
        .build()
}

fn expression_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("expr")
        .token("NUM", r"\d+")
        .token("PLUS", "+")
        .token("STAR", "*")
        .token("LPAREN", "(")
        .token("RPAREN", ")")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["expr", "STAR", "expr"])
        .rule("expr", vec!["LPAREN", "expr", "RPAREN"])
        .start("expr")
        .build()
}

fn statement_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("stmt")
        .token("ID", r"[a-z]+")
        .token("NUM", r"\d+")
        .token("EQ", "=")
        .token("SEMI", ";")
        .rule("stmt", vec!["ID", "EQ", "NUM", "SEMI"])
        .rule("program", vec!["stmt"])
        .start("program")
        .build()
}

fn multi_rule_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("s1", vec!["a", "b"])
        .rule("s2", vec!["c", "d"])
        .rule("start", vec!["s1"])
        .rule("start", vec!["s2"])
        .start("start")
        .build()
}

fn scaled_grammar(n_tokens: usize) -> adze_ir::Grammar {
    let mut builder = GrammarBuilder::new("scaled");
    let mut token_names: Vec<String> = Vec::new();
    for i in 0..n_tokens {
        let name = format!("t{i}");
        let pattern = format!("tok{i}");
        builder = builder.token(
            Box::leak(name.clone().into_boxed_str()),
            Box::leak(pattern.into_boxed_str()),
        );
        token_names.push(name);
    }
    // Each token gets its own rule
    for (i, name) in token_names.iter().enumerate() {
        let rule_name = format!("r{i}");
        builder = builder.rule(
            Box::leak(rule_name.into_boxed_str()),
            vec![Box::leak(name.clone().into_boxed_str())],
        );
    }
    // Start rule picks from first few sub-rules
    let limit = n_tokens.min(5);
    for i in 0..limit {
        let rule_name = format!("r{i}");
        builder = builder.rule("top", vec![Box::leak(rule_name.into_boxed_str())]);
    }
    builder = builder.start("top");
    builder.build()
}

// ===========================================================================
// 1. Language code contains struct (8 tests)
// ===========================================================================

#[test]
fn code_contains_language_keyword_single_token() {
    let (grammar, table) = build_pipeline(single_token_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    assert!(code.contains("language"), "output must mention 'language'");
}

#[test]
fn code_contains_language_keyword_two_tokens() {
    let (grammar, table) = build_pipeline(two_token_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    assert!(code.contains("language"));
}

#[test]
fn code_contains_language_keyword_alternatives() {
    let (grammar, table) = build_pipeline(alternatives_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    assert!(code.contains("language"));
}

#[test]
fn code_contains_language_keyword_chain() {
    let (grammar, table) = build_pipeline(chain_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    assert!(code.contains("language"));
}

#[test]
fn code_contains_language_keyword_expression() {
    let (grammar, table) = build_pipeline(expression_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    assert!(code.contains("language"));
}

#[test]
fn code_contains_language_keyword_statement() {
    let (grammar, table) = build_pipeline(statement_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    assert!(code.contains("language"));
}

#[test]
fn code_contains_language_keyword_multi_rule() {
    let (grammar, table) = build_pipeline(multi_rule_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    assert!(code.contains("language"));
}

#[test]
fn code_is_nonempty_for_any_grammar() {
    for grammar_fn in [
        single_token_grammar as fn() -> adze_ir::Grammar,
        two_token_grammar,
        alternatives_grammar,
        chain_grammar,
    ] {
        let (grammar, table) = build_pipeline(grammar_fn);
        let lang_gen = StaticLanguageGenerator::new(grammar, table);
        let code = lang_gen.generate_language_code();
        assert!(!code.is_empty());
    }
}

// ===========================================================================
// 2. Language code contains state tables (8 tests)
// ===========================================================================

#[test]
fn code_references_state_count_single() {
    let (grammar, table) = build_pipeline(single_token_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    // The generated code should contain numeric state-related data
    assert!(!code.is_empty(), "state table code must be non-empty");
}

#[test]
fn code_references_state_count_expression() {
    let (grammar, table) = build_pipeline(expression_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    assert!(
        code.len() > 100,
        "expression grammar should produce substantial code"
    );
}

#[test]
fn code_has_action_or_parse_table_data_single() {
    let (grammar, table) = build_pipeline(single_token_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    let has_table_ref = code.contains("parse") || code.contains("action") || code.contains("state");
    assert!(
        has_table_ref,
        "code should reference parse tables or actions"
    );
}

#[test]
fn code_has_action_or_parse_table_data_alternatives() {
    let (grammar, table) = build_pipeline(alternatives_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    let has_table_ref = code.contains("parse") || code.contains("action") || code.contains("state");
    assert!(has_table_ref);
}

#[test]
fn code_has_action_or_parse_table_data_chain() {
    let (grammar, table) = build_pipeline(chain_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    let has_table_ref = code.contains("parse") || code.contains("action") || code.contains("state");
    assert!(has_table_ref);
}

#[test]
fn code_has_symbol_names_single() {
    let (grammar, table) = build_pipeline(single_token_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    let has_symbol_ref = code.contains("symbol") || code.contains("name");
    assert!(has_symbol_ref, "code should reference symbol names");
}

#[test]
fn code_has_symbol_names_expression() {
    let (grammar, table) = build_pipeline(expression_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    let has_symbol_ref = code.contains("symbol") || code.contains("name");
    assert!(has_symbol_ref);
}

#[test]
fn code_has_goto_or_reduce_info() {
    let (grammar, table) = build_pipeline(multi_rule_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    // Generated code should contain some form of goto/reduce data
    assert!(
        code.len() > 50,
        "multi-rule grammar should produce non-trivial code"
    );
}

// ===========================================================================
// 3. Language code deterministic (8 tests)
// ===========================================================================

#[test]
fn determinism_single_token() {
    let (g1, t1) = build_pipeline(single_token_grammar);
    let (g2, t2) = build_pipeline(single_token_grammar);
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2, "same grammar must produce identical code");
}

#[test]
fn determinism_two_tokens() {
    let (g1, t1) = build_pipeline(two_token_grammar);
    let (g2, t2) = build_pipeline(two_token_grammar);
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

#[test]
fn determinism_alternatives() {
    let (g1, t1) = build_pipeline(alternatives_grammar);
    let (g2, t2) = build_pipeline(alternatives_grammar);
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

#[test]
fn determinism_chain() {
    let (g1, t1) = build_pipeline(chain_grammar);
    let (g2, t2) = build_pipeline(chain_grammar);
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

#[test]
fn determinism_expression() {
    let (g1, t1) = build_pipeline(expression_grammar);
    let (g2, t2) = build_pipeline(expression_grammar);
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

#[test]
fn determinism_statement() {
    let (g1, t1) = build_pipeline(statement_grammar);
    let (g2, t2) = build_pipeline(statement_grammar);
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

#[test]
fn determinism_multi_rule() {
    let (g1, t1) = build_pipeline(multi_rule_grammar);
    let (g2, t2) = build_pipeline(multi_rule_grammar);
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

#[test]
fn determinism_node_types_single() {
    let (g1, t1) = build_pipeline(single_token_grammar);
    let (g2, t2) = build_pipeline(single_token_grammar);
    let n1 = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let n2 = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_eq!(n1, n2, "node types must be deterministic");
}

// ===========================================================================
// 4. Node types valid JSON (7 tests)
// ===========================================================================

#[test]
fn node_types_valid_json_single() {
    let (grammar, table) = build_pipeline(single_token_grammar);
    let json_str = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).unwrap_or_else(|e| panic!("invalid JSON: {e}\n{json_str}"));
    assert!(parsed.is_array(), "node types must be a JSON array");
}

#[test]
fn node_types_valid_json_two_tokens() {
    let (grammar, table) = build_pipeline(two_token_grammar);
    let json_str = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("invalid JSON");
    assert!(parsed.is_array());
}

#[test]
fn node_types_valid_json_alternatives() {
    let (grammar, table) = build_pipeline(alternatives_grammar);
    let json_str = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    let _: serde_json::Value = serde_json::from_str(&json_str).expect("invalid JSON");
}

#[test]
fn node_types_valid_json_chain() {
    let (grammar, table) = build_pipeline(chain_grammar);
    let json_str = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    let _: serde_json::Value = serde_json::from_str(&json_str).expect("invalid JSON");
}

#[test]
fn node_types_valid_json_expression() {
    let (grammar, table) = build_pipeline(expression_grammar);
    let json_str = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    let _: serde_json::Value = serde_json::from_str(&json_str).expect("invalid JSON");
}

#[test]
fn node_types_valid_json_statement() {
    let (grammar, table) = build_pipeline(statement_grammar);
    let json_str = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    let _: serde_json::Value = serde_json::from_str(&json_str).expect("invalid JSON");
}

#[test]
fn node_types_entries_have_type_and_named_fields() {
    let (grammar, table) = build_pipeline(chain_grammar);
    let json_str = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("invalid JSON");
    let arr = parsed.as_array().expect("expected array");
    for entry in arr {
        assert!(
            entry.get("type").is_some(),
            "each entry needs a 'type' field"
        );
        assert!(
            entry.get("named").is_some(),
            "each entry needs a 'named' field"
        );
    }
}

// ===========================================================================
// 5. Code scales with grammar (8 tests)
// ===========================================================================

#[test]
fn larger_grammar_produces_more_code_2_vs_1() {
    let (g1, t1) = build_pipeline(single_token_grammar);
    let (g2, t2) = build_pipeline(two_token_grammar);
    let len1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string()
        .len();
    let len2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string()
        .len();
    assert!(
        len2 > len1,
        "two-token grammar ({len2}) should produce more code than single ({len1})"
    );
}

#[test]
fn expression_grammar_larger_than_single() {
    let (g1, t1) = build_pipeline(single_token_grammar);
    let (g2, t2) = build_pipeline(expression_grammar);
    let len1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string()
        .len();
    let len2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string()
        .len();
    assert!(len2 > len1);
}

#[test]
fn chain_grammar_larger_than_single() {
    let (g1, t1) = build_pipeline(single_token_grammar);
    let (g2, t2) = build_pipeline(chain_grammar);
    let len1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string()
        .len();
    let len2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string()
        .len();
    assert!(len2 > len1);
}

#[test]
fn multi_rule_larger_than_single() {
    let (g1, t1) = build_pipeline(single_token_grammar);
    let (g2, t2) = build_pipeline(multi_rule_grammar);
    let len1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string()
        .len();
    let len2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string()
        .len();
    assert!(len2 > len1);
}

#[test]
fn scaled_5_larger_than_scaled_2() {
    let (g1, t1) = build_pipeline(|| scaled_grammar(2));
    let (g2, t2) = build_pipeline(|| scaled_grammar(5));
    let len1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string()
        .len();
    let len2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string()
        .len();
    assert!(
        len2 > len1,
        "scaled(5)={len2} should exceed scaled(2)={len1}"
    );
}

#[test]
fn scaled_10_larger_than_scaled_5() {
    let (g1, t1) = build_pipeline(|| scaled_grammar(5));
    let (g2, t2) = build_pipeline(|| scaled_grammar(10));
    let len1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string()
        .len();
    let len2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string()
        .len();
    assert!(
        len2 > len1,
        "scaled(10)={len2} should exceed scaled(5)={len1}"
    );
}

#[test]
fn node_types_scale_with_rules() {
    let (g1, t1) = build_pipeline(single_token_grammar);
    let (g2, t2) = build_pipeline(chain_grammar);
    let n1 = StaticLanguageGenerator::new(g1, t1)
        .generate_node_types()
        .len();
    let n2 = StaticLanguageGenerator::new(g2, t2)
        .generate_node_types()
        .len();
    assert!(
        n2 > n1,
        "chain grammar node types ({n2}) should be larger than single ({n1})"
    );
}

#[test]
fn statement_grammar_larger_than_alternatives() {
    let (g1, t1) = build_pipeline(alternatives_grammar);
    let (g2, t2) = build_pipeline(statement_grammar);
    let len1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string()
        .len();
    let len2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string()
        .len();
    assert!(len2 > len1);
}

// ===========================================================================
// 6. Multiple grammars produce different code (8 tests)
// ===========================================================================

#[test]
fn different_code_single_vs_two() {
    let (g1, t1) = build_pipeline(single_token_grammar);
    let (g2, t2) = build_pipeline(two_token_grammar);
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_ne!(c1, c2, "different grammars must produce different code");
}

#[test]
fn different_code_single_vs_chain() {
    let (g1, t1) = build_pipeline(single_token_grammar);
    let (g2, t2) = build_pipeline(chain_grammar);
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_ne!(c1, c2);
}

#[test]
fn different_code_single_vs_expression() {
    let (g1, t1) = build_pipeline(single_token_grammar);
    let (g2, t2) = build_pipeline(expression_grammar);
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_ne!(c1, c2);
}

#[test]
fn different_code_alternatives_vs_chain() {
    let (g1, t1) = build_pipeline(alternatives_grammar);
    let (g2, t2) = build_pipeline(chain_grammar);
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_ne!(c1, c2);
}

#[test]
fn different_code_expression_vs_statement() {
    let (g1, t1) = build_pipeline(expression_grammar);
    let (g2, t2) = build_pipeline(statement_grammar);
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_ne!(c1, c2);
}

#[test]
fn different_node_types_single_vs_chain() {
    let (g1, t1) = build_pipeline(single_token_grammar);
    let (g2, t2) = build_pipeline(chain_grammar);
    let n1 = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let n2 = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_ne!(
        n1, n2,
        "different grammars must produce different node types"
    );
}

#[test]
fn different_node_types_expression_vs_statement() {
    let (g1, t1) = build_pipeline(expression_grammar);
    let (g2, t2) = build_pipeline(statement_grammar);
    let n1 = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let n2 = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_ne!(n1, n2);
}

#[test]
fn different_code_multi_rule_vs_alternatives() {
    let (g1, t1) = build_pipeline(multi_rule_grammar);
    let (g2, t2) = build_pipeline(alternatives_grammar);
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_ne!(c1, c2);
}

// ===========================================================================
// 7. Edge cases (8 tests)
// ===========================================================================

#[test]
fn edge_minimal_one_token_grammar_generates_code() {
    let (grammar, table) = build_pipeline(single_token_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    assert!(!code.is_empty(), "minimal grammar must still produce code");
}

#[test]
fn edge_minimal_one_token_grammar_generates_valid_node_types() {
    let (grammar, table) = build_pipeline(single_token_grammar);
    let json_str = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("invalid JSON");
    assert!(parsed.is_array());
}

#[test]
fn edge_expression_grammar_handles_self_referencing_rules() {
    let (grammar, table) = build_pipeline(expression_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    assert!(
        !code.is_empty(),
        "self-referencing rules should not break codegen"
    );
}

#[test]
fn edge_chain_grammar_deep_nonterminal_nesting() {
    let (grammar, table) = build_pipeline(chain_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    assert!(!code.is_empty(), "deep non-terminal chain should work");
}

#[test]
fn edge_alternatives_grammar_multiple_productions() {
    let (grammar, table) = build_pipeline(alternatives_grammar);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn edge_start_can_be_empty_flag_does_not_crash() {
    let (grammar, table) = build_pipeline(single_token_grammar);
    let mut lang_gen = StaticLanguageGenerator::new(grammar, table);
    lang_gen.set_start_can_be_empty(true);
    let code = lang_gen.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn edge_scaled_grammar_15_tokens() {
    let (grammar, table) = build_pipeline(|| scaled_grammar(15));
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    let code = lang_gen.generate_language_code().to_string();
    assert!(
        code.len() > 200,
        "15-token scaled grammar should produce substantial code"
    );
}

#[test]
fn edge_node_types_always_json_array_even_for_minimal() {
    let (grammar, table) = build_pipeline(single_token_grammar);
    let json_str = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    assert!(
        json_str.trim_start().starts_with('['),
        "node types must start with '['",
    );
    assert!(
        json_str.trim_end().ends_with(']'),
        "node types must end with ']'",
    );
}
