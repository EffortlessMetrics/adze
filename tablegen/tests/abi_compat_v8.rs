//! ABI compatibility tests v8 — 84 tests across 15 categories.
//!
//! Categories:
//!   abi_build_*          — AbiLanguageBuilder produces non-empty output
//!   abi_name_*           — ABI output contains grammar name
//!   node_types_json_*    — NodeTypesGenerator produces valid JSON
//!   node_types_array_*   — NodeTypesGenerator output is a JSON array
//!   static_gen_rust_*    — StaticLanguageGenerator produces Rust code
//!   static_gen_const_*   — StaticLanguageGenerator output contains const/static
//!   compressor_new_*     — TableCompressor::new() valid
//!   compressor_from_pt_* — CompressedParseTable::from_parse_table no panic
//!   determinism_*        — same grammar → same output
//!   divergence_*         — different grammars → different outputs
//!   multi_tok_*          — multi-token grammar ABI compat
//!   prec_*               — precedence grammars
//!   conflict_*           — grammars with conflicts
//!   node_sym_*           — NodeTypes includes symbol names
//!   size_*               — various grammar sizes (1–10 rules)

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tablegen::{
    AbiLanguageBuilder, CompressedParseTable, NodeTypesGenerator, StaticLanguageGenerator,
    TableCompressor,
};

// ============================================================================
// Helpers
// ============================================================================

fn make_grammar_and_table(name: &str) -> (Grammar, ParseTable) {
    let g = GrammarBuilder::new(name)
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let mut gc = g.clone();
    let ff = FirstFollowSets::compute_normalized(&mut gc).expect("ff");
    let pt = build_lr1_automaton(&gc, &ff).expect("table");
    (g, pt)
}

fn build_table(grammar: &Grammar) -> ParseTable {
    let mut g = grammar.clone();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW");
    build_lr1_automaton(&g, &ff).expect("LR(1)")
}

fn abi_output(grammar: &Grammar, pt: &ParseTable) -> String {
    AbiLanguageBuilder::new(grammar, pt).generate().to_string()
}

fn static_output(grammar: Grammar, pt: ParseTable) -> String {
    StaticLanguageGenerator::new(grammar, pt)
        .generate_language_code()
        .to_string()
}

fn node_types_output(grammar: &Grammar) -> String {
    NodeTypesGenerator::new(grammar)
        .generate()
        .expect("node types")
}

// --- grammar factories ---

fn single_token(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

fn two_token(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
}

fn three_token(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build()
}

fn alternatives(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build()
}

fn nested(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["inner"])
        .rule("inner", vec!["x", "y"])
        .start("start")
        .build()
}

fn left_recursive(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .rule("start", vec!["a"])
        .rule("start", vec!["start", "a"])
        .start("start")
        .build()
}

fn right_recursive(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .rule("start", vec!["a"])
        .rule("start", vec!["a", "start"])
        .start("start")
        .build()
}

fn precedence_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build()
}

fn right_assoc_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("caret", r"\^")
        .rule_with_precedence(
            "expr",
            vec!["expr", "caret", "expr"],
            1,
            Associativity::Right,
        )
        .rule("expr", vec!["num"])
        .start("expr")
        .build()
}

fn conflict_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

fn n_rule_grammar(name: &str, n: usize) -> Grammar {
    let mut gb = GrammarBuilder::new(name);
    for i in 0..n {
        let tok_name = format!("t{i}");
        let pat = format!("{}", (b'a' + (i as u8 % 26)) as char);
        gb = gb.token(&tok_name, &pat);
        gb = gb.rule("start", vec![&tok_name]);
    }
    gb.start("start").build()
}

fn deep_chain(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("z", "z")
        .rule("start", vec!["layer1"])
        .rule("layer1", vec!["layer2"])
        .rule("layer2", vec!["z"])
        .start("start")
        .build()
}

fn extras_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("ws", r"\s+")
        .extra("ws")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

fn nullable_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .rule("start", vec!["opt"])
        .rule("opt", vec!["a"])
        .rule("opt", vec![])
        .start("start")
        .build()
}

// ============================================================================
// 1. abi_build_* — AbiLanguageBuilder produces non-empty output (6 tests)
// ============================================================================

#[test]
fn abi_build_single_token_nonempty() {
    let (g, pt) = make_grammar_and_table("abi_v8_build1");
    assert!(!abi_output(&g, &pt).is_empty());
}

#[test]
fn abi_build_two_token_nonempty() {
    let g = two_token("abi_v8_build2");
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

#[test]
fn abi_build_alternatives_nonempty() {
    let g = alternatives("abi_v8_build3");
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

#[test]
fn abi_build_nested_nonempty() {
    let g = nested("abi_v8_build4");
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

#[test]
fn abi_build_recursive_nonempty() {
    let g = left_recursive("abi_v8_build5");
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

#[test]
fn abi_build_nullable_nonempty() {
    let g = nullable_grammar("abi_v8_build6");
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

// ============================================================================
// 2. abi_name_* — ABI output contains grammar name (6 tests)
// ============================================================================

#[test]
fn abi_name_single_token() {
    let (g, pt) = make_grammar_and_table("abi_v8_name1");
    let out = abi_output(&g, &pt);
    assert!(out.contains("abi_v8_name1"));
}

#[test]
fn abi_name_two_token() {
    let g = two_token("abi_v8_name2");
    let pt = build_table(&g);
    assert!(abi_output(&g, &pt).contains("abi_v8_name2"));
}

#[test]
fn abi_name_alternatives() {
    let g = alternatives("abi_v8_name3");
    let pt = build_table(&g);
    assert!(abi_output(&g, &pt).contains("abi_v8_name3"));
}

#[test]
fn abi_name_nested() {
    let g = nested("abi_v8_name4");
    let pt = build_table(&g);
    assert!(abi_output(&g, &pt).contains("abi_v8_name4"));
}

#[test]
fn abi_name_precedence() {
    let g = precedence_grammar("abi_v8_name5");
    let pt = build_table(&g);
    assert!(abi_output(&g, &pt).contains("abi_v8_name5"));
}

#[test]
fn abi_name_deep_chain() {
    let g = deep_chain("abi_v8_name6");
    let pt = build_table(&g);
    assert!(abi_output(&g, &pt).contains("abi_v8_name6"));
}

// ============================================================================
// 3. node_types_json_* — NodeTypesGenerator produces valid JSON (6 tests)
// ============================================================================

#[test]
fn node_types_json_single_token() {
    let g = single_token("abi_v8_ntj1");
    let json_str = node_types_output(&g);
    let _: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
}

#[test]
fn node_types_json_two_token() {
    let g = two_token("abi_v8_ntj2");
    let json_str = node_types_output(&g);
    let _: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
}

#[test]
fn node_types_json_alternatives() {
    let g = alternatives("abi_v8_ntj3");
    let json_str = node_types_output(&g);
    let _: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
}

#[test]
fn node_types_json_nested() {
    let g = nested("abi_v8_ntj4");
    let json_str = node_types_output(&g);
    let _: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
}

#[test]
fn node_types_json_recursive() {
    let g = left_recursive("abi_v8_ntj5");
    let json_str = node_types_output(&g);
    let _: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
}

#[test]
fn node_types_json_precedence() {
    let g = precedence_grammar("abi_v8_ntj6");
    let json_str = node_types_output(&g);
    let _: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
}

// ============================================================================
// 4. node_types_array_* — NodeTypesGenerator output is a JSON array (6 tests)
// ============================================================================

#[test]
fn node_types_array_single_token() {
    let g = single_token("abi_v8_nta1");
    let v: serde_json::Value = serde_json::from_str(&node_types_output(&g)).unwrap();
    assert!(v.is_array());
}

#[test]
fn node_types_array_two_token() {
    let g = two_token("abi_v8_nta2");
    let v: serde_json::Value = serde_json::from_str(&node_types_output(&g)).unwrap();
    assert!(v.is_array());
}

#[test]
fn node_types_array_alternatives() {
    let g = alternatives("abi_v8_nta3");
    let v: serde_json::Value = serde_json::from_str(&node_types_output(&g)).unwrap();
    assert!(v.is_array());
}

#[test]
fn node_types_array_nested() {
    let g = nested("abi_v8_nta4");
    let v: serde_json::Value = serde_json::from_str(&node_types_output(&g)).unwrap();
    assert!(v.is_array());
}

#[test]
fn node_types_array_extras() {
    let g = extras_grammar("abi_v8_nta5");
    let v: serde_json::Value = serde_json::from_str(&node_types_output(&g)).unwrap();
    assert!(v.is_array());
}

#[test]
fn node_types_array_nullable() {
    let g = nullable_grammar("abi_v8_nta6");
    let v: serde_json::Value = serde_json::from_str(&node_types_output(&g)).unwrap();
    assert!(v.is_array());
}

// ============================================================================
// 5. static_gen_rust_* — StaticLanguageGenerator produces Rust code (6 tests)
// ============================================================================

#[test]
fn static_gen_rust_single_token() {
    let g = single_token("abi_v8_sgr1");
    let pt = build_table(&g);
    let code = static_output(g, pt);
    assert!(!code.is_empty());
}

#[test]
fn static_gen_rust_two_token() {
    let g = two_token("abi_v8_sgr2");
    let pt = build_table(&g);
    let code = static_output(g, pt);
    assert!(!code.is_empty());
}

#[test]
fn static_gen_rust_alternatives() {
    let g = alternatives("abi_v8_sgr3");
    let pt = build_table(&g);
    let code = static_output(g, pt);
    assert!(!code.is_empty());
}

#[test]
fn static_gen_rust_nested() {
    let g = nested("abi_v8_sgr4");
    let pt = build_table(&g);
    let code = static_output(g, pt);
    assert!(!code.is_empty());
}

#[test]
fn static_gen_rust_recursive() {
    let g = left_recursive("abi_v8_sgr5");
    let pt = build_table(&g);
    let code = static_output(g, pt);
    assert!(!code.is_empty());
}

#[test]
fn static_gen_rust_precedence() {
    let g = precedence_grammar("abi_v8_sgr6");
    let pt = build_table(&g);
    let code = static_output(g, pt);
    assert!(!code.is_empty());
}

// ============================================================================
// 6. static_gen_const_* — output contains "const" or "static" (6 tests)
// ============================================================================

#[test]
fn static_gen_const_single_token() {
    let g = single_token("abi_v8_sgc1");
    let pt = build_table(&g);
    let code = static_output(g, pt);
    assert!(code.contains("const") || code.contains("static"));
}

#[test]
fn static_gen_const_two_token() {
    let g = two_token("abi_v8_sgc2");
    let pt = build_table(&g);
    let code = static_output(g, pt);
    assert!(code.contains("const") || code.contains("static"));
}

#[test]
fn static_gen_const_alternatives() {
    let g = alternatives("abi_v8_sgc3");
    let pt = build_table(&g);
    let code = static_output(g, pt);
    assert!(code.contains("const") || code.contains("static"));
}

#[test]
fn static_gen_const_nested() {
    let g = nested("abi_v8_sgc4");
    let pt = build_table(&g);
    let code = static_output(g, pt);
    assert!(code.contains("const") || code.contains("static"));
}

#[test]
fn static_gen_const_deep_chain() {
    let g = deep_chain("abi_v8_sgc5");
    let pt = build_table(&g);
    let code = static_output(g, pt);
    assert!(code.contains("const") || code.contains("static"));
}

#[test]
fn static_gen_const_extras() {
    let g = extras_grammar("abi_v8_sgc6");
    let pt = build_table(&g);
    let code = static_output(g, pt);
    assert!(code.contains("const") || code.contains("static"));
}

// ============================================================================
// 7. compressor_new_* — TableCompressor::new() creates valid compressor (4 tests)
// ============================================================================

#[test]
fn compressor_new_default() {
    let _tc = TableCompressor::new();
}

#[test]
fn compressor_new_default_trait() {
    let _tc = TableCompressor::default();
}

#[test]
fn compressor_new_encode_shift() {
    use adze_glr_core::Action;
    use adze_ir::StateId;
    let tc = TableCompressor::new();
    let encoded = tc.encode_action_small(&Action::Shift(StateId(1)));
    assert!(encoded.is_ok());
}

#[test]
fn compressor_new_encode_accept() {
    use adze_glr_core::Action;
    let tc = TableCompressor::new();
    let encoded = tc.encode_action_small(&Action::Accept);
    assert!(encoded.is_ok());
}

// ============================================================================
// 8. compressor_from_pt_* — CompressedParseTable::from_parse_table (6 tests)
// ============================================================================

#[test]
fn compressor_from_pt_single_token() {
    let (_g, pt) = make_grammar_and_table("abi_v8_cfpt1");
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert!(cpt.symbol_count() > 0);
}

#[test]
fn compressor_from_pt_two_token() {
    let g = two_token("abi_v8_cfpt2");
    let pt = build_table(&g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert!(cpt.state_count() > 0);
}

#[test]
fn compressor_from_pt_alternatives() {
    let g = alternatives("abi_v8_cfpt3");
    let pt = build_table(&g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert!(cpt.symbol_count() > 0);
    assert!(cpt.state_count() > 0);
}

#[test]
fn compressor_from_pt_nested() {
    let g = nested("abi_v8_cfpt4");
    let pt = build_table(&g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert!(cpt.symbol_count() >= 2);
}

#[test]
fn compressor_from_pt_recursive() {
    let g = left_recursive("abi_v8_cfpt5");
    let pt = build_table(&g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert!(cpt.state_count() >= 1);
}

#[test]
fn compressor_from_pt_new_for_testing() {
    let cpt = CompressedParseTable::new_for_testing(10, 5);
    assert_eq!(cpt.symbol_count(), 10);
    assert_eq!(cpt.state_count(), 5);
}

// ============================================================================
// 9. determinism_* — same grammar → same output (6 tests)
// ============================================================================

#[test]
fn determinism_abi_single_token() {
    let g = single_token("abi_v8_det1");
    let pt1 = build_table(&g);
    let pt2 = build_table(&g);
    assert_eq!(abi_output(&g, &pt1), abi_output(&g, &pt2));
}

#[test]
fn determinism_abi_two_token() {
    let g = two_token("abi_v8_det2");
    let pt1 = build_table(&g);
    let pt2 = build_table(&g);
    assert_eq!(abi_output(&g, &pt1), abi_output(&g, &pt2));
}

#[test]
fn determinism_abi_alternatives() {
    let g = alternatives("abi_v8_det3");
    let pt1 = build_table(&g);
    let pt2 = build_table(&g);
    assert_eq!(abi_output(&g, &pt1), abi_output(&g, &pt2));
}

#[test]
fn determinism_node_types() {
    let g = single_token("abi_v8_det4");
    assert_eq!(node_types_output(&g), node_types_output(&g));
}

#[test]
fn determinism_static_gen() {
    let g1 = single_token("abi_v8_det5");
    let g2 = single_token("abi_v8_det5");
    let pt1 = build_table(&g1);
    let pt2 = build_table(&g2);
    assert_eq!(static_output(g1, pt1), static_output(g2, pt2));
}

#[test]
fn determinism_compressed_table() {
    let g = single_token("abi_v8_det6");
    let pt1 = build_table(&g);
    let pt2 = build_table(&g);
    let cpt1 = CompressedParseTable::from_parse_table(&pt1);
    let cpt2 = CompressedParseTable::from_parse_table(&pt2);
    assert_eq!(cpt1.symbol_count(), cpt2.symbol_count());
    assert_eq!(cpt1.state_count(), cpt2.state_count());
}

// ============================================================================
// 10. divergence_* — different grammars → different ABI outputs (6 tests)
// ============================================================================

#[test]
fn divergence_different_names() {
    let g1 = single_token("abi_v8_div1a");
    let g2 = single_token("abi_v8_div1b");
    let pt1 = build_table(&g1);
    let pt2 = build_table(&g2);
    assert_ne!(abi_output(&g1, &pt1), abi_output(&g2, &pt2));
}

#[test]
fn divergence_different_tokens() {
    let g1 = single_token("abi_v8_div2a");
    let g2 = two_token("abi_v8_div2b");
    let pt1 = build_table(&g1);
    let pt2 = build_table(&g2);
    assert_ne!(abi_output(&g1, &pt1), abi_output(&g2, &pt2));
}

#[test]
fn divergence_different_rules() {
    let g1 = alternatives("abi_v8_div3a");
    let g2 = nested("abi_v8_div3b");
    let pt1 = build_table(&g1);
    let pt2 = build_table(&g2);
    assert_ne!(abi_output(&g1, &pt1), abi_output(&g2, &pt2));
}

#[test]
fn divergence_node_types_different_grammars() {
    let g1 = single_token("abi_v8_div4a");
    let g2 = nested("abi_v8_div4b");
    assert_ne!(node_types_output(&g1), node_types_output(&g2));
}

#[test]
fn divergence_static_gen_different_names() {
    let g1 = single_token("abi_v8_div5a");
    let g2 = single_token("abi_v8_div5b");
    let pt1 = build_table(&g1);
    let pt2 = build_table(&g2);
    assert_ne!(static_output(g1, pt1), static_output(g2, pt2));
}

#[test]
fn divergence_compressed_different_complexity() {
    let g1 = single_token("abi_v8_div6a");
    let g2 = three_token("abi_v8_div6b");
    let pt1 = build_table(&g1);
    let pt2 = build_table(&g2);
    let cpt1 = CompressedParseTable::from_parse_table(&pt1);
    let cpt2 = CompressedParseTable::from_parse_table(&pt2);
    // Different grammar complexity leads to different table dimensions
    let same_symbols = cpt1.symbol_count() == cpt2.symbol_count();
    let same_states = cpt1.state_count() == cpt2.state_count();
    assert!(!(same_symbols && same_states));
}

// ============================================================================
// 11. multi_tok_* — multi-token grammar ABI compatibility (6 tests)
// ============================================================================

#[test]
fn multi_tok_two_abi_nonempty() {
    let g = two_token("abi_v8_mt1");
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

#[test]
fn multi_tok_three_abi_nonempty() {
    let g = three_token("abi_v8_mt2");
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

#[test]
fn multi_tok_four_tokens() {
    let g = GrammarBuilder::new("abi_v8_mt3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a", "b", "c", "d"])
        .start("start")
        .build();
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

#[test]
fn multi_tok_static_gen() {
    let g = three_token("abi_v8_mt4");
    let pt = build_table(&g);
    let code = static_output(g, pt);
    assert!(!code.is_empty());
}

#[test]
fn multi_tok_node_types() {
    let g = three_token("abi_v8_mt5");
    let json_str = node_types_output(&g);
    let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(v.is_array());
}

#[test]
fn multi_tok_compressed() {
    let g = three_token("abi_v8_mt6");
    let pt = build_table(&g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert!(cpt.symbol_count() >= 3);
}

// ============================================================================
// 12. prec_* — precedence grammars (6 tests)
// ============================================================================

#[test]
fn prec_left_abi_nonempty() {
    let g = precedence_grammar("abi_v8_prec1");
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

#[test]
fn prec_left_contains_name() {
    let g = precedence_grammar("abi_v8_prec2");
    let pt = build_table(&g);
    assert!(abi_output(&g, &pt).contains("abi_v8_prec2"));
}

#[test]
fn prec_right_abi_nonempty() {
    let g = right_assoc_grammar("abi_v8_prec3");
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

#[test]
fn prec_left_static_gen() {
    let g = precedence_grammar("abi_v8_prec4");
    let pt = build_table(&g);
    let code = static_output(g, pt);
    assert!(code.contains("const") || code.contains("static"));
}

#[test]
fn prec_right_static_gen() {
    let g = right_assoc_grammar("abi_v8_prec5");
    let pt = build_table(&g);
    let code = static_output(g, pt);
    assert!(!code.is_empty());
}

#[test]
fn prec_node_types_valid_json() {
    let g = precedence_grammar("abi_v8_prec6");
    let json_str = node_types_output(&g);
    let _: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
}

// ============================================================================
// 13. conflict_* — grammars with conflicts (6 tests)
// ============================================================================

#[test]
fn conflict_abi_nonempty() {
    let g = conflict_grammar("abi_v8_conf1");
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

#[test]
fn conflict_contains_name() {
    let g = conflict_grammar("abi_v8_conf2");
    let pt = build_table(&g);
    assert!(abi_output(&g, &pt).contains("abi_v8_conf2"));
}

#[test]
fn conflict_static_gen() {
    let g = conflict_grammar("abi_v8_conf3");
    let pt = build_table(&g);
    let code = static_output(g, pt);
    assert!(!code.is_empty());
}

#[test]
fn conflict_node_types_valid() {
    let g = conflict_grammar("abi_v8_conf4");
    let json_str = node_types_output(&g);
    let _: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
}

#[test]
fn conflict_compressed_no_panic() {
    let g = conflict_grammar("abi_v8_conf5");
    let pt = build_table(&g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert!(cpt.state_count() > 0);
}

#[test]
fn conflict_left_recursive_abi() {
    let g = left_recursive("abi_v8_conf6");
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

// ============================================================================
// 14. node_sym_* — NodeTypes includes symbol names (6 tests)
// ============================================================================

#[test]
fn node_sym_start_present() {
    let g = single_token("abi_v8_ns1");
    let json_str = node_types_output(&g);
    assert!(json_str.contains("start") || json_str.contains("type"));
}

#[test]
fn node_sym_nested_inner() {
    let g = nested("abi_v8_ns2");
    let json_str = node_types_output(&g);
    // The output should mention at least some symbol
    assert!(!json_str.is_empty());
    let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(v.is_array());
}

#[test]
fn node_sym_alternatives_has_entries() {
    let g = alternatives("abi_v8_ns3");
    let json_str = node_types_output(&g);
    let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = v.as_array().expect("array");
    assert!(!arr.is_empty());
}

#[test]
fn node_sym_has_named_field() {
    let g = single_token("abi_v8_ns4");
    let json_str = node_types_output(&g);
    let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = v.as_array().expect("array");
    // Each entry should have a "type" field
    for entry in arr {
        assert!(entry.get("type").is_some());
    }
}

#[test]
fn node_sym_has_named_bool() {
    let g = single_token("abi_v8_ns5");
    let json_str = node_types_output(&g);
    let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = v.as_array().expect("array");
    for entry in arr {
        assert!(entry.get("named").is_some());
    }
}

#[test]
fn node_sym_deep_chain_entries() {
    let g = deep_chain("abi_v8_ns6");
    let json_str = node_types_output(&g);
    let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = v.as_array().expect("array");
    assert!(!arr.is_empty());
}

// ============================================================================
// 15. size_* — various grammar sizes, 1–10 rules (10 tests)
// ============================================================================

#[test]
fn size_1_rule_abi() {
    let g = n_rule_grammar("abi_v8_sz1", 1);
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

#[test]
fn size_2_rules_abi() {
    let g = n_rule_grammar("abi_v8_sz2", 2);
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

#[test]
fn size_3_rules_abi() {
    let g = n_rule_grammar("abi_v8_sz3", 3);
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

#[test]
fn size_4_rules_abi() {
    let g = n_rule_grammar("abi_v8_sz4", 4);
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

#[test]
fn size_5_rules_abi() {
    let g = n_rule_grammar("abi_v8_sz5", 5);
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

#[test]
fn size_6_rules_static() {
    let g = n_rule_grammar("abi_v8_sz6", 6);
    let pt = build_table(&g);
    let code = static_output(g, pt);
    assert!(!code.is_empty());
}

#[test]
fn size_7_rules_node_types() {
    let g = n_rule_grammar("abi_v8_sz7", 7);
    let json_str = node_types_output(&g);
    let _: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
}

#[test]
fn size_8_rules_compressed() {
    let g = n_rule_grammar("abi_v8_sz8", 8);
    let pt = build_table(&g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert!(cpt.symbol_count() > 0);
}

#[test]
fn size_9_rules_determinism() {
    let g1 = n_rule_grammar("abi_v8_sz9", 9);
    let g2 = n_rule_grammar("abi_v8_sz9", 9);
    let pt1 = build_table(&g1);
    let pt2 = build_table(&g2);
    assert_eq!(abi_output(&g1, &pt1), abi_output(&g2, &pt2));
}

#[test]
fn size_10_rules_full_pipeline() {
    let g = n_rule_grammar("abi_v8_sz10", 10);
    let pt = build_table(&g);
    // ABI builder
    assert!(!abi_output(&g, &pt).is_empty());
    // Node types
    let json_str = node_types_output(&g);
    let _: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
    // Compressed table
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert!(cpt.state_count() > 0);
}

// ============================================================================
// Bonus — cross-cutting edge cases (4 tests)
// ============================================================================

#[test]
fn edge_right_recursive_abi() {
    let g = right_recursive("abi_v8_edge1");
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

#[test]
fn edge_extras_abi() {
    let g = extras_grammar("abi_v8_edge2");
    let pt = build_table(&g);
    assert!(!abi_output(&g, &pt).is_empty());
}

#[test]
fn edge_nullable_static_gen() {
    let g = nullable_grammar("abi_v8_edge3");
    let pt = build_table(&g);
    let code = static_output(g, pt);
    assert!(!code.is_empty());
}

#[test]
fn edge_new_for_testing_zeroes() {
    let cpt = CompressedParseTable::new_for_testing(0, 0);
    assert_eq!(cpt.symbol_count(), 0);
    assert_eq!(cpt.state_count(), 0);
}
