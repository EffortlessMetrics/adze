//! Comprehensive integration tests for StaticLanguageGenerator, AbiLanguageBuilder,
//! and NodeTypesGenerator across various grammar shapes.

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tablegen::{
    AbiLanguageBuilder, NodeTypesGenerator, StaticLanguageGenerator, TableCompressor,
    collect_token_indices,
};
use serde_json::Value;

// =====================================================================
// Helpers: build grammar + parse table via the full pipeline
// =====================================================================

/// Minimal: single token, single rule.
fn simple_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    let mut g = GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    (g, pt)
}

/// Two tokens, two alternative productions.
fn two_alt_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    let mut g = GrammarBuilder::new("two_alt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    (g, pt)
}

/// Chain: s -> a b.
fn chain_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    let mut g = GrammarBuilder::new("chain")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    (g, pt)
}

/// Recursive: s -> a | s a (left recursion).
fn recursive_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    let mut g = GrammarBuilder::new("recursive")
        .token("a", "a")
        .rule("s", vec!["a"])
        .rule("s", vec!["s", "a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    (g, pt)
}

/// Three tokens, three alternatives.
fn three_alt_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    let mut g = GrammarBuilder::new("three_alt")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .rule("s", vec!["z"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    (g, pt)
}

/// Long chain: s -> a b c d.
fn long_chain_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    let mut g = GrammarBuilder::new("long_chain")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("s", vec!["a", "b", "c", "d"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    (g, pt)
}

/// Multi-rule: two non-terminals.
fn multi_rule_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    let mut g = GrammarBuilder::new("multi_rule")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["a"])
        .rule("s", vec!["inner", "b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    (g, pt)
}

/// Deep nesting: s -> mid, mid -> leaf, leaf -> a.
fn deep_nesting_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    let mut g = GrammarBuilder::new("deep_nesting")
        .token("a", "a")
        .rule("leaf", vec!["a"])
        .rule("mid", vec!["leaf"])
        .rule("s", vec!["mid"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    (g, pt)
}

/// Grammar with precedence: expr -> expr + expr | num, left-assoc.
fn precedence_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    let mut g = GrammarBuilder::new("precedence")
        .token("num", r"\d+")
        .token("plus", "+")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    (g, pt)
}

/// Mixed alternative + chain: s -> a b | c.
fn mixed_grammar() -> (Grammar, adze_glr_core::ParseTable) {
    let mut g = GrammarBuilder::new("mixed")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    (g, pt)
}

// =====================================================================
// StaticLanguageGenerator — construction
// =====================================================================

#[test]
fn static_gen_new_simple() {
    let (g, pt) = simple_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.grammar.name.is_empty());
}

#[test]
fn static_gen_new_two_alt() {
    let (g, pt) = two_alt_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert_eq!(slg.grammar.name, "two_alt");
}

#[test]
fn static_gen_new_chain() {
    let (g, pt) = chain_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert_eq!(slg.grammar.name, "chain");
}

#[test]
fn static_gen_new_recursive() {
    let (g, pt) = recursive_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert_eq!(slg.grammar.name, "recursive");
}

#[test]
fn static_gen_compressed_tables_default_none() {
    let (g, pt) = simple_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(slg.compressed_tables.is_none());
}

#[test]
fn static_gen_start_can_be_empty_default_false() {
    let (g, pt) = simple_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.start_can_be_empty);
}

#[test]
fn static_gen_set_start_can_be_empty() {
    let (g, pt) = simple_grammar();
    let mut slg = StaticLanguageGenerator::new(g, pt);
    slg.set_start_can_be_empty(true);
    assert!(slg.start_can_be_empty);
}

// =====================================================================
// StaticLanguageGenerator — generate_language_code
// =====================================================================

#[test]
fn static_gen_code_nonempty_simple() {
    let (g, pt) = simple_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    let code = slg.generate_language_code();
    assert!(!code.is_empty(), "TokenStream must not be empty");
}

#[test]
fn static_gen_code_nonempty_two_alt() {
    let (g, pt) = two_alt_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    let code = slg.generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn static_gen_code_nonempty_chain() {
    let (g, pt) = chain_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn static_gen_code_nonempty_recursive() {
    let (g, pt) = recursive_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn static_gen_code_nonempty_three_alt() {
    let (g, pt) = three_alt_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn static_gen_code_nonempty_long_chain() {
    let (g, pt) = long_chain_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn static_gen_code_nonempty_multi_rule() {
    let (g, pt) = multi_rule_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn static_gen_code_nonempty_deep_nesting() {
    let (g, pt) = deep_nesting_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn static_gen_code_nonempty_precedence() {
    let (g, pt) = precedence_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn static_gen_code_nonempty_mixed() {
    let (g, pt) = mixed_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

// =====================================================================
// StaticLanguageGenerator — generated code contains expected identifiers
// =====================================================================

#[test]
fn static_gen_code_contains_language_keyword() {
    let (g, pt) = simple_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    let code_str = slg.generate_language_code().to_string();
    // The generated code should reference some language-related identifier
    assert!(
        code_str.contains("LANGUAGE")
            || code_str.contains("language")
            || code_str.contains("Language"),
        "generated code should contain a language identifier"
    );
}

#[test]
fn static_gen_code_contains_state_count() {
    let (g, pt) = simple_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    let code_str = slg.generate_language_code().to_string();
    // Must contain at least one numeric literal for state/symbol counts
    assert!(
        code_str.chars().any(|c| c.is_ascii_digit()),
        "generated code should contain numeric literals"
    );
}

// =====================================================================
// StaticLanguageGenerator — generate_node_types
// =====================================================================

#[test]
fn static_gen_node_types_nonempty() {
    let (g, pt) = simple_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    let nt = slg.generate_node_types();
    assert!(!nt.is_empty());
}

#[test]
fn static_gen_node_types_is_valid_json() {
    let (g, pt) = multi_rule_grammar();
    let slg = StaticLanguageGenerator::new(g, pt);
    let nt = slg.generate_node_types();
    let parsed: Value = serde_json::from_str(&nt).expect("must be valid JSON");
    assert!(parsed.is_array());
}

// =====================================================================
// StaticLanguageGenerator — determinism
// =====================================================================

#[test]
fn static_gen_deterministic_simple() {
    let (g1, pt1) = simple_grammar();
    let (g2, pt2) = simple_grammar();
    let code1 = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let code2 = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert_eq!(code1, code2);
}

#[test]
fn static_gen_deterministic_recursive() {
    let (g1, pt1) = recursive_grammar();
    let (g2, pt2) = recursive_grammar();
    let code1 = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let code2 = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert_eq!(code1, code2);
}

// =====================================================================
// AbiLanguageBuilder — construction and generate
// =====================================================================

#[test]
fn abi_builder_new_simple() {
    let (g, pt) = simple_grammar();
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

#[test]
fn abi_builder_generate_nonempty_simple() {
    let (g, pt) = simple_grammar();
    let builder = AbiLanguageBuilder::new(&g, &pt);
    let code = builder.generate();
    assert!(!code.is_empty());
}

#[test]
fn abi_builder_generate_nonempty_two_alt() {
    let (g, pt) = two_alt_grammar();
    let builder = AbiLanguageBuilder::new(&g, &pt);
    assert!(!builder.generate().is_empty());
}

#[test]
fn abi_builder_generate_nonempty_chain() {
    let (g, pt) = chain_grammar();
    let builder = AbiLanguageBuilder::new(&g, &pt);
    assert!(!builder.generate().is_empty());
}

#[test]
fn abi_builder_generate_nonempty_recursive() {
    let (g, pt) = recursive_grammar();
    let builder = AbiLanguageBuilder::new(&g, &pt);
    assert!(!builder.generate().is_empty());
}

#[test]
fn abi_builder_generate_nonempty_deep_nesting() {
    let (g, pt) = deep_nesting_grammar();
    let builder = AbiLanguageBuilder::new(&g, &pt);
    assert!(!builder.generate().is_empty());
}

#[test]
fn abi_builder_generate_nonempty_precedence() {
    let (g, pt) = precedence_grammar();
    let builder = AbiLanguageBuilder::new(&g, &pt);
    assert!(!builder.generate().is_empty());
}

#[test]
fn abi_builder_generate_nonempty_mixed() {
    let (g, pt) = mixed_grammar();
    let builder = AbiLanguageBuilder::new(&g, &pt);
    assert!(!builder.generate().is_empty());
}

#[test]
fn abi_builder_with_compressed_tables() {
    let (g, pt) = simple_grammar();
    let token_indices = collect_token_indices(&g, &pt);
    let compressor = TableCompressor::new();
    let compressed = compressor.compress(&pt, &token_indices, false).unwrap();
    let builder = AbiLanguageBuilder::new(&g, &pt).with_compressed_tables(&compressed);
    let code = builder.generate();
    assert!(!code.is_empty());
}

#[test]
fn abi_builder_deterministic() {
    let (g1, pt1) = simple_grammar();
    let (g2, pt2) = simple_grammar();
    let code1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let code2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_eq!(code1, code2);
}

// =====================================================================
// AbiLanguageBuilder vs StaticLanguageGenerator — both produce output
// =====================================================================

#[test]
fn abi_and_static_both_produce_code_simple() {
    let (g, pt) = simple_grammar();
    let abi_code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let static_code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!abi_code.is_empty());
    assert!(!static_code.is_empty());
}

#[test]
fn abi_and_static_both_produce_code_multi_rule() {
    let (g, pt) = multi_rule_grammar();
    let abi_code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let static_code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!abi_code.is_empty());
    assert!(!static_code.is_empty());
}

// =====================================================================
// NodeTypesGenerator — construction (only takes grammar ref)
// =====================================================================

#[test]
fn node_types_new_simple() {
    let (g, _) = simple_grammar();
    let _ntg = NodeTypesGenerator::new(&g);
}

#[test]
fn node_types_new_empty_grammar() {
    let g = Grammar::new("empty".to_string());
    let _ntg = NodeTypesGenerator::new(&g);
}

// =====================================================================
// NodeTypesGenerator — generate() -> Result<String, String>
// =====================================================================

#[test]
fn node_types_generate_ok_simple() {
    let (g, _) = simple_grammar();
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok(), "generate should succeed for simple grammar");
}

#[test]
fn node_types_generate_ok_two_alt() {
    let (g, _) = two_alt_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn node_types_generate_ok_chain() {
    let (g, _) = chain_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn node_types_generate_ok_recursive() {
    let (g, _) = recursive_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn node_types_generate_ok_three_alt() {
    let (g, _) = three_alt_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn node_types_generate_ok_long_chain() {
    let (g, _) = long_chain_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn node_types_generate_ok_multi_rule() {
    let (g, _) = multi_rule_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn node_types_generate_ok_deep_nesting() {
    let (g, _) = deep_nesting_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn node_types_generate_ok_precedence() {
    let (g, _) = precedence_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn node_types_generate_ok_mixed() {
    let (g, _) = mixed_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn node_types_generate_ok_empty_grammar() {
    let g = Grammar::new("empty".to_string());
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// =====================================================================
// NodeTypesGenerator — output is valid JSON array
// =====================================================================

#[test]
fn node_types_output_valid_json_simple() {
    let (g, _) = simple_grammar();
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: Value = serde_json::from_str(&json_str).expect("valid JSON");
    assert!(val.is_array());
}

#[test]
fn node_types_output_valid_json_recursive() {
    let (g, _) = recursive_grammar();
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: Value = serde_json::from_str(&json_str).expect("valid JSON");
    assert!(val.is_array());
}

#[test]
fn node_types_output_valid_json_deep_nesting() {
    let (g, _) = deep_nesting_grammar();
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: Value = serde_json::from_str(&json_str).expect("valid JSON");
    assert!(val.is_array());
}

// =====================================================================
// NodeTypesGenerator — schema: every entry has "type" + "named"
// =====================================================================

#[test]
fn node_types_schema_fields_present() {
    let (g, _) = multi_rule_grammar();
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let arr: Vec<Value> = serde_json::from_str(&json_str).unwrap();
    for entry in &arr {
        assert!(entry.get("type").and_then(Value::as_str).is_some());
        assert!(entry.get("named").and_then(Value::as_bool).is_some());
    }
}

// =====================================================================
// NodeTypesGenerator — determinism
// =====================================================================

#[test]
fn node_types_deterministic_simple() {
    let (g1, _) = simple_grammar();
    let (g2, _) = simple_grammar();
    let a = NodeTypesGenerator::new(&g1).generate().unwrap();
    let b = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_eq!(a, b);
}

#[test]
fn node_types_deterministic_recursive() {
    let (g1, _) = recursive_grammar();
    let (g2, _) = recursive_grammar();
    let a = NodeTypesGenerator::new(&g1).generate().unwrap();
    let b = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_eq!(a, b);
}

// =====================================================================
// NodeTypesGenerator — increasing complexity: entry count grows
// =====================================================================

#[test]
fn node_types_more_rules_more_entries() {
    let (g_simple, _) = simple_grammar();
    let (g_multi, _) = multi_rule_grammar();
    let arr_simple: Vec<Value> =
        serde_json::from_str(&NodeTypesGenerator::new(&g_simple).generate().unwrap()).unwrap();
    let arr_multi: Vec<Value> =
        serde_json::from_str(&NodeTypesGenerator::new(&g_multi).generate().unwrap()).unwrap();
    assert!(
        arr_multi.len() >= arr_simple.len(),
        "multi_rule grammar should produce at least as many node types as simple"
    );
}

// =====================================================================
// NodeTypesGenerator — preset grammars
// =====================================================================

#[test]
fn node_types_python_like() {
    let g = GrammarBuilder::python_like();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn node_types_javascript_like() {
    let g = GrammarBuilder::javascript_like();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// =====================================================================
// Compression round-trip for various grammar shapes
// =====================================================================

#[test]
fn compress_simple_ok() {
    let (g, pt) = simple_grammar();
    let ti = collect_token_indices(&g, &pt);
    assert!(TableCompressor::new().compress(&pt, &ti, false).is_ok());
}

#[test]
fn compress_recursive_ok() {
    let (g, pt) = recursive_grammar();
    let ti = collect_token_indices(&g, &pt);
    assert!(TableCompressor::new().compress(&pt, &ti, false).is_ok());
}

#[test]
fn compress_precedence_ok() {
    let (g, pt) = precedence_grammar();
    let ti = collect_token_indices(&g, &pt);
    assert!(TableCompressor::new().compress(&pt, &ti, false).is_ok());
}
