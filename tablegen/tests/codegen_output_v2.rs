//! Comprehensive tests for adze-tablegen code generation output.
//!
//! Covers: non-empty output, language struct presence, node types JSON validity,
//! token name presence in node types, determinism, scaling, and NodeTypesGenerator.

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::{NodeTypesGenerator, StaticLanguageGenerator};
use serde_json::Value;

// =====================================================================
// Helpers
// =====================================================================

fn build_pipeline(grammar_builder: GrammarBuilder) -> (Grammar, ParseTable) {
    let mut grammar = grammar_builder.build();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).expect("FIRST/FOLLOW");
    let table = build_lr1_automaton(&grammar, &ff).expect("LR(1)");
    (grammar, table)
}

fn expr_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("expr_lang")
            .token("NUM", r"\d+")
            .token("PLUS", r"\+")
            .rule("expr", vec!["NUM", "PLUS", "NUM"])
            .start("expr"),
    )
}

fn single_token_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("single")
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s"),
    )
}

fn two_alt_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("two_alt")
            .token("x", "x")
            .token("y", "y")
            .rule("s", vec!["x"])
            .rule("s", vec!["y"])
            .start("s"),
    )
}

fn chain_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("chain")
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a", "b"])
            .start("s"),
    )
}

fn nested_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("nested")
            .token("a", "a")
            .token("b", "b")
            .rule("inner", vec!["a"])
            .rule("outer", vec!["inner", "b"])
            .start("outer"),
    )
}

fn keyword_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("keywords")
            .token("IF", "if")
            .token("THEN", "then")
            .token("ID", r"[a-z]+")
            .rule("stmt", vec!["IF", "ID", "THEN", "ID"])
            .start("stmt"),
    )
}

fn multi_rule_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("multi")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("s", vec!["a", "m"])
            .rule("m", vec!["b", "c"])
            .start("s"),
    )
}

fn large_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("large")
            .token("t1", "t1")
            .token("t2", "t2")
            .token("t3", "t3")
            .token("t4", "t4")
            .token("t5", "t5")
            .rule("s", vec!["r1"])
            .rule("r1", vec!["t1", "r2"])
            .rule("r2", vec!["t2", "r3"])
            .rule("r3", vec!["t3", "r4"])
            .rule("r4", vec!["t4", "t5"])
            .start("s"),
    )
}

// =====================================================================
// 1. Generated code is non-empty (8 tests)
// =====================================================================

#[test]
fn test_nonempty_single_token() {
    let (grammar, table) = single_token_grammar();
    let codegen = StaticLanguageGenerator::new(grammar, table);
    assert!(!codegen.generate_language_code().is_empty());
}

#[test]
fn test_nonempty_expr() {
    let (grammar, table) = expr_grammar();
    let codegen = StaticLanguageGenerator::new(grammar, table);
    assert!(!codegen.generate_language_code().is_empty());
}

#[test]
fn test_nonempty_two_alt() {
    let (grammar, table) = two_alt_grammar();
    let codegen = StaticLanguageGenerator::new(grammar, table);
    assert!(!codegen.generate_language_code().is_empty());
}

#[test]
fn test_nonempty_chain() {
    let (grammar, table) = chain_grammar();
    let codegen = StaticLanguageGenerator::new(grammar, table);
    assert!(!codegen.generate_language_code().is_empty());
}

#[test]
fn test_nonempty_nested() {
    let (grammar, table) = nested_grammar();
    let codegen = StaticLanguageGenerator::new(grammar, table);
    assert!(!codegen.generate_language_code().is_empty());
}

#[test]
fn test_nonempty_keyword() {
    let (grammar, table) = keyword_grammar();
    let codegen = StaticLanguageGenerator::new(grammar, table);
    assert!(!codegen.generate_language_code().is_empty());
}

#[test]
fn test_nonempty_multi_rule() {
    let (grammar, table) = multi_rule_grammar();
    let codegen = StaticLanguageGenerator::new(grammar, table);
    assert!(!codegen.generate_language_code().is_empty());
}

#[test]
fn test_nonempty_large() {
    let (grammar, table) = large_grammar();
    let codegen = StaticLanguageGenerator::new(grammar, table);
    assert!(!codegen.generate_language_code().is_empty());
}

// =====================================================================
// 2. Generated code contains language struct (8 tests)
// =====================================================================

#[test]
fn test_contains_language_single_token() {
    let (grammar, table) = single_token_grammar();
    let code_str = StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string();
    assert!(
        code_str.contains("Language") || code_str.contains("LANGUAGE"),
        "Expected 'Language' or 'LANGUAGE' in output"
    );
}

#[test]
fn test_contains_language_expr() {
    let (grammar, table) = expr_grammar();
    let code_str = StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string();
    assert!(
        code_str.contains("Language") || code_str.contains("LANGUAGE"),
        "Expected 'Language' or 'LANGUAGE' in output"
    );
}

#[test]
fn test_contains_language_two_alt() {
    let (grammar, table) = two_alt_grammar();
    let code_str = StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string();
    assert!(
        code_str.contains("Language") || code_str.contains("LANGUAGE"),
        "Expected language reference in output"
    );
}

#[test]
fn test_contains_language_chain() {
    let (grammar, table) = chain_grammar();
    let code_str = StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string();
    assert!(
        code_str.contains("Language") || code_str.contains("LANGUAGE"),
        "Expected language reference in output"
    );
}

#[test]
fn test_contains_language_nested() {
    let (grammar, table) = nested_grammar();
    let code_str = StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string();
    assert!(
        code_str.contains("Language") || code_str.contains("LANGUAGE"),
        "Expected language reference in output"
    );
}

#[test]
fn test_contains_language_keyword() {
    let (grammar, table) = keyword_grammar();
    let code_str = StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string();
    assert!(
        code_str.contains("Language") || code_str.contains("LANGUAGE"),
        "Expected language reference in output"
    );
}

#[test]
fn test_contains_language_multi_rule() {
    let (grammar, table) = multi_rule_grammar();
    let code_str = StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string();
    assert!(
        code_str.contains("Language") || code_str.contains("LANGUAGE"),
        "Expected language reference in output"
    );
}

#[test]
fn test_contains_language_large() {
    let (grammar, table) = large_grammar();
    let code_str = StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string();
    assert!(
        code_str.contains("Language") || code_str.contains("LANGUAGE"),
        "Expected language reference in output"
    );
}

// =====================================================================
// 3. Node types is valid JSON (8 tests)
// =====================================================================

fn assert_valid_json(json_str: &str) {
    serde_json::from_str::<Value>(json_str)
        .expect("generate_node_types() should produce valid JSON");
}

#[test]
fn test_node_types_json_single_token() {
    let (grammar, table) = single_token_grammar();
    let json = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    assert_valid_json(&json);
}

#[test]
fn test_node_types_json_expr() {
    let (grammar, table) = expr_grammar();
    let json = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    assert_valid_json(&json);
}

#[test]
fn test_node_types_json_two_alt() {
    let (grammar, table) = two_alt_grammar();
    let json = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    assert_valid_json(&json);
}

#[test]
fn test_node_types_json_chain() {
    let (grammar, table) = chain_grammar();
    let json = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    assert_valid_json(&json);
}

#[test]
fn test_node_types_json_nested() {
    let (grammar, table) = nested_grammar();
    let json = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    assert_valid_json(&json);
}

#[test]
fn test_node_types_json_keyword() {
    let (grammar, table) = keyword_grammar();
    let json = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    assert_valid_json(&json);
}

#[test]
fn test_node_types_json_multi_rule() {
    let (grammar, table) = multi_rule_grammar();
    let json = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    assert_valid_json(&json);
}

#[test]
fn test_node_types_json_large() {
    let (grammar, table) = large_grammar();
    let json = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    assert_valid_json(&json);
}

// =====================================================================
// 4. Node types contain grammar info (7 tests)
//
// Note: generate_node_types() emits rules as "rule_N" and only includes
// tokens with regex patterns (TokenPattern::Regex), not string literals.
// =====================================================================

#[test]
fn test_node_types_contain_rule_entries() {
    let (grammar, table) = single_token_grammar();
    let json = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    assert!(
        json.contains("rule_"),
        "Expected rule entries in node types JSON"
    );
}

#[test]
fn test_node_types_mention_regex_tokens() {
    let (grammar, table) = expr_grammar();
    let json = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    assert!(
        json.contains("NUM") || json.contains("PLUS"),
        "Expected regex token names NUM or PLUS in node types JSON"
    );
}

#[test]
fn test_node_types_has_named_entries() {
    let (grammar, table) = two_alt_grammar();
    let json = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    assert!(
        json.contains("\"named\""),
        "Expected 'named' field in node types JSON"
    );
}

#[test]
fn test_node_types_chain_has_rule_entries() {
    let (grammar, table) = chain_grammar();
    let json = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    assert!(
        json.contains("rule_"),
        "Expected rule entries in chain grammar node types"
    );
}

#[test]
fn test_node_types_mention_keyword_regex_token() {
    let (grammar, table) = keyword_grammar();
    let json = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    assert!(
        json.contains("ID"),
        "Expected regex token ID in node types JSON"
    );
}

#[test]
fn test_node_types_nested_has_type_field() {
    let (grammar, table) = nested_grammar();
    let json = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    assert!(
        json.contains("\"type\""),
        "Expected 'type' field in nested grammar node types"
    );
}

#[test]
fn test_node_types_multi_rule_has_multiple_entries() {
    let (grammar, table) = multi_rule_grammar();
    let json = StaticLanguageGenerator::new(grammar, table).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().expect("node types should be an array");
    assert!(
        arr.len() > 1,
        "Multi-rule grammar should produce multiple node type entries"
    );
}

// =====================================================================
// 5. Code determinism (8 tests)
// =====================================================================

#[test]
fn test_determinism_single_token() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = single_token_grammar();
    let a = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let b = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(a, b, "Same grammar must produce identical code");
}

#[test]
fn test_determinism_expr() {
    let (g1, t1) = expr_grammar();
    let (g2, t2) = expr_grammar();
    let a = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let b = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(a, b);
}

#[test]
fn test_determinism_two_alt() {
    let (g1, t1) = two_alt_grammar();
    let (g2, t2) = two_alt_grammar();
    let a = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let b = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(a, b);
}

#[test]
fn test_determinism_chain() {
    let (g1, t1) = chain_grammar();
    let (g2, t2) = chain_grammar();
    let a = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let b = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(a, b);
}

#[test]
fn test_determinism_nested() {
    let (g1, t1) = nested_grammar();
    let (g2, t2) = nested_grammar();
    let a = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let b = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(a, b);
}

#[test]
fn test_determinism_keyword() {
    let (g1, t1) = keyword_grammar();
    let (g2, t2) = keyword_grammar();
    let a = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let b = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(a, b);
}

#[test]
fn test_determinism_multi_rule() {
    let (g1, t1) = multi_rule_grammar();
    let (g2, t2) = multi_rule_grammar();
    let a = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let b = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(a, b);
}

#[test]
fn test_determinism_node_types() {
    let (g1, t1) = expr_grammar();
    let (g2, t2) = expr_grammar();
    let a = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let b = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_eq!(a, b, "Node types must be deterministic");
}

// =====================================================================
// 6. Code scales with grammar (8 tests)
// =====================================================================

#[test]
fn test_scale_more_rules_more_code() {
    let (g_small, t_small) = single_token_grammar();
    let (g_large, t_large) = large_grammar();
    let small_len = StaticLanguageGenerator::new(g_small, t_small)
        .generate_language_code()
        .to_string()
        .len();
    let large_len = StaticLanguageGenerator::new(g_large, t_large)
        .generate_language_code()
        .to_string()
        .len();
    assert!(
        large_len > small_len,
        "Larger grammar should produce more code: {large_len} <= {small_len}"
    );
}

#[test]
fn test_scale_chain_vs_single() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = chain_grammar();
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
        "Chain grammar should produce more code than single token"
    );
}

#[test]
fn test_scale_nested_vs_single() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = nested_grammar();
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
        "Nested grammar should produce more code than single token"
    );
}

#[test]
fn test_scale_keyword_vs_single() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = keyword_grammar();
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
        "Keyword grammar should produce more code than single token"
    );
}

#[test]
fn test_scale_node_types_more_rules_larger() {
    let (g_small, t_small) = single_token_grammar();
    let (g_large, t_large) = large_grammar();
    let small_len = StaticLanguageGenerator::new(g_small, t_small)
        .generate_node_types()
        .len();
    let large_len = StaticLanguageGenerator::new(g_large, t_large)
        .generate_node_types()
        .len();
    assert!(
        large_len > small_len,
        "Larger grammar should produce larger node types JSON"
    );
}

#[test]
fn test_scale_multi_rule_vs_single() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = multi_rule_grammar();
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
        "Multi-rule grammar should produce more code than single token"
    );
}

#[test]
fn test_scale_expr_vs_single() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = expr_grammar();
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
        "Expr grammar should produce more code than single token"
    );
}

#[test]
fn test_scale_two_alt_vs_single() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = two_alt_grammar();
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
        "Two-alt grammar should produce more code than single token"
    );
}

// =====================================================================
// 7. NodeTypesGenerator (8 tests)
// =====================================================================

#[test]
fn test_ntg_single_token_valid() {
    let mut grammar = GrammarBuilder::new("ntg_single")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let output = NodeTypesGenerator::new(&grammar).generate().unwrap();
    assert!(!output.is_empty());
}

#[test]
fn test_ntg_single_token_json() {
    let mut grammar = GrammarBuilder::new("ntg_json")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let output = NodeTypesGenerator::new(&grammar).generate().unwrap();
    serde_json::from_str::<Value>(&output).expect("NodeTypesGenerator should produce valid JSON");
}

#[test]
fn test_ntg_expr_contains_rule_type() {
    let mut grammar = GrammarBuilder::new("ntg_expr")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["NUM", "PLUS", "NUM"])
        .start("expr")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let output = NodeTypesGenerator::new(&grammar).generate().unwrap();
    assert!(output.contains("expr"), "Should reference rule 'expr'");
}

#[test]
fn test_ntg_nested_contains_rules() {
    let mut grammar = GrammarBuilder::new("ntg_nested")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["a"])
        .rule("outer", vec!["inner", "b"])
        .start("outer")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let output = NodeTypesGenerator::new(&grammar).generate().unwrap();
    assert!(
        output.contains("inner") || output.contains("outer"),
        "Should reference grammar rules"
    );
}

#[test]
fn test_ntg_determinism() {
    let mut g1 = GrammarBuilder::new("ntg_det")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let _ff1 = FirstFollowSets::compute_normalized(&mut g1).unwrap();
    let out1 = NodeTypesGenerator::new(&g1).generate().unwrap();

    let mut g2 = GrammarBuilder::new("ntg_det")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let _ff2 = FirstFollowSets::compute_normalized(&mut g2).unwrap();
    let out2 = NodeTypesGenerator::new(&g2).generate().unwrap();

    assert_eq!(out1, out2, "Same grammar should yield identical output");
}

#[test]
fn test_ntg_multi_rule_valid_json() {
    let mut grammar = GrammarBuilder::new("ntg_multi")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "m"])
        .rule("m", vec!["b", "c"])
        .start("s")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let output = NodeTypesGenerator::new(&grammar).generate().unwrap();
    serde_json::from_str::<Value>(&output).expect("Multi-rule grammar should produce valid JSON");
}

#[test]
fn test_ntg_keyword_grammar() {
    let mut grammar = GrammarBuilder::new("ntg_kw")
        .token("IF", "if")
        .token("THEN", "then")
        .token("ID", r"[a-z]+")
        .rule("stmt", vec!["IF", "ID", "THEN", "ID"])
        .start("stmt")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let output = NodeTypesGenerator::new(&grammar).generate().unwrap();
    assert!(!output.is_empty(), "Keyword grammar should generate output");
    serde_json::from_str::<Value>(&output).expect("Should be valid JSON");
}

#[test]
fn test_ntg_large_grammar_scales() {
    let mut g_small = GrammarBuilder::new("ntg_sm")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let _ff_small = FirstFollowSets::compute_normalized(&mut g_small).unwrap();
    let small_out = NodeTypesGenerator::new(&g_small).generate().unwrap();

    let mut g_large = GrammarBuilder::new("ntg_lg")
        .token("t1", "t1")
        .token("t2", "t2")
        .token("t3", "t3")
        .token("t4", "t4")
        .rule("s", vec!["r1"])
        .rule("r1", vec!["t1", "r2"])
        .rule("r2", vec!["t2", "r3"])
        .rule("r3", vec!["t3", "t4"])
        .start("s")
        .build();
    let _ff_large = FirstFollowSets::compute_normalized(&mut g_large).unwrap();
    let large_out = NodeTypesGenerator::new(&g_large).generate().unwrap();

    assert!(
        large_out.len() > small_out.len(),
        "Larger grammar should produce larger node types output"
    );
}
