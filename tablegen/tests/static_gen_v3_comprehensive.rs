//! Comprehensive v3 tests for `StaticLanguageGenerator` in adze-tablegen.
//!
//! 55+ tests covering: language code generation, node types JSON, determinism,
//! grammar topologies, token/rule interaction, and edge cases.

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::StaticLanguageGenerator;

// ===================================================================
// Helpers
// ===================================================================

/// Builds grammar + real LR(1) parse table from a GrammarBuilder.
fn build_gt(builder: GrammarBuilder) -> (Grammar, ParseTable) {
    let mut g = builder.build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("compute ff");
    let t = build_lr1_automaton(&g, &ff).expect("build automaton");
    (g, t)
}

fn single_token_grammar() -> (Grammar, ParseTable) {
    build_gt(
        GrammarBuilder::new("single")
            .token("x", r"x")
            .rule("start", vec!["x"])
            .start("start"),
    )
}

fn two_alt_grammar() -> (Grammar, ParseTable) {
    build_gt(
        GrammarBuilder::new("two_alt")
            .token("a", r"a")
            .token("b", r"b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start"),
    )
}

fn chain_grammar() -> (Grammar, ParseTable) {
    build_gt(
        GrammarBuilder::new("chain")
            .token("x", r"x")
            .rule("c", vec!["x"])
            .rule("b", vec!["c"])
            .rule("a", vec!["b"])
            .start("a"),
    )
}

fn multi_token_grammar() -> (Grammar, ParseTable) {
    build_gt(
        GrammarBuilder::new("multi")
            .token("num", r"\d+")
            .token("id", r"[a-z]+")
            .token("plus", "+")
            .rule("start", vec!["num", "plus", "id"])
            .start("start"),
    )
}

fn seq_grammar() -> (Grammar, ParseTable) {
    build_gt(
        GrammarBuilder::new("seq")
            .token("a", r"a")
            .token("b", r"b")
            .token("c", r"c")
            .rule("start", vec!["a", "b", "c"])
            .start("start"),
    )
}

fn recursive_list_grammar() -> (Grammar, ParseTable) {
    build_gt(
        GrammarBuilder::new("list")
            .token("item", r"[a-z]+")
            .token("comma", ",")
            .rule("list", vec!["item"])
            .rule("list", vec!["list", "comma", "item"])
            .start("list"),
    )
}

fn nested_grammar() -> (Grammar, ParseTable) {
    build_gt(
        GrammarBuilder::new("nested")
            .token("lp", "(")
            .token("rp", ")")
            .token("x", r"x")
            .rule("atom", vec!["x"])
            .rule("atom", vec!["lp", "expr", "rp"])
            .rule("expr", vec!["atom"])
            .start("expr"),
    )
}

fn with_extras_grammar() -> (Grammar, ParseTable) {
    build_gt(
        GrammarBuilder::new("extras")
            .token("id", r"[a-z]+")
            .token("ws", r"[ \t]+")
            .extra("ws")
            .rule("start", vec!["id"])
            .start("start"),
    )
}

fn with_external_grammar() -> (Grammar, ParseTable) {
    // Externals are added to Grammar but do not participate in LR(1) table
    // building directly, so we construct a minimal parseable grammar.
    build_gt(
        GrammarBuilder::new("ext")
            .token("id", r"[a-z]+")
            .external("indent")
            .external("dedent")
            .rule("start", vec!["id"])
            .start("start"),
    )
}

fn wide_alt_grammar() -> (Grammar, ParseTable) {
    build_gt(
        GrammarBuilder::new("wide")
            .token("t1", r"1")
            .token("t2", r"2")
            .token("t3", r"3")
            .token("t4", r"4")
            .token("t5", r"5")
            .rule("start", vec!["t1"])
            .rule("start", vec!["t2"])
            .rule("start", vec!["t3"])
            .rule("start", vec!["t4"])
            .rule("start", vec!["t5"])
            .start("start"),
    )
}

fn diamond_grammar() -> (Grammar, ParseTable) {
    // Diamond: top -> left | right; left -> bottom; right -> bottom
    build_gt(
        GrammarBuilder::new("diamond")
            .token("x", r"x")
            .rule("bottom", vec!["x"])
            .rule("left_node", vec!["bottom"])
            .rule("right_node", vec!["bottom"])
            .rule("top", vec!["left_node"])
            .rule("top", vec!["right_node"])
            .start("top"),
    )
}

fn keyword_rich_grammar() -> (Grammar, ParseTable) {
    build_gt(
        GrammarBuilder::new("keywords")
            .token("if_kw", "if")
            .token("else_kw", "else")
            .token("while_kw", "while")
            .token("id", r"[a-z]+")
            .rule("start", vec!["if_kw", "id"])
            .start("start"),
    )
}

// ===================================================================
// 1. Language code is non-empty TokenStream (8 tests)
// ===================================================================

#[test]
fn lang_code_nonempty_single_token() {
    let (g, t) = single_token_grammar();
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty(), "language code must not be empty");
}

#[test]
fn lang_code_nonempty_two_alt() {
    let (g, t) = two_alt_grammar();
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn lang_code_nonempty_chain() {
    let (g, t) = chain_grammar();
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn lang_code_nonempty_multi_token() {
    let (g, t) = multi_token_grammar();
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn lang_code_nonempty_seq() {
    let (g, t) = seq_grammar();
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn lang_code_nonempty_recursive_list() {
    let (g, t) = recursive_list_grammar();
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn lang_code_nonempty_nested() {
    let (g, t) = nested_grammar();
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn lang_code_nonempty_with_extras() {
    let (g, t) = with_extras_grammar();
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

// ===================================================================
// 2. Language code contains expected patterns (8 tests)
// ===================================================================

#[test]
fn lang_code_contains_tree_sitter_fn() {
    let (g, t) = single_token_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(
        code.contains("tree_sitter"),
        "should reference tree_sitter function"
    );
}

#[test]
fn lang_code_contains_language_name() {
    let (g, t) = single_token_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("single"), "should embed grammar name");
}

#[test]
fn lang_code_two_alt_contains_name() {
    let (g, t) = two_alt_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("two_alt"));
}

#[test]
fn lang_code_contains_fn_keyword() {
    let (g, t) = single_token_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("fn"), "should contain function definitions");
}

#[test]
fn lang_code_chain_references_name() {
    let (g, t) = chain_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("chain"));
}

#[test]
fn lang_code_multi_token_contains_name() {
    let (g, t) = multi_token_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("multi"));
}

#[test]
fn lang_code_nested_contains_name() {
    let (g, t) = nested_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("nested"));
}

#[test]
fn lang_code_extras_contains_name() {
    let (g, t) = with_extras_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("extras"));
}

// ===================================================================
// 3. Node types is valid JSON (8 tests)
// ===================================================================

fn assert_valid_json_array(json: &str) {
    let v: serde_json::Value = serde_json::from_str(json).expect("node types must be valid JSON");
    assert!(v.is_array(), "node types must be a JSON array");
}

#[test]
fn node_types_valid_json_single_token() {
    let (g, t) = single_token_grammar();
    assert_valid_json_array(&StaticLanguageGenerator::new(g, t).generate_node_types());
}

#[test]
fn node_types_valid_json_two_alt() {
    let (g, t) = two_alt_grammar();
    assert_valid_json_array(&StaticLanguageGenerator::new(g, t).generate_node_types());
}

#[test]
fn node_types_valid_json_chain() {
    let (g, t) = chain_grammar();
    assert_valid_json_array(&StaticLanguageGenerator::new(g, t).generate_node_types());
}

#[test]
fn node_types_valid_json_multi_token() {
    let (g, t) = multi_token_grammar();
    assert_valid_json_array(&StaticLanguageGenerator::new(g, t).generate_node_types());
}

#[test]
fn node_types_valid_json_recursive_list() {
    let (g, t) = recursive_list_grammar();
    assert_valid_json_array(&StaticLanguageGenerator::new(g, t).generate_node_types());
}

#[test]
fn node_types_valid_json_nested() {
    let (g, t) = nested_grammar();
    assert_valid_json_array(&StaticLanguageGenerator::new(g, t).generate_node_types());
}

#[test]
fn node_types_valid_json_with_extras() {
    let (g, t) = with_extras_grammar();
    assert_valid_json_array(&StaticLanguageGenerator::new(g, t).generate_node_types());
}

#[test]
fn node_types_valid_json_with_externals() {
    let (g, t) = with_external_grammar();
    assert_valid_json_array(&StaticLanguageGenerator::new(g, t).generate_node_types());
}

// ===================================================================
// 4. Node types contains expected type names (5 tests)
// ===================================================================

#[test]
fn node_types_contains_rule_entries() {
    let (g, t) = single_token_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let arr = v.as_array().unwrap();
    // At minimum the grammar has a named rule → should produce entries
    assert!(!arr.is_empty(), "should contain at least one node type");
}

#[test]
fn node_types_entries_have_type_field() {
    let (g, t) = two_alt_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(
            entry.get("type").is_some(),
            "each entry must have a 'type' field"
        );
    }
}

#[test]
fn node_types_entries_have_named_field() {
    let (g, t) = chain_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(
            entry.get("named").is_some(),
            "each entry must have a 'named' field"
        );
    }
}

#[test]
fn node_types_external_tokens_appear() {
    let (g, t) = with_external_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let types: Vec<String> = v
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e.get("type").and_then(|t| t.as_str()).map(String::from))
        .collect();
    assert!(types.contains(&"indent".to_string()), "indent expected");
    assert!(types.contains(&"dedent".to_string()), "dedent expected");
}

#[test]
fn node_types_all_named_true_for_rules() {
    let (g, t) = multi_token_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    for entry in v.as_array().unwrap() {
        let named = entry.get("named").and_then(|n| n.as_bool());
        assert_eq!(named, Some(true), "all generated types should be named");
    }
}

// ===================================================================
// 5. Determinism (5 tests)
// ===================================================================

#[test]
fn determinism_language_code_single() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = single_token_grammar();
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2, "language code must be deterministic");
}

#[test]
fn determinism_language_code_two_alt() {
    let (g1, t1) = two_alt_grammar();
    let (g2, t2) = two_alt_grammar();
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
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = single_token_grammar();
    let n1 = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let n2 = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_eq!(n1, n2, "node types must be deterministic");
}

#[test]
fn determinism_node_types_chain() {
    let (g1, t1) = chain_grammar();
    let (g2, t2) = chain_grammar();
    let n1 = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let n2 = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_eq!(n1, n2);
}

#[test]
fn determinism_node_types_recursive_list() {
    let (g1, t1) = recursive_list_grammar();
    let (g2, t2) = recursive_list_grammar();
    let n1 = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let n2 = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_eq!(n1, n2);
}

// ===================================================================
// 6. Various grammar topologies (8 tests)
// ===================================================================

#[test]
fn topology_wide_alt_produces_code() {
    let (g, t) = wide_alt_grammar();
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn topology_wide_alt_node_types_valid() {
    let (g, t) = wide_alt_grammar();
    assert_valid_json_array(&StaticLanguageGenerator::new(g, t).generate_node_types());
}

#[test]
fn topology_diamond_produces_code() {
    let (g, t) = diamond_grammar();
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn topology_diamond_node_types_valid() {
    let (g, t) = diamond_grammar();
    assert_valid_json_array(&StaticLanguageGenerator::new(g, t).generate_node_types());
}

#[test]
fn topology_deep_chain_three_levels() {
    // 3-level chain: top -> mid -> low -> tok
    let (g, t) = build_gt(
        GrammarBuilder::new("deep3")
            .token("tok", r"t")
            .rule("low", vec!["tok"])
            .rule("mid", vec!["low"])
            .rule("top", vec!["mid"])
            .start("top"),
    );
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn topology_deep_chain_node_types() {
    let (g, t) = build_gt(
        GrammarBuilder::new("deep3n")
            .token("tok", r"t")
            .rule("low", vec!["tok"])
            .rule("mid", vec!["low"])
            .rule("top", vec!["mid"])
            .start("top"),
    );
    assert_valid_json_array(&StaticLanguageGenerator::new(g, t).generate_node_types());
}

#[test]
fn topology_left_recursive() {
    let (g, t) = build_gt(
        GrammarBuilder::new("lrec")
            .token("x", r"x")
            .token("plus", "+")
            .rule("expr", vec!["x"])
            .rule("expr", vec!["expr", "plus", "x"])
            .start("expr"),
    );
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn topology_right_recursive() {
    let (g, t) = build_gt(
        GrammarBuilder::new("rrec")
            .token("x", r"x")
            .token("cons", ":")
            .rule("list_r", vec!["x"])
            .rule("list_r", vec!["x", "cons", "list_r"])
            .start("list_r"),
    );
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

// ===================================================================
// 7. Token/rule interaction (5 tests)
// ===================================================================

#[test]
fn token_rule_single_token_single_rule() {
    let (g, t) = single_token_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(!v.as_array().unwrap().is_empty());
}

#[test]
fn token_rule_many_tokens_one_rule() {
    let (g, t) = seq_grammar();
    let gen_val = StaticLanguageGenerator::new(g, t);
    let code = gen_val.generate_language_code();
    let node = gen_val.generate_node_types();
    assert!(!code.is_empty());
    assert_valid_json_array(&node);
}

#[test]
fn token_rule_one_token_many_rules() {
    // Single terminal referenced by multiple rules
    let (g, t) = build_gt(
        GrammarBuilder::new("onetok")
            .token("x", r"x")
            .rule("a_rule", vec!["x"])
            .rule("b_rule", vec!["x"])
            .rule("top_rule", vec!["a_rule"])
            .rule("top_rule", vec!["b_rule"])
            .start("top_rule"),
    );
    let gen_val = StaticLanguageGenerator::new(g, t);
    assert!(!gen_val.generate_language_code().is_empty());
    assert_valid_json_array(&gen_val.generate_node_types());
}

#[test]
fn token_rule_keyword_tokens() {
    let (g, t) = keyword_rich_grammar();
    let gen_val = StaticLanguageGenerator::new(g, t);
    let code = gen_val.generate_language_code().to_string();
    assert!(code.contains("keywords"), "grammar name should appear");
}

#[test]
fn token_rule_extras_do_not_break_output() {
    let (g, t) = with_extras_grammar();
    let gen_val = StaticLanguageGenerator::new(g, t);
    let code = gen_val.generate_language_code().to_string();
    let json = gen_val.generate_node_types();
    assert!(!code.is_empty());
    assert_valid_json_array(&json);
}

// ===================================================================
// 8. Edge cases (8 tests)
// ===================================================================

#[test]
fn edge_case_default_parse_table() {
    // Use ParseTable::default() instead of real LR(1) table
    let grammar = GrammarBuilder::new("defpt")
        .token("x", r"x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let gen_val = StaticLanguageGenerator::new(grammar, ParseTable::default());
    // Should not panic; code may be minimal but non-empty
    let code = gen_val.generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn edge_case_default_table_node_types() {
    let grammar = GrammarBuilder::new("defnt")
        .token("x", r"x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let json = StaticLanguageGenerator::new(grammar, ParseTable::default()).generate_node_types();
    assert_valid_json_array(&json);
}

#[test]
fn edge_case_empty_grammar_default_table() {
    let grammar = Grammar::new("empty".to_string());
    let gen_val = StaticLanguageGenerator::new(grammar, ParseTable::default());
    let code = gen_val.generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn edge_case_empty_grammar_node_types() {
    let grammar = Grammar::new("empty_nt".to_string());
    let json = StaticLanguageGenerator::new(grammar, ParseTable::default()).generate_node_types();
    assert_valid_json_array(&json);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(
        v.as_array().unwrap().is_empty(),
        "empty grammar → empty array"
    );
}

#[test]
fn edge_case_set_start_can_be_empty() {
    let (g, t) = single_token_grammar();
    let mut gen_val = StaticLanguageGenerator::new(g, t);
    gen_val.set_start_can_be_empty(true);
    assert!(gen_val.start_can_be_empty);
    // Should still generate valid output
    assert!(!gen_val.generate_language_code().is_empty());
    assert_valid_json_array(&gen_val.generate_node_types());
}

#[test]
fn edge_case_compressed_tables_none_by_default() {
    let (g, t) = single_token_grammar();
    let gen_val = StaticLanguageGenerator::new(g, t);
    assert!(gen_val.compressed_tables.is_none());
}

#[test]
fn edge_case_grammar_name_preserved() {
    let (g, t) = single_token_grammar();
    let gen_val = StaticLanguageGenerator::new(g, t);
    assert_eq!(gen_val.grammar.name, "single");
}

#[test]
fn edge_case_long_grammar_name() {
    let long_name = "a_very_long_grammar_name_for_testing_purposes";
    let (g, t) = build_gt(
        GrammarBuilder::new(long_name)
            .token("x", r"x")
            .rule("start", vec!["x"])
            .start("start"),
    );
    let gen_val = StaticLanguageGenerator::new(g, t);
    assert_eq!(gen_val.grammar.name, long_name);
    let code = gen_val.generate_language_code().to_string();
    assert!(code.contains(long_name));
}
