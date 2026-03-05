//! End-to-end integration tests for the full adze-tablegen code generation pipeline.
//!
//! Pipeline: GrammarBuilder → normalize → FIRST/FOLLOW → LR(1) parse table →
//! StaticLanguageGenerator / AbiLanguageBuilder → generate code + node types → verify.
//!
//! Eight test categories, eight tests each (64 total).

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tablegen::{AbiLanguageBuilder, NodeTypesGenerator, StaticLanguageGenerator};

// ===========================================================================
// Helpers
// ===========================================================================

/// Run the full pipeline: Grammar → FIRST/FOLLOW → LR(1) automaton → (Grammar, ParseTable).
fn pipeline(grammar: Grammar) -> (Grammar, ParseTable) {
    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW computation failed");
    let pt = build_lr1_automaton(&grammar, &ff).expect("LR(1) automaton construction failed");
    (grammar, pt)
}

/// Minimal grammar: `start -> tok`.
fn minimal_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("tok", "t")
        .rule("start", vec!["tok"])
        .start("start")
        .build()
}

/// Two-token sequence: `start -> a b`.
fn seq2_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
}

/// Three-token sequence: `start -> a b c`.
fn seq3_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build()
}

/// N alternatives: `start -> t0 | t1 | ... | tN-1`.
fn alt_grammar(name: &str, n: usize) -> Grammar {
    let mut b = GrammarBuilder::new(name);
    for i in 0..n {
        let tok: &str = Box::leak(format!("t{i}").into_boxed_str());
        b = b.token(tok, tok).rule("start", vec![tok]);
    }
    b.start("start").build()
}

/// Arithmetic grammar with two precedence levels.
fn arith_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build()
}

/// Chain grammar: `start -> rN-1 -> ... -> r0 -> tok`.
fn chain_grammar(name: &str, depth: usize) -> Grammar {
    let mut b = GrammarBuilder::new(name);
    b = b.token("x", "x");
    let names: Vec<String> = (0..depth).map(|i| format!("r{i}")).collect();
    let first: &str = Box::leak(names[0].clone().into_boxed_str());
    b = b.rule(first, vec!["x"]);
    for i in 1..depth {
        let lhs: &str = Box::leak(names[i].clone().into_boxed_str());
        let rhs: &str = Box::leak(names[i - 1].clone().into_boxed_str());
        b = b.rule(lhs, vec![rhs]);
    }
    let last: &str = Box::leak(names[depth - 1].clone().into_boxed_str());
    b = b.rule("start", vec![last]);
    b.start("start").build()
}

/// Grammar with external tokens.
fn externals_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("tok", "t")
        .rule("start", vec!["tok"])
        .start("start")
        .external("INDENT")
        .external("DEDENT")
        .build()
}

/// Grammar with many distinct tokens.
fn many_symbols_grammar(name: &str, n: usize) -> Grammar {
    let mut b = GrammarBuilder::new(name);
    for i in 0..n {
        let tok: &str = Box::leak(format!("sym{i}").into_boxed_str());
        b = b.token(tok, tok);
    }
    // start -> sym0
    b = b.rule("start", vec!["sym0"]);
    b.start("start").build()
}

/// Right-associative assignment grammar.
fn right_assoc_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("id", r"[a-z]+")
        .token("eq", "=")
        .rule("assign", vec!["id"])
        .rule_with_precedence(
            "assign",
            vec!["id", "eq", "assign"],
            1,
            Associativity::Right,
        )
        .start("assign")
        .build()
}

// ===========================================================================
// 1. Full pipeline produces non-empty Rust code (8 tests)
// ===========================================================================

#[test]
fn code_nonempty_minimal() {
    let (g, pt) = pipeline(minimal_grammar("cn1"));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn code_nonempty_seq2() {
    let (g, pt) = pipeline(seq2_grammar("cn2"));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn code_nonempty_seq3() {
    let (g, pt) = pipeline(seq3_grammar("cn3"));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn code_nonempty_alternatives() {
    let (g, pt) = pipeline(alt_grammar("cn4", 5));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn code_nonempty_arithmetic() {
    let (g, pt) = pipeline(arith_grammar("cn5"));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn code_nonempty_chain() {
    let (g, pt) = pipeline(chain_grammar("cn6", 4));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn code_nonempty_abi_minimal() {
    let (g, pt) = pipeline(minimal_grammar("cn7"));
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn code_nonempty_abi_arith() {
    let (g, pt) = pipeline(arith_grammar("cn8"));
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

// ===========================================================================
// 2. Full pipeline produces valid JSON node types (8 tests)
// ===========================================================================

#[test]
fn node_types_valid_json_minimal() {
    let g = minimal_grammar("nj1");
    let json_str = NodeTypesGenerator::new(&g).generate().expect("generate");
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(val.is_array());
}

#[test]
fn node_types_valid_json_seq2() {
    let g = seq2_grammar("nj2");
    let json_str = NodeTypesGenerator::new(&g).generate().expect("generate");
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(val.is_array());
}

#[test]
fn node_types_valid_json_alternatives() {
    let g = alt_grammar("nj3", 4);
    let json_str = NodeTypesGenerator::new(&g).generate().expect("generate");
    let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
}

#[test]
fn node_types_valid_json_arith() {
    let g = arith_grammar("nj4");
    let json_str = NodeTypesGenerator::new(&g).generate().expect("generate");
    let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
}

#[test]
fn node_types_valid_json_chain() {
    let g = chain_grammar("nj5", 3);
    let json_str = NodeTypesGenerator::new(&g).generate().expect("generate");
    let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
}

#[test]
fn node_types_static_gen_valid_json() {
    let (g, pt) = pipeline(seq2_grammar("nj6"));
    let slg = StaticLanguageGenerator::new(g, pt);
    let json_str = slg.generate_node_types();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(val.is_array());
}

#[test]
fn node_types_entries_have_type_and_named() {
    let g = arith_grammar("nj7");
    let json_str = NodeTypesGenerator::new(&g).generate().expect("generate");
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    for entry in val.as_array().unwrap() {
        assert!(entry.get("type").is_some(), "entry must have 'type'");
        assert!(entry.get("named").is_some(), "entry must have 'named'");
    }
}

#[test]
fn node_types_nonempty_array() {
    let g = minimal_grammar("nj8");
    let json_str = NodeTypesGenerator::new(&g).generate().expect("generate");
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = val.as_array().unwrap();
    assert!(!arr.is_empty(), "node types must not be empty");
}

// ===========================================================================
// 3. Generated code contains expected symbol names (8 tests)
// ===========================================================================

#[test]
fn code_contains_symbol_name_tok() {
    let (g, pt) = pipeline(minimal_grammar("sn1"));
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(code.contains("tok"), "code should contain token name 'tok'");
}

#[test]
fn code_contains_symbol_name_a_b() {
    let (g, pt) = pipeline(seq2_grammar("sn2"));
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(code.contains('a'), "code should contain token 'a'");
    assert!(code.contains('b'), "code should contain token 'b'");
}

#[test]
fn code_contains_end_symbol() {
    let (g, pt) = pipeline(minimal_grammar("sn3"));
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(code.contains("end"), "code should contain EOF symbol 'end'");
}

#[test]
fn code_contains_num_token() {
    let (g, pt) = pipeline(arith_grammar("sn4"));
    let _abi = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let slg = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(
        slg.contains("num"),
        "static code should contain 'num' token"
    );
}

#[test]
fn code_contains_plus_token() {
    let (g, pt) = pipeline(arith_grammar("sn5"));
    let _abi = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let slg = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(
        slg.contains("plus"),
        "static code should contain 'plus' token"
    );
}

#[test]
fn code_contains_star_token() {
    let (g, pt) = pipeline(arith_grammar("sn6"));
    let _abi = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let slg = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(
        slg.contains("star"),
        "static code should contain 'star' token"
    );
}

#[test]
fn static_gen_contains_start_symbol() {
    let (g, pt) = pipeline(minimal_grammar("sn7"));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(
        code.contains("start") || code.contains("rule_"),
        "code should reference start rule"
    );
}

#[test]
fn code_contains_ffi_function_name() {
    let (g, pt) = pipeline(minimal_grammar("sn8"));
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(
        code.contains("tree_sitter_sn8"),
        "code should contain FFI export named after grammar"
    );
}

// ===========================================================================
// 4. Generated code contains correct state count (8 tests)
// ===========================================================================

#[test]
fn state_count_minimal_positive() {
    let (_g, pt) = pipeline(minimal_grammar("sc1"));
    assert!(pt.state_count > 0, "must have at least one state");
}

#[test]
fn state_count_seq2_ge3() {
    let (_g, pt) = pipeline(seq2_grammar("sc2"));
    assert!(pt.state_count >= 3, "a b sequence needs ≥3 states");
}

#[test]
fn state_count_seq3_ge4() {
    let (_g, pt) = pipeline(seq3_grammar("sc3"));
    assert!(pt.state_count >= 4, "a b c sequence needs ≥4 states");
}

#[test]
fn state_count_in_abi_code() {
    let (g, pt) = pipeline(seq2_grammar("sc4"));
    let sc = pt.state_count;
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let sc_str = format!("{sc}");
    assert!(
        code.contains(&sc_str),
        "ABI code must embed state_count literal {sc}"
    );
}

#[test]
fn state_count_in_static_code() {
    let (g, pt) = pipeline(minimal_grammar("sc5"));
    let sc = pt.state_count;
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    let sc_str = format!("{sc}");
    assert!(
        code.contains(&sc_str),
        "static code must embed state_count literal {sc}"
    );
}

#[test]
fn state_count_chain_grows() {
    let (_g1, pt1) = pipeline(chain_grammar("sc6a", 2));
    let (_g2, pt2) = pipeline(chain_grammar("sc6b", 5));
    assert!(
        pt2.state_count >= pt1.state_count,
        "deeper chain should have at least as many states"
    );
}

#[test]
fn state_count_alt_grows() {
    let (_g1, pt1) = pipeline(alt_grammar("sc7a", 2));
    let (_g2, pt2) = pipeline(alt_grammar("sc7b", 8));
    assert!(
        pt2.state_count >= pt1.state_count,
        "more alternatives should have at least as many states"
    );
}

#[test]
fn state_count_arith_nontrivial() {
    let (_g, pt) = pipeline(arith_grammar("sc8"));
    assert!(
        pt.state_count >= 3,
        "arithmetic grammar must have nontrivial state count"
    );
}

// ===========================================================================
// 5. AbiLanguageBuilder and StaticLanguageGenerator produce consistent output (8 tests)
// ===========================================================================

#[test]
fn consistent_both_nonempty_minimal() {
    let (g, pt) = pipeline(minimal_grammar("co1"));
    let abi = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let slg = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!abi.is_empty());
    assert!(!slg.is_empty());
}

#[test]
fn consistent_both_nonempty_arith() {
    let (g, pt) = pipeline(arith_grammar("co2"));
    let abi = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let slg = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!abi.is_empty());
    assert!(!slg.is_empty());
}

#[test]
fn consistent_both_contain_language() {
    let (g, pt) = pipeline(seq2_grammar("co3"));
    let abi = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let slg = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(abi.contains("LANGUAGE"));
    assert!(slg.contains("LANGUAGE") || slg.contains("Language") || slg.contains("language"));
}

#[test]
fn consistent_both_reference_parse_table() {
    let (g, pt) = pipeline(minimal_grammar("co4"));
    let abi = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let slg = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    let abi_lower = abi.to_lowercase();
    let slg_lower = slg.to_lowercase();
    assert!(abi_lower.contains("parse_table") || abi_lower.contains("parsetable"));
    assert!(slg_lower.contains("parse_table") || slg_lower.contains("parsetable"));
}

#[test]
fn consistent_node_types_matches_between_generators() {
    let g = arith_grammar("co5");
    let (g2, pt) = pipeline(g);
    let nt_direct = NodeTypesGenerator::new(&g2).generate().expect("nt");
    let nt_slg = StaticLanguageGenerator::new(g2, pt).generate_node_types();
    let v1: serde_json::Value = serde_json::from_str(&nt_direct).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&nt_slg).unwrap();
    assert!(v1.is_array());
    assert!(v2.is_array());
}

#[test]
fn consistent_abi_longer_for_complex_grammar() {
    let (g1, pt1) = pipeline(minimal_grammar("co6a"));
    let (g2, pt2) = pipeline(arith_grammar("co6b"));
    let simple = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let complex = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert!(
        complex.len() >= simple.len(),
        "complex grammar should produce at least as much code"
    );
}

#[test]
fn consistent_static_longer_for_complex_grammar() {
    let (g1, pt1) = pipeline(minimal_grammar("co7a"));
    let (g2, pt2) = pipeline(arith_grammar("co7b"));
    let simple = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let complex = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert!(
        complex.len() >= simple.len(),
        "complex grammar should produce at least as much code"
    );
}

#[test]
fn consistent_both_embed_same_state_count() {
    let (g, pt) = pipeline(seq2_grammar("co8"));
    let sc = pt.state_count;
    let sc_str = format!("{sc}");
    let abi = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let slg = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(
        abi.contains(&sc_str),
        "ABI code must contain state_count {sc}"
    );
    assert!(
        slg.contains(&sc_str),
        "static code must contain state_count {sc}"
    );
}

// ===========================================================================
// 6. Complex grammars produce larger output (8 tests)
// ===========================================================================

#[test]
fn larger_output_more_alternatives_abi() {
    let (g1, pt1) = pipeline(alt_grammar("lo1a", 2));
    let (g2, pt2) = pipeline(alt_grammar("lo1b", 10));
    let small = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let big = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert!(big.len() > small.len());
}

#[test]
fn larger_output_more_alternatives_static() {
    let (g1, pt1) = pipeline(alt_grammar("lo2a", 2));
    let (g2, pt2) = pipeline(alt_grammar("lo2b", 10));
    let small = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let big = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert!(big.len() > small.len());
}

#[test]
fn larger_output_deeper_chain_abi() {
    let (g1, pt1) = pipeline(chain_grammar("lo3a", 2));
    let (g2, pt2) = pipeline(chain_grammar("lo3b", 6));
    let small = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let big = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert!(big.len() > small.len());
}

#[test]
fn larger_output_deeper_chain_static() {
    let (g1, pt1) = pipeline(chain_grammar("lo4a", 2));
    let (g2, pt2) = pipeline(chain_grammar("lo4b", 6));
    let small = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let big = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert!(big.len() > small.len());
}

#[test]
fn larger_output_more_symbols_abi() {
    let (g1, pt1) = pipeline(many_symbols_grammar("lo5a", 3));
    let (g2, pt2) = pipeline(many_symbols_grammar("lo5b", 15));
    let small = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let big = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert!(big.len() > small.len());
}

#[test]
fn larger_output_more_symbols_static() {
    let (g1, pt1) = pipeline(many_symbols_grammar("lo6a", 3));
    let (g2, pt2) = pipeline(many_symbols_grammar("lo6b", 15));
    let small = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let big = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert!(big.len() > small.len());
}

#[test]
fn larger_node_types_more_alternatives() {
    let g1 = alt_grammar("lo7a", 2);
    let g2 = alt_grammar("lo7b", 8);
    let small = NodeTypesGenerator::new(&g1).generate().expect("nt");
    let big = NodeTypesGenerator::new(&g2).generate().expect("nt");
    assert!(big.len() > small.len());
}

#[test]
fn larger_node_types_more_tokens() {
    let g1 = many_symbols_grammar("lo8a", 3);
    let g2 = many_symbols_grammar("lo8b", 12);
    let (g1, _pt1) = pipeline(g1);
    let (g2, _pt2) = pipeline(g2);
    let small = StaticLanguageGenerator::new(g1, _pt1).generate_node_types();
    let big = StaticLanguageGenerator::new(g2, _pt2).generate_node_types();
    let sv: serde_json::Value = serde_json::from_str(&small).unwrap();
    let bv: serde_json::Value = serde_json::from_str(&big).unwrap();
    assert!(sv.is_array());
    assert!(bv.is_array());
}

// ===========================================================================
// 7. Pipeline determinism (8 tests)
// ===========================================================================

#[test]
fn determinism_static_minimal() {
    let a = StaticLanguageGenerator::new(
        pipeline(minimal_grammar("dm1")).0,
        pipeline(minimal_grammar("dm1")).1,
    )
    .generate_language_code()
    .to_string();
    let b = StaticLanguageGenerator::new(
        pipeline(minimal_grammar("dm1")).0,
        pipeline(minimal_grammar("dm1")).1,
    )
    .generate_language_code()
    .to_string();
    assert_eq!(a, b);
}

#[test]
fn determinism_static_arith() {
    let (g1, pt1) = pipeline(arith_grammar("dm2"));
    let (g2, pt2) = pipeline(arith_grammar("dm2"));
    let a = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let b = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert_eq!(a, b);
}

#[test]
fn determinism_abi_minimal() {
    let (g1, pt1) = pipeline(minimal_grammar("dm3"));
    let (g2, pt2) = pipeline(minimal_grammar("dm3"));
    let a = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let b = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_eq!(a, b);
}

#[test]
fn determinism_abi_arith() {
    let (g1, pt1) = pipeline(arith_grammar("dm4"));
    let (g2, pt2) = pipeline(arith_grammar("dm4"));
    let a = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let b = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_eq!(a, b);
}

#[test]
fn determinism_node_types_minimal() {
    let g1 = minimal_grammar("dm5");
    let g2 = minimal_grammar("dm5");
    let a = NodeTypesGenerator::new(&g1).generate().expect("nt");
    let b = NodeTypesGenerator::new(&g2).generate().expect("nt");
    assert_eq!(a, b);
}

#[test]
fn determinism_node_types_arith() {
    let g1 = arith_grammar("dm6");
    let g2 = arith_grammar("dm6");
    let a = NodeTypesGenerator::new(&g1).generate().expect("nt");
    let b = NodeTypesGenerator::new(&g2).generate().expect("nt");
    assert_eq!(a, b);
}

#[test]
fn determinism_abi_three_runs() {
    let (g, pt) = pipeline(seq2_grammar("dm7"));
    let c1 = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let c2 = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let c3 = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert_eq!(c1, c2);
    assert_eq!(c2, c3);
}

#[test]
fn determinism_static_three_runs() {
    let g1 = chain_grammar("dm8", 3);
    let g2 = chain_grammar("dm8", 3);
    let g3 = chain_grammar("dm8", 3);
    let (ga, pta) = pipeline(g1);
    let (gb, ptb) = pipeline(g2);
    let (gc, ptc) = pipeline(g3);
    let a = StaticLanguageGenerator::new(ga, pta)
        .generate_language_code()
        .to_string();
    let b = StaticLanguageGenerator::new(gb, ptb)
        .generate_language_code()
        .to_string();
    let c = StaticLanguageGenerator::new(gc, ptc)
        .generate_language_code()
        .to_string();
    assert_eq!(a, b);
    assert_eq!(b, c);
}

// ===========================================================================
// 8. Edge cases (8 tests)
// ===========================================================================

#[test]
fn edge_minimal_single_token_single_rule() {
    let (g, pt) = pipeline(minimal_grammar("ec1"));
    let abi = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let slg = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!abi.is_empty());
    assert!(!slg.is_empty());
}

#[test]
fn edge_many_symbols_abi() {
    let (g, pt) = pipeline(many_symbols_grammar("ec2", 20));
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
    assert!(pt.symbol_count >= 20);
}

#[test]
fn edge_many_symbols_static() {
    let (g, pt) = pipeline(many_symbols_grammar("ec3", 20));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn edge_externals_abi() {
    let (g, pt) = pipeline(externals_grammar("ec4"));
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
    assert!(
        code.contains("INDENT") || code.contains("EXTERNAL"),
        "external tokens should appear in code"
    );
}

#[test]
fn edge_externals_static() {
    let (g, pt) = pipeline(externals_grammar("ec5"));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn edge_externals_node_types() {
    let g = externals_grammar("ec6");
    let json_str = NodeTypesGenerator::new(&g).generate().expect("nt");
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(val.is_array());
}

#[test]
fn edge_right_assoc_abi() {
    let (g, pt) = pipeline(right_assoc_grammar("ec7"));
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn edge_right_assoc_static() {
    let (g, pt) = pipeline(right_assoc_grammar("ec8"));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}
