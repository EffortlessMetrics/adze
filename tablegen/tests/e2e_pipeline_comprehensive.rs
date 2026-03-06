//! Comprehensive end-to-end pipeline tests for adze-tablegen.
//!
//! Tests the full Grammar → FIRST/FOLLOW → ParseTable → Generator pipeline
//! using all three generators: StaticLanguageGenerator, AbiLanguageBuilder,
//! and NodeTypesGenerator.

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tablegen::{AbiLanguageBuilder, NodeTypesGenerator, StaticLanguageGenerator};

// ===========================================================================
// Helpers
// ===========================================================================

/// Build a grammar and parse table from a GrammarBuilder via the full pipeline.
fn pipeline(grammar: Grammar) -> (Grammar, ParseTable) {
    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW failed");
    let pt = build_lr1_automaton(&grammar, &ff).expect("LR(1) automaton failed");
    (grammar, pt)
}

/// Shorthand: single-token grammar `start -> tok`.
fn single_token_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build()
}

/// Shorthand: two-token sequence grammar `start -> a b`.
fn two_token_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
}

/// Shorthand: grammar with N alternative rules `start -> t0 | t1 | ... | tN-1`.
fn alternatives_grammar(name: &str, n: usize) -> Grammar {
    let mut b = GrammarBuilder::new(name);
    for i in 0..n {
        let tok: &str = Box::leak(format!("t{i}").into_boxed_str());
        b = b.token(tok, tok).rule("start", vec![tok]);
    }
    b.start("start").build()
}

/// Build an arithmetic expression grammar with precedence.
fn arithmetic_grammar(name: &str) -> Grammar {
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

/// Build a chain grammar: start -> a -> b -> ... -> tok.
fn chain_grammar(name: &str, depth: usize) -> Grammar {
    let mut b = GrammarBuilder::new(name);
    b = b.token("x", "x");
    let names: Vec<String> = (0..depth).map(|i| format!("r{i}")).collect();
    // r0 -> x
    let first: &str = Box::leak(names[0].clone().into_boxed_str());
    b = b.rule(first, vec!["x"]);
    // r1 -> r0, r2 -> r1, etc.
    for i in 1..depth {
        let lhs: &str = Box::leak(names[i].clone().into_boxed_str());
        let rhs: &str = Box::leak(names[i - 1].clone().into_boxed_str());
        b = b.rule(lhs, vec![rhs]);
    }
    let last: &str = Box::leak(names[depth - 1].clone().into_boxed_str());
    b = b.rule("start", vec![last]);
    b.start("start").build()
}

// ===========================================================================
// 1. Full pipeline: Grammar → FF → PT → StaticLanguageGenerator
// ===========================================================================

#[test]
fn static_gen_single_token_produces_code() {
    let (g, pt) = pipeline(single_token_grammar("sg1"));
    let generator = StaticLanguageGenerator::new(g, pt);
    let code = generator.generate_language_code().to_string();
    assert!(!code.is_empty(), "generated code must not be empty");
}

#[test]
fn static_gen_two_token_sequence() {
    let (g, pt) = pipeline(two_token_grammar("sg2"));
    let generator = StaticLanguageGenerator::new(g, pt);
    let code = generator.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn static_gen_three_alternatives() {
    let (g, pt) = pipeline(alternatives_grammar("sg3", 3));
    let generator = StaticLanguageGenerator::new(g, pt);
    let code = generator.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn static_gen_chain_depth_4() {
    let (g, pt) = pipeline(chain_grammar("sg4", 4));
    let generator = StaticLanguageGenerator::new(g, pt);
    let code = generator.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn static_gen_arithmetic_precedence() {
    let (g, pt) = pipeline(arithmetic_grammar("sg5"));
    let generator = StaticLanguageGenerator::new(g, pt);
    let code = generator.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn static_gen_node_types_valid_json() {
    let (g, pt) = pipeline(single_token_grammar("sg6"));
    let generator = StaticLanguageGenerator::new(g, pt);
    let json_str = generator.generate_node_types();
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("node types must be valid JSON");
    assert!(parsed.is_array());
}

#[test]
fn static_gen_set_start_can_be_empty() {
    let (g, pt) = pipeline(single_token_grammar("sg7"));
    let mut generator = StaticLanguageGenerator::new(g, pt);
    generator.set_start_can_be_empty(true);
    assert!(generator.start_can_be_empty);
    let code = generator.generate_language_code().to_string();
    assert!(!code.is_empty());
}

// ===========================================================================
// 2. Full pipeline: Grammar → FF → PT → AbiLanguageBuilder
// ===========================================================================

#[test]
fn abi_single_token_produces_code() {
    let (g, pt) = pipeline(single_token_grammar("ab1"));
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn abi_two_token_sequence() {
    let (g, pt) = pipeline(two_token_grammar("ab2"));
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn abi_three_alternatives() {
    let (g, pt) = pipeline(alternatives_grammar("ab3", 3));
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn abi_chain_depth_4() {
    let (g, pt) = pipeline(chain_grammar("ab4", 4));
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn abi_arithmetic_precedence() {
    let (g, pt) = pipeline(arithmetic_grammar("ab5"));
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn abi_contains_language_struct() {
    let (g, pt) = pipeline(single_token_grammar("ab6"));
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(code.contains("LANGUAGE"), "ABI code must define LANGUAGE");
}

#[test]
fn abi_contains_ffi_function_with_grammar_name() {
    let (g, pt) = pipeline(single_token_grammar("myffi"));
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(
        code.contains("tree_sitter_myffi"),
        "ABI code must contain FFI function named after grammar"
    );
}

#[test]
fn abi_contains_parse_table() {
    let (g, pt) = pipeline(two_token_grammar("ab8"));
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(
        code.contains("PARSE_TABLE") || code.contains("parse_table"),
        "ABI code must reference parse table"
    );
}

// ===========================================================================
// 3. Full pipeline: Grammar → NodeTypesGenerator
// ===========================================================================

#[test]
fn node_types_single_token() {
    let g = single_token_grammar("nt1");
    let generator = NodeTypesGenerator::new(&g);
    let json_str = generator.generate().expect("generate must succeed");
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(val.is_array());
}

#[test]
fn node_types_two_token() {
    let g = two_token_grammar("nt2");
    let generator = NodeTypesGenerator::new(&g);
    let json_str = generator.generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(val.is_array());
}

#[test]
fn node_types_alternatives() {
    let g = alternatives_grammar("nt3", 4);
    let generator = NodeTypesGenerator::new(&g);
    let json_str = generator.generate().unwrap();
    let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
}

#[test]
fn node_types_arithmetic() {
    let g = arithmetic_grammar("nt4");
    let generator = NodeTypesGenerator::new(&g);
    let json_str = generator.generate().unwrap();
    let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
}

#[test]
fn node_types_chain() {
    let g = chain_grammar("nt5", 3);
    let generator = NodeTypesGenerator::new(&g);
    let json_str = generator.generate().unwrap();
    let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
}

#[test]
fn node_types_json_is_nonempty_array() {
    let g = single_token_grammar("nt6");
    let generator = NodeTypesGenerator::new(&g);
    let json_str = generator.generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    // At minimum the grammar has rule(s) and token(s)
    assert!(!val.as_array().unwrap().is_empty());
}

// ===========================================================================
// 4. All three generators on the same grammar
// ===========================================================================

#[test]
fn all_three_generators_single_token() {
    let g = single_token_grammar("all1");
    let (g2, pt) = pipeline(g.clone());

    // NodeTypesGenerator (no parse table needed)
    let nt = NodeTypesGenerator::new(&g2);
    let json_str = nt.generate().unwrap();
    let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // AbiLanguageBuilder
    let abi_code = AbiLanguageBuilder::new(&g2, &pt).generate().to_string();
    assert!(!abi_code.is_empty());

    // StaticLanguageGenerator (consumes grammar + table)
    let slg = StaticLanguageGenerator::new(g2, pt);
    let slg_code = slg.generate_language_code().to_string();
    assert!(!slg_code.is_empty());
}

#[test]
fn all_three_generators_arithmetic() {
    let g = arithmetic_grammar("all2");
    let (g2, pt) = pipeline(g.clone());

    let nt_json = NodeTypesGenerator::new(&g2).generate().unwrap();
    let _: serde_json::Value = serde_json::from_str(&nt_json).unwrap();

    let abi_code = AbiLanguageBuilder::new(&g2, &pt).generate().to_string();
    assert!(!abi_code.is_empty());

    let slg = StaticLanguageGenerator::new(g2, pt);
    let slg_code = slg.generate_language_code().to_string();
    assert!(!slg_code.is_empty());
}

#[test]
fn all_three_generators_chain() {
    let g = chain_grammar("all3", 5);
    let (g2, pt) = pipeline(g.clone());

    let nt_json = NodeTypesGenerator::new(&g2).generate().unwrap();
    let _: serde_json::Value = serde_json::from_str(&nt_json).unwrap();

    let abi_code = AbiLanguageBuilder::new(&g2, &pt).generate().to_string();
    assert!(!abi_code.is_empty());

    let slg = StaticLanguageGenerator::new(g2, pt);
    assert!(!slg.generate_language_code().to_string().is_empty());
}

#[test]
fn all_three_generators_alternatives() {
    let g = alternatives_grammar("all4", 6);
    let (g2, pt) = pipeline(g.clone());

    let nt_json = NodeTypesGenerator::new(&g2).generate().unwrap();
    let _: serde_json::Value = serde_json::from_str(&nt_json).unwrap();

    let abi_code = AbiLanguageBuilder::new(&g2, &pt).generate().to_string();
    assert!(!abi_code.is_empty());

    let slg = StaticLanguageGenerator::new(g2, pt);
    assert!(!slg.generate_language_code().to_string().is_empty());
}

// ===========================================================================
// 5. Various grammar sizes through pipeline
// ===========================================================================

#[test]
fn pipeline_1_alternative() {
    let (g, pt) = pipeline(alternatives_grammar("sz1", 1));
    assert!(pt.state_count > 0);
    assert!(
        !AbiLanguageBuilder::new(&g, &pt)
            .generate()
            .to_string()
            .is_empty()
    );
}

#[test]
fn pipeline_5_alternatives() {
    let (g, pt) = pipeline(alternatives_grammar("sz5", 5));
    assert!(pt.rules.len() >= 5);
    assert!(
        !AbiLanguageBuilder::new(&g, &pt)
            .generate()
            .to_string()
            .is_empty()
    );
}

#[test]
fn pipeline_10_alternatives() {
    let (g, pt) = pipeline(alternatives_grammar("sz10", 10));
    assert!(pt.rules.len() >= 10);
    let generator = StaticLanguageGenerator::new(g, pt);
    assert!(!generator.generate_language_code().to_string().is_empty());
}

#[test]
fn pipeline_20_alternatives() {
    let (g, pt) = pipeline(alternatives_grammar("sz20", 20));
    assert!(pt.rules.len() >= 20);
    let generator = StaticLanguageGenerator::new(g, pt);
    assert!(!generator.generate_language_code().to_string().is_empty());
}

#[test]
fn pipeline_chain_depth_2() {
    let (g, pt) = pipeline(chain_grammar("cd2", 2));
    assert!(pt.state_count > 0);
    assert!(
        !AbiLanguageBuilder::new(&g, &pt)
            .generate()
            .to_string()
            .is_empty()
    );
}

#[test]
fn pipeline_chain_depth_6() {
    let (g, pt) = pipeline(chain_grammar("cd6", 6));
    assert!(pt.rules.len() >= 6);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn pipeline_chain_depth_10() {
    let (g, pt) = pipeline(chain_grammar("cd10", 10));
    assert!(pt.rules.len() >= 10);
    let generator = StaticLanguageGenerator::new(g, pt);
    assert!(!generator.generate_language_code().to_string().is_empty());
}

// ===========================================================================
// 6. Pipeline with precedence
// ===========================================================================

#[test]
fn precedence_left_assoc_through_pipeline() {
    let g = GrammarBuilder::new("prec1")
        .token("n", "n")
        .token("plus", r"\+")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .start("e")
        .build();
    let (g, pt) = pipeline(g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn precedence_right_assoc_through_pipeline() {
    let g = GrammarBuilder::new("prec2")
        .token("n", "n")
        .token("eq", "=")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "eq", "e"], 1, Associativity::Right)
        .start("e")
        .build();
    let (g, pt) = pipeline(g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn precedence_two_levels_through_pipeline() {
    let g = arithmetic_grammar("prec3");
    let (g, pt) = pipeline(g);
    assert!(pt.rules.len() >= 3);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn precedence_static_gen_produces_code() {
    let g = arithmetic_grammar("prec4");
    let (g, pt) = pipeline(g);
    let generator = StaticLanguageGenerator::new(g, pt);
    assert!(!generator.generate_language_code().to_string().is_empty());
}

#[test]
fn precedence_node_types_valid_json() {
    let g = arithmetic_grammar("prec5");
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let _: serde_json::Value = serde_json::from_str(&json_str).unwrap();
}

// ===========================================================================
// 7. Pipeline determinism
// ===========================================================================

#[test]
fn static_gen_deterministic_across_runs() {
    let g1 = single_token_grammar("det1");
    let g2 = single_token_grammar("det1");
    let (ga, pta) = pipeline(g1);
    let (gb, ptb) = pipeline(g2);
    let code_a = StaticLanguageGenerator::new(ga, pta)
        .generate_language_code()
        .to_string();
    let code_b = StaticLanguageGenerator::new(gb, ptb)
        .generate_language_code()
        .to_string();
    assert_eq!(
        code_a, code_b,
        "identical grammars must produce identical code"
    );
}

#[test]
fn abi_deterministic_across_runs() {
    let g1 = two_token_grammar("det2");
    let g2 = two_token_grammar("det2");
    let (ga, pta) = pipeline(g1);
    let (gb, ptb) = pipeline(g2);
    let code_a = AbiLanguageBuilder::new(&ga, &pta).generate().to_string();
    let code_b = AbiLanguageBuilder::new(&gb, &ptb).generate().to_string();
    assert_eq!(code_a, code_b);
}

#[test]
fn node_types_deterministic_across_runs() {
    let g1 = arithmetic_grammar("det3");
    let g2 = arithmetic_grammar("det3");
    let json_a = NodeTypesGenerator::new(&g1).generate().unwrap();
    let json_b = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_eq!(json_a, json_b);
}

#[test]
fn abi_deterministic_three_calls_same_input() {
    let g = alternatives_grammar("det4", 4);
    let (g, pt) = pipeline(g);
    let c1 = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let c2 = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let c3 = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert_eq!(c1, c2);
    assert_eq!(c2, c3);
}

#[test]
fn static_gen_deterministic_arithmetic() {
    let g1 = arithmetic_grammar("det5");
    let g2 = arithmetic_grammar("det5");
    let (ga, pta) = pipeline(g1);
    let (gb, ptb) = pipeline(g2);
    let a = StaticLanguageGenerator::new(ga, pta)
        .generate_language_code()
        .to_string();
    let b = StaticLanguageGenerator::new(gb, ptb)
        .generate_language_code()
        .to_string();
    assert_eq!(a, b);
}

// ===========================================================================
// 8. Generated code non-empty checks
// ===========================================================================

#[test]
fn static_gen_code_has_multiple_tokens() {
    let (g, pt) = pipeline(single_token_grammar("ne1"));
    let generator = StaticLanguageGenerator::new(g, pt);
    let ts = generator.generate_language_code();
    // TokenStream should have multiple token trees
    assert!(ts.into_iter().count() > 1);
}

#[test]
fn abi_code_has_multiple_tokens() {
    let (g, pt) = pipeline(single_token_grammar("ne2"));
    let ts = AbiLanguageBuilder::new(&g, &pt).generate();
    assert!(ts.into_iter().count() > 1);
}

#[test]
fn static_gen_code_contains_symbol_count() {
    let (g, pt) = pipeline(two_token_grammar("ne3"));
    let generator = StaticLanguageGenerator::new(g, pt);
    let code = generator.generate_language_code().to_string();
    assert!(
        code.contains("symbol_count") || code.contains("SYMBOL"),
        "code should reference symbol metadata"
    );
}

#[test]
fn abi_code_contains_version() {
    let (g, pt) = pipeline(single_token_grammar("ne4"));
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(
        code.contains("LANGUAGE_VERSION") || code.contains("version"),
        "ABI code should reference language version"
    );
}

// ===========================================================================
// 9. Node types JSON validity
// ===========================================================================

#[test]
fn node_types_json_parses_as_array() {
    let g = single_token_grammar("jv1");
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(val.is_array(), "top-level must be array");
}

#[test]
fn node_types_entries_have_type_field() {
    let g = two_token_grammar("jv2");
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    for entry in val.as_array().unwrap() {
        assert!(
            entry.get("type").is_some(),
            "each entry must have a 'type' field"
        );
    }
}

#[test]
fn node_types_entries_have_named_field() {
    let g = single_token_grammar("jv3");
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    for entry in val.as_array().unwrap() {
        assert!(
            entry.get("named").is_some(),
            "each entry must have a 'named' field"
        );
    }
}

#[test]
fn node_types_static_gen_also_valid_json() {
    let (g, pt) = pipeline(two_token_grammar("jv4"));
    let generator = StaticLanguageGenerator::new(g, pt);
    let json_str = generator.generate_node_types();
    let _: serde_json::Value =
        serde_json::from_str(&json_str).expect("StaticLanguageGenerator node types must be JSON");
}

#[test]
fn node_types_arithmetic_entries_are_objects() {
    let g = arithmetic_grammar("jv5");
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    for entry in val.as_array().unwrap() {
        assert!(entry.is_object(), "each entry must be a JSON object");
    }
}

// ===========================================================================
// 10. Multiple pipelines in sequence
// ===========================================================================

#[test]
fn sequential_pipelines_independent_results() {
    let (g1, pt1) = pipeline(single_token_grammar("seq1"));
    let (g2, pt2) = pipeline(two_token_grammar("seq2"));

    let code1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let code2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();

    assert_ne!(
        code1, code2,
        "different grammars should produce different code"
    );
}

#[test]
fn sequential_pipelines_static_gen() {
    let (g1, pt1) = pipeline(single_token_grammar("seq3"));
    let (g2, pt2) = pipeline(arithmetic_grammar("seq4"));

    let c1 = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();

    assert_ne!(c1, c2);
}

#[test]
fn sequential_pipelines_node_types() {
    let g1 = single_token_grammar("seq5");
    let g2 = arithmetic_grammar("seq6");

    let j1 = NodeTypesGenerator::new(&g1).generate().unwrap();
    let j2 = NodeTypesGenerator::new(&g2).generate().unwrap();

    // Both valid JSON but different content
    let _: serde_json::Value = serde_json::from_str(&j1).unwrap();
    let _: serde_json::Value = serde_json::from_str(&j2).unwrap();
    assert_ne!(j1, j2);
}

#[test]
fn five_pipelines_all_succeed() {
    for i in 0..5 {
        let name: &str = Box::leak(format!("batch{i}").into_boxed_str());
        let g = alternatives_grammar(name, i + 1);
        let (g, pt) = pipeline(g);
        let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
        assert!(!code.is_empty(), "pipeline {i} must produce code");
    }
}

#[test]
fn sequential_all_three_then_repeat() {
    // First pass
    let g = single_token_grammar("rep1");
    let (g, pt) = pipeline(g);
    let abi1 = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let nt1 = NodeTypesGenerator::new(&g).generate().unwrap();
    let slg1 = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();

    // Second pass (fresh grammar)
    let g = single_token_grammar("rep1");
    let (g, pt) = pipeline(g);
    let abi2 = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let nt2 = NodeTypesGenerator::new(&g).generate().unwrap();
    let slg2 = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();

    assert_eq!(abi1, abi2);
    assert_eq!(nt1, nt2);
    assert_eq!(slg1, slg2);
}

// ===========================================================================
// Bonus: Edge cases and extra coverage
// ===========================================================================

#[test]
fn recursive_grammar_through_pipeline() {
    let g = GrammarBuilder::new("rec1")
        .token("a", "a")
        .rule("lst", vec!["a", "lst"])
        .rule("lst", vec!["a"])
        .rule("start", vec!["lst"])
        .start("start")
        .build();
    let (g, pt) = pipeline(g);
    assert!(pt.state_count > 0);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn grammar_with_extra_token_through_pipeline() {
    let g = GrammarBuilder::new("ext1")
        .token("id", r"[a-z]+")
        .token("ws", r"\s+")
        .extra("ws")
        .rule("start", vec!["id"])
        .start("start")
        .build();
    let (g, pt) = pipeline(g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn ffi_function_name_matches_grammar_name() {
    let (g, pt) = pipeline(
        GrammarBuilder::new("my_lang")
            .token("x", "x")
            .rule("start", vec!["x"])
            .start("start")
            .build(),
    );
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(code.contains("tree_sitter_my_lang"));
}

#[test]
fn different_names_different_ffi() {
    let (g1, pt1) = pipeline(single_token_grammar("alpha"));
    let (g2, pt2) = pipeline(single_token_grammar("beta"));
    let c1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let c2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert!(c1.contains("tree_sitter_alpha"));
    assert!(c2.contains("tree_sitter_beta"));
    assert!(!c1.contains("tree_sitter_beta"));
    assert!(!c2.contains("tree_sitter_alpha"));
}

#[test]
fn parse_table_state_count_positive() {
    let (_, pt) = pipeline(single_token_grammar("sc1"));
    assert!(pt.state_count > 0);
}

#[test]
fn parse_table_rules_nonempty() {
    let (_, pt) = pipeline(single_token_grammar("sc2"));
    assert!(!pt.rules.is_empty());
}

#[test]
fn parse_table_symbol_metadata_populated() {
    let (_, pt) = pipeline(two_token_grammar("sc3"));
    assert!(!pt.symbol_metadata.is_empty());
}

#[test]
fn static_gen_preserves_grammar_name() {
    let (g, pt) = pipeline(
        GrammarBuilder::new("preserved")
            .token("x", "x")
            .rule("start", vec!["x"])
            .start("start")
            .build(),
    );
    let generator = StaticLanguageGenerator::new(g, pt);
    assert_eq!(generator.grammar.name, "preserved");
}

#[test]
fn static_gen_compressed_tables_none_by_default() {
    let (g, pt) = pipeline(single_token_grammar("comp1"));
    let generator = StaticLanguageGenerator::new(g, pt);
    assert!(generator.compressed_tables.is_none());
}

#[test]
fn node_types_for_grammar_with_multiple_rules() {
    let g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .rule("foo", vec!["a"])
        .rule("bar", vec!["b"])
        .rule("start", vec!["foo", "bar"])
        .start("start")
        .build();
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(val.as_array().unwrap().len() >= 2);
}
