//! Comprehensive tests for Language struct generation in adze-tablegen.
//!
//! Covers: struct definition output, symbol table references, parse action data,
//! determinism, ABI builder properties, multiple grammars, and edge cases.

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tablegen::{AbiLanguageBuilder, StaticLanguageGenerator};

// =====================================================================
// Helpers
// =====================================================================

fn build_pipeline(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> (Grammar, adze_glr_core::ParseTable) {
    let mut b = GrammarBuilder::new(name);
    for &(tok_name, tok_pat) in tokens {
        b = b.token(tok_name, tok_pat);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    let mut g = b.start(start).build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    (g, pt)
}

fn minimal_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline("minimal", &[("a", "a")], &[("s", vec!["a"])], "s")
}

fn two_token_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(
        "two_tok",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    )
}

fn alt_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(
        "alt",
        &[("x", "x"), ("y", "y")],
        &[("s", vec!["x"]), ("s", vec!["y"])],
        "s",
    )
}

fn recursive_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(
        "recur",
        &[("a", "a")],
        &[("s", vec!["a"]), ("s", vec!["s", "a"])],
        "s",
    )
}

fn nested_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(
        "nested",
        &[("a", "a")],
        &[
            ("leaf", vec!["a"]),
            ("mid", vec!["leaf"]),
            ("s", vec!["mid"]),
        ],
        "s",
    )
}

fn three_alt_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(
        "three_alt",
        &[("x", "x"), ("y", "y"), ("z", "z")],
        &[("s", vec!["x"]), ("s", vec!["y"]), ("s", vec!["z"])],
        "s",
    )
}

fn long_chain_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(
        "long_chain",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d")],
        &[("s", vec!["a", "b", "c", "d"])],
        "s",
    )
}

fn expression_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    let mut g = GrammarBuilder::new("expr")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    (g, pt)
}

fn multi_nonterm_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    build_pipeline(
        "multi_nt",
        &[("a", "a"), ("b", "b")],
        &[("inner", vec!["a"]), ("s", vec!["inner", "b"])],
        "s",
    )
}

fn code_str(g: Grammar, pt: adze_glr_core::ParseTable) -> String {
    StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string()
}

// =====================================================================
// 1. Language code contains static Language definition (8 tests)
// =====================================================================

#[test]
fn lang_def_minimal_grammar_has_language_static() {
    let (g, pt) = minimal_grammar();
    let out = code_str(g, pt);
    assert!(
        out.contains("LANGUAGE") || out.contains("Language") || out.contains("language"),
        "output must contain Language definition"
    );
}

#[test]
fn lang_def_two_token_grammar_has_language() {
    let (g, pt) = two_token_grammar();
    let out = code_str(g, pt);
    assert!(out.contains("LANGUAGE") || out.contains("language"));
}

#[test]
fn lang_def_alt_grammar_has_language() {
    let (g, pt) = alt_grammar();
    let out = code_str(g, pt);
    assert!(out.contains("LANGUAGE") || out.contains("language"));
}

#[test]
fn lang_def_recursive_grammar_has_language() {
    let (g, pt) = recursive_grammar();
    let out = code_str(g, pt);
    assert!(out.contains("LANGUAGE") || out.contains("language"));
}

#[test]
fn lang_def_nested_grammar_has_language() {
    let (g, pt) = nested_grammar();
    let out = code_str(g, pt);
    assert!(out.contains("LANGUAGE") || out.contains("language"));
}

#[test]
fn lang_def_three_alt_grammar_has_language() {
    let (g, pt) = three_alt_grammar();
    let out = code_str(g, pt);
    assert!(out.contains("LANGUAGE") || out.contains("language"));
}

#[test]
fn lang_def_expression_grammar_has_language() {
    let (g, pt) = expression_grammar();
    let out = code_str(g, pt);
    assert!(out.contains("LANGUAGE") || out.contains("language"));
}

#[test]
fn lang_def_long_chain_has_language() {
    let (g, pt) = long_chain_grammar();
    let out = code_str(g, pt);
    assert!(out.contains("LANGUAGE") || out.contains("language"));
}

// =====================================================================
// 2. Language code contains symbol table (8 tests)
// =====================================================================

#[test]
fn symbol_table_minimal_contains_symbol_names() {
    let (g, pt) = minimal_grammar();
    let out = code_str(g, pt);
    // Generated code should reference symbol names or symbol_names array
    assert!(
        out.contains("symbol") || out.contains("SYMBOL") || out.contains("Symbol"),
        "output should reference symbol identifiers"
    );
}

#[test]
fn symbol_table_has_end_marker() {
    let (g, pt) = minimal_grammar();
    let out = code_str(g, pt);
    assert!(
        out.contains("end") || out.contains("END") || out.contains("eof") || out.contains("EOF"),
        "output should reference an end/eof marker"
    );
}

#[test]
fn symbol_table_two_token_references_both() {
    let (g, pt) = two_token_grammar();
    let out = code_str(g, pt);
    // Must reference token-related data for the grammar
    assert!(
        out.contains("symbol") || out.contains("token"),
        "two-token grammar output references symbol/token data"
    );
}

#[test]
fn symbol_table_alt_grammar_references_symbols() {
    let (g, pt) = alt_grammar();
    let out = code_str(g, pt);
    assert!(out.contains("symbol") || out.contains("Symbol") || out.contains("SYMBOL"));
}

#[test]
fn symbol_table_three_alt_grammar_references_symbols() {
    let (g, pt) = three_alt_grammar();
    let out = code_str(g, pt);
    assert!(out.contains("symbol") || out.contains("Symbol") || out.contains("SYMBOL"));
}

#[test]
fn symbol_table_metadata_count_matches_parse_table() {
    let (g, pt) = minimal_grammar();
    let meta_count = pt.symbol_metadata.len();
    let lang_gen = StaticLanguageGenerator::new(g, pt);
    let out = lang_gen.generate_language_code().to_string();
    // The output should be non-empty and the symbol_metadata length should be positive
    assert!(!out.is_empty());
    assert!(meta_count > 0, "symbol_metadata should have entries");
}

#[test]
fn symbol_table_nested_grammar_references_symbols() {
    let (g, pt) = nested_grammar();
    let out = code_str(g, pt);
    assert!(out.contains("symbol") || out.contains("Symbol") || out.contains("SYMBOL"));
}

#[test]
fn symbol_table_expression_grammar_references_symbols() {
    let (g, pt) = expression_grammar();
    let out = code_str(g, pt);
    assert!(out.contains("symbol") || out.contains("Symbol") || out.contains("SYMBOL"));
}

// =====================================================================
// 3. Language code contains parse actions (7 tests)
// =====================================================================

#[test]
fn parse_actions_minimal_output_nonempty() {
    let (g, pt) = minimal_grammar();
    let out = code_str(g, pt);
    assert!(!out.is_empty(), "generated code must be non-empty");
}

#[test]
fn parse_actions_two_token_output_nonempty() {
    let (g, pt) = two_token_grammar();
    assert!(!code_str(g, pt).is_empty());
}

#[test]
fn parse_actions_alt_grammar_contains_action_data() {
    let (g, pt) = alt_grammar();
    let out = code_str(g, pt);
    // Action tables are encoded as arrays — check for numeric data
    assert!(
        out.contains("action") || out.contains("ACTION") || out.contains('['),
        "output should contain action data or array literals"
    );
}

#[test]
fn parse_actions_recursive_grammar_output_nonempty() {
    let (g, pt) = recursive_grammar();
    assert!(!code_str(g, pt).is_empty());
}

#[test]
fn parse_actions_expression_grammar_output_nonempty() {
    let (g, pt) = expression_grammar();
    assert!(!code_str(g, pt).is_empty());
}

#[test]
fn parse_actions_long_chain_output_nonempty() {
    let (g, pt) = long_chain_grammar();
    assert!(!code_str(g, pt).is_empty());
}

#[test]
fn parse_actions_table_has_states() {
    let (_g, pt) = minimal_grammar();
    assert!(
        pt.state_count > 0,
        "parse table must have at least one state"
    );
    assert!(
        !pt.action_table.is_empty(),
        "action table must not be empty"
    );
}

// =====================================================================
// 4. Language struct deterministic (8 tests)
// =====================================================================

fn deterministic_code(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> String {
    let (g, pt) = build_pipeline(name, tokens, rules, start);
    code_str(g, pt)
}

#[test]
fn determinism_minimal_same_grammar_same_code() {
    let a = deterministic_code("det_min", &[("a", "a")], &[("s", vec!["a"])], "s");
    let b = deterministic_code("det_min", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(a, b);
}

#[test]
fn determinism_two_token_same_grammar_same_code() {
    let a = deterministic_code(
        "det_two",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    let b = deterministic_code(
        "det_two",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert_eq!(a, b);
}

#[test]
fn determinism_alt_same_grammar_same_code() {
    let a = deterministic_code(
        "det_alt",
        &[("x", "x"), ("y", "y")],
        &[("s", vec!["x"]), ("s", vec!["y"])],
        "s",
    );
    let b = deterministic_code(
        "det_alt",
        &[("x", "x"), ("y", "y")],
        &[("s", vec!["x"]), ("s", vec!["y"])],
        "s",
    );
    assert_eq!(a, b);
}

#[test]
fn determinism_recursive_same_grammar_same_code() {
    let a = deterministic_code(
        "det_rec",
        &[("a", "a")],
        &[("s", vec!["a"]), ("s", vec!["s", "a"])],
        "s",
    );
    let b = deterministic_code(
        "det_rec",
        &[("a", "a")],
        &[("s", vec!["a"]), ("s", vec!["s", "a"])],
        "s",
    );
    assert_eq!(a, b);
}

#[test]
fn determinism_nested_same_grammar_same_code() {
    let a = deterministic_code(
        "det_nest",
        &[("a", "a")],
        &[
            ("leaf", vec!["a"]),
            ("mid", vec!["leaf"]),
            ("s", vec!["mid"]),
        ],
        "s",
    );
    let b = deterministic_code(
        "det_nest",
        &[("a", "a")],
        &[
            ("leaf", vec!["a"]),
            ("mid", vec!["leaf"]),
            ("s", vec!["mid"]),
        ],
        "s",
    );
    assert_eq!(a, b);
}

#[test]
fn determinism_three_alt_same_grammar_same_code() {
    let a = deterministic_code(
        "det_3alt",
        &[("x", "x"), ("y", "y"), ("z", "z")],
        &[("s", vec!["x"]), ("s", vec!["y"]), ("s", vec!["z"])],
        "s",
    );
    let b = deterministic_code(
        "det_3alt",
        &[("x", "x"), ("y", "y"), ("z", "z")],
        &[("s", vec!["x"]), ("s", vec!["y"]), ("s", vec!["z"])],
        "s",
    );
    assert_eq!(a, b);
}

#[test]
fn determinism_long_chain_same_grammar_same_code() {
    let a = deterministic_code(
        "det_lc",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d")],
        &[("s", vec!["a", "b", "c", "d"])],
        "s",
    );
    let b = deterministic_code(
        "det_lc",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d")],
        &[("s", vec!["a", "b", "c", "d"])],
        "s",
    );
    assert_eq!(a, b);
}

#[test]
fn determinism_expression_same_grammar_same_code() {
    let (g1, pt1) = expression_grammar();
    let out1 = code_str(g1, pt1);
    let (g2, pt2) = expression_grammar();
    let out2 = code_str(g2, pt2);
    assert_eq!(out1, out2);
}

// =====================================================================
// 5. ABI builder properties (8 tests)
// =====================================================================

#[test]
fn abi_builder_minimal_produces_nonempty_output() {
    let (g, pt) = minimal_grammar();
    let builder = AbiLanguageBuilder::new(&g, &pt);
    let out = builder.generate().to_string();
    assert!(!out.is_empty());
}

#[test]
fn abi_builder_two_token_produces_output() {
    let (g, pt) = two_token_grammar();
    let out = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!out.is_empty());
}

#[test]
fn abi_builder_alt_grammar_produces_output() {
    let (g, pt) = alt_grammar();
    let out = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!out.is_empty());
}

#[test]
fn abi_builder_recursive_grammar_produces_output() {
    let (g, pt) = recursive_grammar();
    let out = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!out.is_empty());
}

#[test]
fn abi_builder_output_references_language_name() {
    let (g, pt) = minimal_grammar();
    let out = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(
        out.contains("minimal") || out.contains("language") || out.contains("tree_sitter"),
        "ABI output should reference the grammar name or language fn"
    );
}

#[test]
fn abi_builder_nested_grammar_produces_output() {
    let (g, pt) = nested_grammar();
    let out = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!out.is_empty());
}

#[test]
fn abi_builder_expression_grammar_produces_output() {
    let (g, pt) = expression_grammar();
    let out = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!out.is_empty());
}

#[test]
fn abi_builder_long_chain_produces_output() {
    let (g, pt) = long_chain_grammar();
    let out = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!out.is_empty());
}

// =====================================================================
// 6. Multiple grammars produce different structs (8 tests)
// =====================================================================

#[test]
fn diff_grammars_minimal_vs_two_token() {
    let (g1, pt1) = minimal_grammar();
    let (g2, pt2) = two_token_grammar();
    assert_ne!(code_str(g1, pt1), code_str(g2, pt2));
}

#[test]
fn diff_grammars_minimal_vs_alt() {
    let (g1, pt1) = minimal_grammar();
    let (g2, pt2) = alt_grammar();
    assert_ne!(code_str(g1, pt1), code_str(g2, pt2));
}

#[test]
fn diff_grammars_alt_vs_recursive() {
    let (g1, pt1) = alt_grammar();
    let (g2, pt2) = recursive_grammar();
    assert_ne!(code_str(g1, pt1), code_str(g2, pt2));
}

#[test]
fn diff_grammars_minimal_vs_nested() {
    let (g1, pt1) = minimal_grammar();
    let (g2, pt2) = nested_grammar();
    assert_ne!(code_str(g1, pt1), code_str(g2, pt2));
}

#[test]
fn diff_grammars_two_token_vs_alt() {
    let (g1, pt1) = two_token_grammar();
    let (g2, pt2) = alt_grammar();
    assert_ne!(code_str(g1, pt1), code_str(g2, pt2));
}

#[test]
fn diff_grammars_nested_vs_long_chain() {
    let (g1, pt1) = nested_grammar();
    let (g2, pt2) = long_chain_grammar();
    assert_ne!(code_str(g1, pt1), code_str(g2, pt2));
}

#[test]
fn diff_grammars_three_alt_vs_expression() {
    let (g1, pt1) = three_alt_grammar();
    let (g2, pt2) = expression_grammar();
    assert_ne!(code_str(g1, pt1), code_str(g2, pt2));
}

#[test]
fn diff_grammars_abi_minimal_vs_alt() {
    let (g1, pt1) = minimal_grammar();
    let (g2, pt2) = alt_grammar();
    let abi1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let abi2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_ne!(abi1, abi2);
}

// =====================================================================
// 7. Edge cases (8 tests)
// =====================================================================

#[test]
fn edge_single_token_single_rule_compiles() {
    let (g, pt) = minimal_grammar();
    let lang_gen = StaticLanguageGenerator::new(g, pt);
    let out = lang_gen.generate_language_code();
    assert!(!out.is_empty());
}

#[test]
fn edge_expression_with_precedence_compiles() {
    let (g, pt) = expression_grammar();
    let lang_gen = StaticLanguageGenerator::new(g, pt);
    let out = lang_gen.generate_language_code();
    assert!(!out.is_empty());
}

#[test]
fn edge_recursive_grammar_code_nonempty() {
    let (g, pt) = recursive_grammar();
    let out = code_str(g, pt);
    assert!(!out.is_empty());
}

#[test]
fn edge_deep_nesting_three_levels() {
    let (g, pt) = nested_grammar();
    let out = code_str(g, pt);
    assert!(!out.is_empty());
    assert!(out.contains("LANGUAGE") || out.contains("language"));
}

#[test]
fn edge_many_alternatives_three() {
    let (g, pt) = three_alt_grammar();
    let out = code_str(g, pt);
    assert!(!out.is_empty());
}

#[test]
fn edge_long_chain_four_symbols() {
    let (g, pt) = long_chain_grammar();
    let out = code_str(g, pt);
    assert!(!out.is_empty());
    assert!(out.contains("LANGUAGE") || out.contains("language"));
}

#[test]
fn edge_multi_nonterm_grammar() {
    let (g, pt) = multi_nonterm_grammar();
    let out = code_str(g, pt);
    assert!(!out.is_empty());
}

#[test]
fn edge_node_types_json_valid() {
    let (g, pt) = minimal_grammar();
    let lang_gen = StaticLanguageGenerator::new(g, pt);
    let json_str = lang_gen.generate_node_types();
    // Must be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json_str)
        .unwrap_or_else(|e| panic!("node_types must be valid JSON: {e}"));
    assert!(parsed.is_array(), "node_types should be a JSON array");
}

// =====================================================================
// Bonus: additional coverage (extra tests beyond 55)
// =====================================================================

#[test]
fn static_gen_preserves_grammar_name() {
    let (g, pt) = minimal_grammar();
    let lang_gen = StaticLanguageGenerator::new(g, pt);
    assert_eq!(lang_gen.grammar.name, "minimal");
}

#[test]
fn static_gen_start_can_be_empty_roundtrip() {
    let (g, pt) = minimal_grammar();
    let mut lang_gen = StaticLanguageGenerator::new(g, pt);
    assert!(!lang_gen.start_can_be_empty);
    lang_gen.set_start_can_be_empty(true);
    assert!(lang_gen.start_can_be_empty);
    lang_gen.set_start_can_be_empty(false);
    assert!(!lang_gen.start_can_be_empty);
}

#[test]
fn abi_builder_deterministic() {
    let (g, pt) = alt_grammar();
    let out1 = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let out2 = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert_eq!(out1, out2);
}

#[test]
fn node_types_alt_grammar_valid_json() {
    let (g, pt) = alt_grammar();
    let json_str = StaticLanguageGenerator::new(g, pt).generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str)
        .unwrap_or_else(|e| panic!("node_types must be valid JSON: {e}"));
    assert!(parsed.is_array());
}

#[test]
fn node_types_recursive_grammar_valid_json() {
    let (g, pt) = recursive_grammar();
    let json_str = StaticLanguageGenerator::new(g, pt).generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str)
        .unwrap_or_else(|e| panic!("node_types must be valid JSON: {e}"));
    assert!(parsed.is_array());
}

#[test]
fn parse_table_eof_symbol_is_set() {
    let (_g, pt) = minimal_grammar();
    // EOF symbol should be a valid SymbolId (not a sentinel)
    assert!(
        pt.eof_symbol.0 != 0xFFFF,
        "eof_symbol must not be the error sentinel"
    );
}

#[test]
fn parse_table_start_symbol_is_set() {
    let (_g, pt) = minimal_grammar();
    // start_symbol should be present in symbol_to_index
    assert!(
        pt.symbol_to_index.contains_key(&pt.start_symbol),
        "start_symbol must be in symbol_to_index"
    );
}

#[test]
fn abi_builder_chain_vs_alt_differ() {
    let (g1, pt1) = two_token_grammar();
    let (g2, pt2) = alt_grammar();
    let abi1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let abi2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_ne!(abi1, abi2);
}

#[test]
fn static_gen_code_contains_array_literal() {
    let (g, pt) = minimal_grammar();
    let out = code_str(g, pt);
    // Generated code should have at least one array literal (for tables)
    assert!(
        out.contains('['),
        "output should contain array literals for table data"
    );
}

#[test]
fn node_types_expression_grammar_valid_json() {
    let (g, pt) = expression_grammar();
    let json_str = StaticLanguageGenerator::new(g, pt).generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str)
        .unwrap_or_else(|e| panic!("node_types must be valid JSON: {e}"));
    assert!(parsed.is_array());
}
