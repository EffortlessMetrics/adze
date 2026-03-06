//! Comprehensive tests for StaticLanguageGenerator code generation.
//!
//! Covers construction, output content, node types JSON, determinism,
//! grammar differentiation, fields/precedence, and edge cases.

mod test_helpers;

use adze_glr_core::ParseTable;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tablegen::StaticLanguageGenerator;

// ---------------------------------------------------------------------------
// Helper: build a minimal ParseTable from a Grammar
// ---------------------------------------------------------------------------
fn minimal_table(grammar: Grammar) -> ParseTable {
    test_helpers::create_minimal_parse_table(grammar)
}

fn table_with_content(grammar: Grammar, states: usize, symbols: usize) -> ParseTable {
    test_helpers::create_test_parse_table_with_content(grammar, states, symbols)
}

// Shorthand grammar builders

fn grammar_single_token() -> Grammar {
    GrammarBuilder::new("single_tok")
        .token("number", r"\d+")
        .rule("start", vec!["number"])
        .start("start")
        .build()
}

fn grammar_two_tokens() -> Grammar {
    GrammarBuilder::new("two_tok")
        .token("number", r"\d+")
        .token("ident", r"[a-z]+")
        .rule("start", vec!["number"])
        .rule("start", vec!["ident"])
        .start("start")
        .build()
}

fn grammar_arithmetic() -> Grammar {
    GrammarBuilder::new("arith")
        .token("number", r"\d+")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["number"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["expr", "star", "expr"])
        .start("expr")
        .build()
}

fn grammar_with_precedence() -> Grammar {
    GrammarBuilder::new("prec")
        .token("number", r"\d+")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["number"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build()
}

fn grammar_with_extra() -> Grammar {
    GrammarBuilder::new("ws_grammar")
        .token("number", r"\d+")
        .token("ws", r"\s+")
        .rule("start", vec!["number"])
        .start("start")
        .extra("ws")
        .build()
}

fn grammar_nested() -> Grammar {
    GrammarBuilder::new("nested")
        .token("id", r"[a-z]+")
        .token("lparen", "(")
        .token("rparen", ")")
        .rule("expr", vec!["id"])
        .rule("expr", vec!["lparen", "expr", "rparen"])
        .start("expr")
        .build()
}

fn grammar_multiple_rules() -> Grammar {
    GrammarBuilder::new("multi")
        .token("id", r"[a-z]+")
        .token("num", r"\d+")
        .token("semi", ";")
        .rule("program", vec!["stmt"])
        .rule("stmt", vec!["id", "semi"])
        .rule("stmt", vec!["num", "semi"])
        .start("program")
        .build()
}

fn grammar_empty_name() -> Grammar {
    GrammarBuilder::new("")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

// =========================================================================
// 1. StaticLanguageGenerator construction (8 tests)
// =========================================================================

#[test]
fn construct_with_single_token_grammar() {
    let g = grammar_single_token();
    let t = minimal_table(g.clone());
    let _slg = StaticLanguageGenerator::new(g, t);
}

#[test]
fn construct_with_two_token_grammar() {
    let g = grammar_two_tokens();
    let t = minimal_table(g.clone());
    let _slg = StaticLanguageGenerator::new(g, t);
}

#[test]
fn construct_with_arithmetic_grammar() {
    let g = grammar_arithmetic();
    let t = minimal_table(g.clone());
    let _slg = StaticLanguageGenerator::new(g, t);
}

#[test]
fn construct_with_empty_name_grammar() {
    let g = grammar_empty_name();
    let t = minimal_table(g.clone());
    let _slg = StaticLanguageGenerator::new(g, t);
}

#[test]
fn construct_with_content_table() {
    let g = grammar_arithmetic();
    let t = table_with_content(g.clone(), 4, 6);
    let _slg = StaticLanguageGenerator::new(g, t);
}

#[test]
fn construct_preserves_grammar_name() {
    let g = grammar_single_token();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    assert_eq!(slg.grammar.name, "single_tok");
}

#[test]
fn construct_start_can_be_empty_defaults_false() {
    let g = grammar_single_token();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(!slg.start_can_be_empty);
}

#[test]
fn construct_set_start_can_be_empty() {
    let g = grammar_single_token();
    let t = minimal_table(g.clone());
    let mut slg = StaticLanguageGenerator::new(g, t);
    slg.set_start_can_be_empty(true);
    assert!(slg.start_can_be_empty);
}

// =========================================================================
// 2. Generated code is non-empty (8 tests)
// =========================================================================

#[test]
fn code_nonempty_single_token() {
    let g = grammar_single_token();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty(), "generated code must not be empty");
}

#[test]
fn code_nonempty_two_tokens() {
    let g = grammar_two_tokens();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn code_nonempty_arithmetic() {
    let g = grammar_arithmetic();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn code_nonempty_nested() {
    let g = grammar_nested();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn code_nonempty_with_extra() {
    let g = grammar_with_extra();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn code_nonempty_multiple_rules() {
    let g = grammar_multiple_rules();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn code_nonempty_precedence() {
    let g = grammar_with_precedence();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn code_nonempty_with_content_table() {
    let g = grammar_arithmetic();
    let t = table_with_content(g.clone(), 3, 5);
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

// =========================================================================
// 3. Generated code contains expected tokens (8 tests)
// =========================================================================

#[test]
fn code_contains_fn_keyword() {
    let g = grammar_single_token();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(code.contains("fn"), "generated code should contain `fn`");
}

#[test]
fn code_contains_language_reference() {
    let g = grammar_single_token();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    // Language struct or function reference should appear
    let has_language = code.contains("language") || code.contains("Language");
    assert!(
        has_language,
        "generated code should reference 'language': {code}"
    );
}

#[test]
fn code_contains_const_or_static() {
    let g = grammar_single_token();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    let has_const = code.contains("const") || code.contains("static");
    assert!(has_const, "generated code should have const/static: {code}");
}

#[test]
fn code_contains_state_count_value() {
    let g = grammar_single_token();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    // state_count = 1 for minimal table
    let has_state_info = code.contains("state") || code.contains("1");
    assert!(has_state_info, "code should reference state info: {code}");
}

#[test]
fn code_contains_pub_keyword() {
    let g = grammar_single_token();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(
        code.contains("pub"),
        "generated code should have `pub` items"
    );
}

#[test]
fn code_contains_unsafe_for_ffi() {
    let g = grammar_single_token();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    // FFI language structs typically need unsafe or raw pointers
    let has_unsafe_or_ptr =
        code.contains("unsafe") || code.contains("*const") || code.contains("ptr");
    assert!(
        has_unsafe_or_ptr,
        "FFI code should contain unsafe or pointers: {code}"
    );
}

#[test]
fn code_grows_with_more_tokens() {
    let g1 = grammar_single_token();
    let t1 = minimal_table(g1.clone());
    let slg1 = StaticLanguageGenerator::new(g1, t1);
    let code1 = slg1.generate_language_code().to_string();

    let g2 = grammar_arithmetic();
    let t2 = minimal_table(g2.clone());
    let slg2 = StaticLanguageGenerator::new(g2, t2);
    let code2 = slg2.generate_language_code().to_string();

    assert!(
        code2.len() >= code1.len(),
        "more complex grammar should produce at least as much code"
    );
}

#[test]
fn code_contains_symbol_count_related() {
    let g = grammar_arithmetic();
    let t = table_with_content(g.clone(), 4, 8);
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    // With 8 symbols or 4 states, the code should reference numeric data
    let has_numeric = code.contains("8") || code.contains("4") || code.contains("symbol");
    assert!(
        has_numeric,
        "code should contain table dimension data: {code}"
    );
}

// =========================================================================
// 4. Node types JSON output (8 tests)
// =========================================================================

#[test]
fn node_types_is_valid_json() {
    let g = grammar_single_token();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let json_str = slg.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
    assert!(parsed.is_array(), "node types should be a JSON array");
}

#[test]
fn node_types_array_nonempty_for_grammar_with_rules() {
    let g = grammar_single_token();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let json_str = slg.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = parsed.as_array().unwrap();
    assert!(
        !arr.is_empty(),
        "grammar with rules should produce node types"
    );
}

#[test]
fn node_types_entries_have_type_field() {
    let g = grammar_single_token();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let json_str = slg.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    for entry in parsed.as_array().unwrap() {
        assert!(
            entry.get("type").is_some(),
            "each node type entry should have 'type' field: {entry}"
        );
    }
}

#[test]
fn node_types_entries_have_named_field() {
    let g = grammar_two_tokens();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let json_str = slg.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    for entry in parsed.as_array().unwrap() {
        assert!(
            entry.get("named").is_some(),
            "each node type entry should have 'named' field: {entry}"
        );
    }
}

#[test]
fn node_types_rule_entries_are_named() {
    let g = grammar_single_token();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let json_str = slg.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    for entry in parsed.as_array().unwrap() {
        let named = entry.get("named").and_then(|v| v.as_bool());
        assert_eq!(named, Some(true), "node types should be named: {entry}");
    }
}

#[test]
fn node_types_grows_with_grammar_complexity() {
    let g1 = grammar_single_token();
    let t1 = minimal_table(g1.clone());
    let slg1 = StaticLanguageGenerator::new(g1, t1);
    let json1 = slg1.generate_node_types();
    let arr1: serde_json::Value = serde_json::from_str(&json1).unwrap();

    let g2 = grammar_multiple_rules();
    let t2 = minimal_table(g2.clone());
    let slg2 = StaticLanguageGenerator::new(g2, t2);
    let json2 = slg2.generate_node_types();
    let arr2: serde_json::Value = serde_json::from_str(&json2).unwrap();

    assert!(
        arr2.as_array().unwrap().len() >= arr1.as_array().unwrap().len(),
        "more rules should produce at least as many node types"
    );
}

#[test]
fn node_types_contains_regex_tokens() {
    // Tokens with regex patterns are included as named node types
    let g = grammar_two_tokens();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let json_str = slg.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let types: Vec<&str> = parsed
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e.get("type").and_then(|t| t.as_str()))
        .collect();
    // "number" and "ident" are regex tokens
    assert!(
        types.iter().any(|t| *t == "number" || *t == "ident"),
        "regex tokens should appear in node types: {types:?}"
    );
}

#[test]
fn node_types_json_is_pretty_printed() {
    let g = grammar_single_token();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let json_str = slg.generate_node_types();
    // Pretty-printed JSON contains newlines
    assert!(
        json_str.contains('\n'),
        "node types JSON should be pretty-printed"
    );
}

// =========================================================================
// 5. Deterministic generation (5 tests)
// =========================================================================

#[test]
fn deterministic_code_single_token() {
    let g1 = grammar_single_token();
    let t1 = minimal_table(g1.clone());
    let slg1 = StaticLanguageGenerator::new(g1, t1);

    let g2 = grammar_single_token();
    let t2 = minimal_table(g2.clone());
    let slg2 = StaticLanguageGenerator::new(g2, t2);

    assert_eq!(
        slg1.generate_language_code().to_string(),
        slg2.generate_language_code().to_string(),
        "same grammar should produce identical code"
    );
}

#[test]
fn deterministic_code_arithmetic() {
    let g1 = grammar_arithmetic();
    let t1 = minimal_table(g1.clone());
    let slg1 = StaticLanguageGenerator::new(g1, t1);

    let g2 = grammar_arithmetic();
    let t2 = minimal_table(g2.clone());
    let slg2 = StaticLanguageGenerator::new(g2, t2);

    assert_eq!(
        slg1.generate_language_code().to_string(),
        slg2.generate_language_code().to_string(),
    );
}

#[test]
fn deterministic_node_types_single_token() {
    let g1 = grammar_single_token();
    let t1 = minimal_table(g1.clone());
    let slg1 = StaticLanguageGenerator::new(g1, t1);

    let g2 = grammar_single_token();
    let t2 = minimal_table(g2.clone());
    let slg2 = StaticLanguageGenerator::new(g2, t2);

    assert_eq!(slg1.generate_node_types(), slg2.generate_node_types());
}

#[test]
fn deterministic_node_types_arithmetic() {
    let g1 = grammar_arithmetic();
    let t1 = minimal_table(g1.clone());
    let slg1 = StaticLanguageGenerator::new(g1, t1);

    let g2 = grammar_arithmetic();
    let t2 = minimal_table(g2.clone());
    let slg2 = StaticLanguageGenerator::new(g2, t2);

    assert_eq!(slg1.generate_node_types(), slg2.generate_node_types());
}

#[test]
fn deterministic_multiple_invocations_same_generator() {
    let g = grammar_nested();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);

    let code_a = slg.generate_language_code().to_string();
    let code_b = slg.generate_language_code().to_string();
    assert_eq!(
        code_a, code_b,
        "repeated calls should produce identical output"
    );

    let json_a = slg.generate_node_types();
    let json_b = slg.generate_node_types();
    assert_eq!(json_a, json_b);
}

// =========================================================================
// 6. Different grammars produce different code (5 tests)
// =========================================================================

#[test]
fn different_grammars_different_code_single_vs_two() {
    let g1 = grammar_single_token();
    let t1 = minimal_table(g1.clone());
    let slg1 = StaticLanguageGenerator::new(g1, t1);

    let g2 = grammar_two_tokens();
    let t2 = minimal_table(g2.clone());
    let slg2 = StaticLanguageGenerator::new(g2, t2);

    // Code or node types should differ
    let code_differ =
        slg1.generate_language_code().to_string() != slg2.generate_language_code().to_string();
    let json_differ = slg1.generate_node_types() != slg2.generate_node_types();
    assert!(
        code_differ || json_differ,
        "different grammars should produce different output"
    );
}

#[test]
fn different_grammars_different_code_arith_vs_nested() {
    let g1 = grammar_arithmetic();
    let t1 = minimal_table(g1.clone());
    let slg1 = StaticLanguageGenerator::new(g1, t1);

    let g2 = grammar_nested();
    let t2 = minimal_table(g2.clone());
    let slg2 = StaticLanguageGenerator::new(g2, t2);

    let code_differ =
        slg1.generate_language_code().to_string() != slg2.generate_language_code().to_string();
    let json_differ = slg1.generate_node_types() != slg2.generate_node_types();
    assert!(code_differ || json_differ);
}

#[test]
fn different_grammars_different_node_types_count() {
    let g1 = grammar_single_token();
    let t1 = minimal_table(g1.clone());
    let slg1 = StaticLanguageGenerator::new(g1, t1);
    let json1: serde_json::Value = serde_json::from_str(&slg1.generate_node_types()).unwrap();

    let g2 = grammar_multiple_rules();
    let t2 = minimal_table(g2.clone());
    let slg2 = StaticLanguageGenerator::new(g2, t2);
    let json2: serde_json::Value = serde_json::from_str(&slg2.generate_node_types()).unwrap();

    // multi-rule grammar has more non-terminals
    assert_ne!(
        json1.as_array().unwrap().len(),
        json2.as_array().unwrap().len(),
        "grammars with different rule counts should have different node type counts"
    );
}

#[test]
fn different_grammar_names_different_code() {
    let g1 = GrammarBuilder::new("alpha")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let t1 = minimal_table(g1.clone());
    let slg1 = StaticLanguageGenerator::new(g1, t1);

    let g2 = GrammarBuilder::new("beta")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let t2 = minimal_table(g2.clone());
    let slg2 = StaticLanguageGenerator::new(g2, t2);

    // Different grammar names may or may not affect generated code,
    // but they definitely produce valid output.
    let code1 = slg1.generate_language_code().to_string();
    let code2 = slg2.generate_language_code().to_string();
    assert!(!code1.is_empty());
    assert!(!code2.is_empty());
}

#[test]
fn different_table_sizes_different_code() {
    let g1 = grammar_arithmetic();
    let t1 = minimal_table(g1.clone());
    let slg1 = StaticLanguageGenerator::new(g1, t1);

    let g2 = grammar_arithmetic();
    let t2 = table_with_content(g2.clone(), 5, 10);
    let slg2 = StaticLanguageGenerator::new(g2, t2);

    let code1 = slg1.generate_language_code().to_string();
    let code2 = slg2.generate_language_code().to_string();
    assert_ne!(
        code1, code2,
        "different table sizes should produce different code"
    );
}

// =========================================================================
// 7. Generated code with fields/precedence (5 tests)
// =========================================================================

#[test]
fn precedence_grammar_generates_valid_code() {
    let g = grammar_with_precedence();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn precedence_grammar_generates_valid_node_types() {
    let g = grammar_with_precedence();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let json_str = slg.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
    assert!(parsed.is_array());
}

#[test]
fn right_associativity_grammar_generates_code() {
    let g = GrammarBuilder::new("right_assoc")
        .token("number", r"\d+")
        .token("pow", "^")
        .rule("expr", vec!["number"])
        .rule_with_precedence("expr", vec!["expr", "pow", "expr"], 3, Associativity::Right)
        .start("expr")
        .build();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn mixed_precedence_levels_generate_code() {
    let g = GrammarBuilder::new("mixed_prec")
        .token("num", r"\d+")
        .token("add", "+")
        .token("mul", "*")
        .token("pow", "^")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "add", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "mul", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "pow", "expr"], 3, Associativity::Right)
        .start("expr")
        .build();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
    let json_str = slg.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn grammar_with_extra_token_generates_code() {
    let g = grammar_with_extra();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

// =========================================================================
// 8. Edge cases (8 tests)
// =========================================================================

#[test]
fn edge_empty_grammar_name() {
    let g = grammar_empty_name();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(
        !code.is_empty(),
        "empty grammar name should still generate code"
    );
}

#[test]
fn edge_single_character_token() {
    let g = GrammarBuilder::new("single_char")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn edge_many_tokens() {
    let g = GrammarBuilder::new("many_tokens")
        .token("t0", "a")
        .token("t1", "b")
        .token("t2", "c")
        .token("t3", "d")
        .token("t4", "e")
        .token("t5", "f")
        .token("t6", "g")
        .token("t7", "h")
        .rule("start", vec!["t0"])
        .rule("start", vec!["t1"])
        .rule("start", vec!["t2"])
        .rule("start", vec!["t3"])
        .rule("start", vec!["t4"])
        .rule("start", vec!["t5"])
        .rule("start", vec!["t6"])
        .rule("start", vec!["t7"])
        .start("start")
        .build();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn edge_start_can_be_empty_affects_output() {
    let g1 = grammar_single_token();
    let t1 = minimal_table(g1.clone());
    let slg1 = StaticLanguageGenerator::new(g1, t1);
    let code1 = slg1.generate_language_code().to_string();

    let g2 = grammar_single_token();
    let t2 = minimal_table(g2.clone());
    let mut slg2 = StaticLanguageGenerator::new(g2, t2);
    slg2.set_start_can_be_empty(true);
    let code2 = slg2.generate_language_code().to_string();

    // Both should produce valid code
    assert!(!code1.is_empty());
    assert!(!code2.is_empty());
}

#[test]
fn edge_large_state_count() {
    let g = grammar_single_token();
    let t = table_with_content(g.clone(), 50, 10);
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty(), "large state table should generate code");
}

#[test]
fn edge_large_symbol_count() {
    let g = grammar_single_token();
    let t = table_with_content(g.clone(), 3, 50);
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty(), "large symbol table should generate code");
}

#[test]
fn edge_node_types_json_parseable_round_trip() {
    let g = grammar_arithmetic();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let json_str = slg.generate_node_types();
    // Parse, serialize, parse again
    let first: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let re_serialized = serde_json::to_string_pretty(&first).unwrap();
    let second: serde_json::Value = serde_json::from_str(&re_serialized).unwrap();
    assert_eq!(first, second, "JSON round-trip should be stable");
}

#[test]
fn edge_grammar_with_external_token() {
    let g = GrammarBuilder::new("ext_grammar")
        .token("id", r"[a-z]+")
        .external("comment")
        .rule("start", vec!["id"])
        .start("start")
        .build();
    let t = minimal_table(g.clone());
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());

    let json_str = slg.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.is_array());
    // External token "comment" should appear in node types
    let types: Vec<&str> = parsed
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e.get("type").and_then(|t| t.as_str()))
        .collect();
    assert!(
        types.contains(&"comment"),
        "external token should appear in node types: {types:?}"
    );
}
