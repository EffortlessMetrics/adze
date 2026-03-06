//! Comprehensive determinism and consistency tests for the tablegen pipeline.
//!
//! Verifies that:
//! - Same grammar always produces same output (determinism)
//! - Different grammars produce different output (differentiation)
//! - Output is consistent across multiple runs (stability)

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::{AbiLanguageBuilder, NodeTypesGenerator, StaticLanguageGenerator};

// =====================================================================
// Helpers
// =====================================================================

fn build_pipeline(mut grammar: Grammar) -> (Grammar, adze_glr_core::ParseTable) {
    let ff =
        FirstFollowSets::compute_normalized(&mut grammar).expect("FIRST/FOLLOW computation failed");
    let pt = build_lr1_automaton(&grammar, &ff).expect("LR(1) automaton build failed");
    (grammar, pt)
}

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

fn two_alt_grammar() -> Grammar {
    GrammarBuilder::new("two_alt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build()
}

fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x", "y"])
        .start("s")
        .build()
}

fn recursive_grammar() -> Grammar {
    GrammarBuilder::new("recursive")
        .token("a", "a")
        .rule("s", vec!["a"])
        .rule("s", vec!["s", "a"])
        .start("s")
        .build()
}

fn three_alt_grammar() -> Grammar {
    GrammarBuilder::new("three_alt")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .rule("s", vec!["z"])
        .start("s")
        .build()
}

fn nested_grammar() -> Grammar {
    GrammarBuilder::new("nested")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("outer", vec!["inner"])
        .rule("s", vec!["outer"])
        .start("s")
        .build()
}

fn multi_rule_grammar() -> Grammar {
    GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .rule("z", vec!["c"])
        .rule("s", vec!["x", "y", "z"])
        .start("s")
        .build()
}

fn deep_chain_grammar() -> Grammar {
    GrammarBuilder::new("deep")
        .token("t", "t")
        .rule("d", vec!["t"])
        .rule("c", vec!["d"])
        .rule("b", vec!["c"])
        .rule("a", vec!["b"])
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

// =====================================================================
// 1. StaticLanguageGenerator determinism (10 tests)
// =====================================================================

#[test]
fn static_gen_determinism_simple_code() {
    let (g1, pt1) = build_pipeline(simple_grammar());
    let (g2, pt2) = build_pipeline(simple_grammar());
    let out1 = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let out2 = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert_eq!(out1, out2);
}

#[test]
fn static_gen_determinism_two_alt_code() {
    let (g1, pt1) = build_pipeline(two_alt_grammar());
    let (g2, pt2) = build_pipeline(two_alt_grammar());
    let out1 = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let out2 = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert_eq!(out1, out2);
}

#[test]
fn static_gen_determinism_chain_code() {
    let (g1, pt1) = build_pipeline(chain_grammar());
    let (g2, pt2) = build_pipeline(chain_grammar());
    let out1 = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let out2 = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert_eq!(out1, out2);
}

#[test]
fn static_gen_determinism_recursive_code() {
    let (g1, pt1) = build_pipeline(recursive_grammar());
    let (g2, pt2) = build_pipeline(recursive_grammar());
    let out1 = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let out2 = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert_eq!(out1, out2);
}

#[test]
fn static_gen_determinism_three_alt_code() {
    let (g1, pt1) = build_pipeline(three_alt_grammar());
    let (g2, pt2) = build_pipeline(three_alt_grammar());
    let out1 = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let out2 = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert_eq!(out1, out2);
}

#[test]
fn static_gen_determinism_nested_code() {
    let (g1, pt1) = build_pipeline(nested_grammar());
    let (g2, pt2) = build_pipeline(nested_grammar());
    let out1 = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let out2 = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert_eq!(out1, out2);
}

#[test]
fn static_gen_determinism_multi_rule_code() {
    let (g1, pt1) = build_pipeline(multi_rule_grammar());
    let (g2, pt2) = build_pipeline(multi_rule_grammar());
    let out1 = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let out2 = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert_eq!(out1, out2);
}

#[test]
fn static_gen_determinism_deep_chain_code() {
    let (g1, pt1) = build_pipeline(deep_chain_grammar());
    let (g2, pt2) = build_pipeline(deep_chain_grammar());
    let out1 = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let out2 = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert_eq!(out1, out2);
}

#[test]
fn static_gen_determinism_node_types_simple() {
    let (g1, pt1) = build_pipeline(simple_grammar());
    let (g2, pt2) = build_pipeline(simple_grammar());
    let out1 = StaticLanguageGenerator::new(g1, pt1).generate_node_types();
    let out2 = StaticLanguageGenerator::new(g2, pt2).generate_node_types();
    assert_eq!(out1, out2);
}

#[test]
fn static_gen_determinism_five_runs() {
    let outputs: Vec<String> = (0..5)
        .map(|_| {
            let (g, pt) = build_pipeline(multi_rule_grammar());
            StaticLanguageGenerator::new(g, pt)
                .generate_language_code()
                .to_string()
        })
        .collect();
    for i in 1..outputs.len() {
        assert_eq!(outputs[0], outputs[i], "run 0 vs run {i} differ");
    }
}

// =====================================================================
// 2. NodeTypesGenerator determinism (8 tests)
// =====================================================================

#[test]
fn node_types_determinism_simple() {
    let g1 = simple_grammar();
    let g2 = simple_grammar();
    let out1 = NodeTypesGenerator::new(&g1).generate().unwrap();
    let out2 = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_eq!(out1, out2);
}

#[test]
fn node_types_determinism_two_alt() {
    let g1 = two_alt_grammar();
    let g2 = two_alt_grammar();
    let out1 = NodeTypesGenerator::new(&g1).generate().unwrap();
    let out2 = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_eq!(out1, out2);
}

#[test]
fn node_types_determinism_chain() {
    let g1 = chain_grammar();
    let g2 = chain_grammar();
    let out1 = NodeTypesGenerator::new(&g1).generate().unwrap();
    let out2 = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_eq!(out1, out2);
}

#[test]
fn node_types_determinism_recursive() {
    let g1 = recursive_grammar();
    let g2 = recursive_grammar();
    let out1 = NodeTypesGenerator::new(&g1).generate().unwrap();
    let out2 = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_eq!(out1, out2);
}

#[test]
fn node_types_determinism_three_alt() {
    let g1 = three_alt_grammar();
    let g2 = three_alt_grammar();
    let out1 = NodeTypesGenerator::new(&g1).generate().unwrap();
    let out2 = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_eq!(out1, out2);
}

#[test]
fn node_types_determinism_nested() {
    let g1 = nested_grammar();
    let g2 = nested_grammar();
    let out1 = NodeTypesGenerator::new(&g1).generate().unwrap();
    let out2 = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_eq!(out1, out2);
}

#[test]
fn node_types_determinism_multi_rule() {
    let g1 = multi_rule_grammar();
    let g2 = multi_rule_grammar();
    let out1 = NodeTypesGenerator::new(&g1).generate().unwrap();
    let out2 = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_eq!(out1, out2);
}

#[test]
fn node_types_determinism_five_runs() {
    let outputs: Vec<String> = (0..5)
        .map(|_| {
            let g = deep_chain_grammar();
            NodeTypesGenerator::new(&g).generate().unwrap()
        })
        .collect();
    for i in 1..outputs.len() {
        assert_eq!(outputs[0], outputs[i], "run 0 vs run {i} differ");
    }
}

// =====================================================================
// 3. AbiLanguageBuilder determinism (8 tests)
// =====================================================================

#[test]
fn abi_determinism_simple() {
    let (g1, pt1) = build_pipeline(simple_grammar());
    let (g2, pt2) = build_pipeline(simple_grammar());
    let out1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let out2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_eq!(out1, out2);
}

#[test]
fn abi_determinism_two_alt() {
    let (g1, pt1) = build_pipeline(two_alt_grammar());
    let (g2, pt2) = build_pipeline(two_alt_grammar());
    let out1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let out2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_eq!(out1, out2);
}

#[test]
fn abi_determinism_chain() {
    let (g1, pt1) = build_pipeline(chain_grammar());
    let (g2, pt2) = build_pipeline(chain_grammar());
    let out1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let out2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_eq!(out1, out2);
}

#[test]
fn abi_determinism_recursive() {
    let (g1, pt1) = build_pipeline(recursive_grammar());
    let (g2, pt2) = build_pipeline(recursive_grammar());
    let out1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let out2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_eq!(out1, out2);
}

#[test]
fn abi_determinism_three_alt() {
    let (g1, pt1) = build_pipeline(three_alt_grammar());
    let (g2, pt2) = build_pipeline(three_alt_grammar());
    let out1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let out2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_eq!(out1, out2);
}

#[test]
fn abi_determinism_nested() {
    let (g1, pt1) = build_pipeline(nested_grammar());
    let (g2, pt2) = build_pipeline(nested_grammar());
    let out1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let out2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_eq!(out1, out2);
}

#[test]
fn abi_determinism_multi_rule() {
    let (g1, pt1) = build_pipeline(multi_rule_grammar());
    let (g2, pt2) = build_pipeline(multi_rule_grammar());
    let out1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let out2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_eq!(out1, out2);
}

#[test]
fn abi_determinism_five_runs() {
    let outputs: Vec<String> = (0..5)
        .map(|_| {
            let (g, pt) = build_pipeline(deep_chain_grammar());
            AbiLanguageBuilder::new(&g, &pt).generate().to_string()
        })
        .collect();
    for i in 1..outputs.len() {
        assert_eq!(outputs[0], outputs[i], "run 0 vs run {i} differ");
    }
}

// =====================================================================
// 4. Cross-grammar differentiation (8 tests)
// =====================================================================

#[test]
fn diff_static_code_simple_vs_two_alt() {
    let (g1, pt1) = build_pipeline(simple_grammar());
    let (g2, pt2) = build_pipeline(two_alt_grammar());
    let out1 = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let out2 = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert_ne!(
        out1, out2,
        "different grammars should produce different code"
    );
}

#[test]
fn diff_static_code_simple_vs_chain() {
    let (g1, pt1) = build_pipeline(simple_grammar());
    let (g2, pt2) = build_pipeline(chain_grammar());
    let out1 = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let out2 = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert_ne!(
        out1, out2,
        "different grammars should produce different code"
    );
}

#[test]
fn diff_static_code_chain_vs_recursive() {
    let (g1, pt1) = build_pipeline(chain_grammar());
    let (g2, pt2) = build_pipeline(recursive_grammar());
    let out1 = StaticLanguageGenerator::new(g1, pt1)
        .generate_language_code()
        .to_string();
    let out2 = StaticLanguageGenerator::new(g2, pt2)
        .generate_language_code()
        .to_string();
    assert_ne!(
        out1, out2,
        "different grammars should produce different code"
    );
}

#[test]
fn diff_node_types_simple_vs_two_alt() {
    let g1 = simple_grammar();
    let g2 = two_alt_grammar();
    let out1 = NodeTypesGenerator::new(&g1).generate().unwrap();
    let out2 = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_ne!(
        out1, out2,
        "different grammars should produce different node types"
    );
}

#[test]
fn diff_node_types_simple_vs_nested() {
    let g1 = simple_grammar();
    let g2 = nested_grammar();
    let out1 = NodeTypesGenerator::new(&g1).generate().unwrap();
    let out2 = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_ne!(
        out1, out2,
        "different grammars should produce different node types"
    );
}

#[test]
fn diff_abi_simple_vs_two_alt() {
    let (g1, pt1) = build_pipeline(simple_grammar());
    let (g2, pt2) = build_pipeline(two_alt_grammar());
    let out1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let out2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_ne!(
        out1, out2,
        "different grammars should produce different ABI"
    );
}

#[test]
fn diff_abi_chain_vs_recursive() {
    let (g1, pt1) = build_pipeline(chain_grammar());
    let (g2, pt2) = build_pipeline(recursive_grammar());
    let out1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let out2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_ne!(
        out1, out2,
        "different grammars should produce different ABI"
    );
}

#[test]
fn diff_abi_simple_vs_multi_rule() {
    let (g1, pt1) = build_pipeline(simple_grammar());
    let (g2, pt2) = build_pipeline(multi_rule_grammar());
    let out1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let out2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_ne!(
        out1, out2,
        "different grammars should produce different ABI"
    );
}

// =====================================================================
// 5. Pipeline consistency (8 tests)
// =====================================================================

#[test]
fn pipeline_consistency_simple_all_generators() {
    let (g1, pt1) = build_pipeline(simple_grammar());
    let (g2, pt2) = build_pipeline(simple_grammar());

    let code1 = StaticLanguageGenerator::new(g1.clone(), pt1.clone())
        .generate_language_code()
        .to_string();
    let code2 = StaticLanguageGenerator::new(g2.clone(), pt2.clone())
        .generate_language_code()
        .to_string();
    assert_eq!(code1, code2, "static code differs");

    let nt1 = NodeTypesGenerator::new(&g1).generate().unwrap();
    let nt2 = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_eq!(nt1, nt2, "node types differ");

    let abi1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let abi2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_eq!(abi1, abi2, "ABI output differs");
}

#[test]
fn pipeline_consistency_two_alt_all_generators() {
    let (g1, pt1) = build_pipeline(two_alt_grammar());
    let (g2, pt2) = build_pipeline(two_alt_grammar());

    let code1 = StaticLanguageGenerator::new(g1.clone(), pt1.clone())
        .generate_language_code()
        .to_string();
    let code2 = StaticLanguageGenerator::new(g2.clone(), pt2.clone())
        .generate_language_code()
        .to_string();
    assert_eq!(code1, code2);

    let nt1 = NodeTypesGenerator::new(&g1).generate().unwrap();
    let nt2 = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_eq!(nt1, nt2);

    let abi1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let abi2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_eq!(abi1, abi2);
}

#[test]
fn pipeline_consistency_chain_all_generators() {
    let (g1, pt1) = build_pipeline(chain_grammar());
    let (g2, pt2) = build_pipeline(chain_grammar());

    let code1 = StaticLanguageGenerator::new(g1.clone(), pt1.clone())
        .generate_language_code()
        .to_string();
    let code2 = StaticLanguageGenerator::new(g2.clone(), pt2.clone())
        .generate_language_code()
        .to_string();
    assert_eq!(code1, code2);

    let abi1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let abi2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_eq!(abi1, abi2);
}

#[test]
fn pipeline_consistency_recursive_all_generators() {
    let (g1, pt1) = build_pipeline(recursive_grammar());
    let (g2, pt2) = build_pipeline(recursive_grammar());

    let code1 = StaticLanguageGenerator::new(g1.clone(), pt1.clone())
        .generate_language_code()
        .to_string();
    let code2 = StaticLanguageGenerator::new(g2.clone(), pt2.clone())
        .generate_language_code()
        .to_string();
    assert_eq!(code1, code2);

    let abi1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
    let abi2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
    assert_eq!(abi1, abi2);
}

#[test]
fn pipeline_consistency_nested_all_generators() {
    let (g1, pt1) = build_pipeline(nested_grammar());
    let (g2, pt2) = build_pipeline(nested_grammar());

    let code1 = StaticLanguageGenerator::new(g1.clone(), pt1.clone())
        .generate_language_code()
        .to_string();
    let code2 = StaticLanguageGenerator::new(g2.clone(), pt2.clone())
        .generate_language_code()
        .to_string();
    assert_eq!(code1, code2);

    let nt1 = NodeTypesGenerator::new(&g1).generate().unwrap();
    let nt2 = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_eq!(nt1, nt2);
}

#[test]
fn pipeline_consistency_multi_rule_ten_runs() {
    let results: Vec<(String, String, String)> = (0..10)
        .map(|_| {
            let (g, pt) = build_pipeline(multi_rule_grammar());
            let code = StaticLanguageGenerator::new(g.clone(), pt.clone())
                .generate_language_code()
                .to_string();
            let nt = NodeTypesGenerator::new(&g).generate().unwrap();
            let abi = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
            (code, nt, abi)
        })
        .collect();
    for i in 1..results.len() {
        assert_eq!(results[0].0, results[i].0, "static code run 0 vs {i}");
        assert_eq!(results[0].1, results[i].1, "node types run 0 vs {i}");
        assert_eq!(results[0].2, results[i].2, "ABI run 0 vs {i}");
    }
}

#[test]
fn pipeline_consistency_deep_chain_ten_runs() {
    let results: Vec<(String, String, String)> = (0..10)
        .map(|_| {
            let (g, pt) = build_pipeline(deep_chain_grammar());
            let code = StaticLanguageGenerator::new(g.clone(), pt.clone())
                .generate_language_code()
                .to_string();
            let nt = NodeTypesGenerator::new(&g).generate().unwrap();
            let abi = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
            (code, nt, abi)
        })
        .collect();
    for i in 1..results.len() {
        assert_eq!(results[0].0, results[i].0, "static code run 0 vs {i}");
        assert_eq!(results[0].1, results[i].1, "node types run 0 vs {i}");
        assert_eq!(results[0].2, results[i].2, "ABI run 0 vs {i}");
    }
}

#[test]
fn pipeline_consistency_three_alt_ten_runs() {
    let results: Vec<(String, String, String)> = (0..10)
        .map(|_| {
            let (g, pt) = build_pipeline(three_alt_grammar());
            let code = StaticLanguageGenerator::new(g.clone(), pt.clone())
                .generate_language_code()
                .to_string();
            let nt = NodeTypesGenerator::new(&g).generate().unwrap();
            let abi = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
            (code, nt, abi)
        })
        .collect();
    for i in 1..results.len() {
        assert_eq!(results[0].0, results[i].0, "static code run 0 vs {i}");
        assert_eq!(results[0].1, results[i].1, "node types run 0 vs {i}");
        assert_eq!(results[0].2, results[i].2, "ABI run 0 vs {i}");
    }
}

// =====================================================================
// 6. Output property verification (8 tests)
// =====================================================================

#[test]
fn property_static_code_nonempty_simple() {
    let (g, pt) = build_pipeline(simple_grammar());
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty(), "generated code must not be empty");
}

#[test]
fn property_static_code_nonempty_multi_rule() {
    let (g, pt) = build_pipeline(multi_rule_grammar());
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty(), "generated code must not be empty");
}

#[test]
fn property_node_types_valid_json_simple() {
    let g = simple_grammar();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&output).expect("node types output must be valid JSON");
    assert!(parsed.is_array(), "node types must be a JSON array");
}

#[test]
fn property_node_types_valid_json_multi_rule() {
    let g = multi_rule_grammar();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&output).expect("node types output must be valid JSON");
    assert!(parsed.is_array(), "node types must be a JSON array");
}

#[test]
fn property_node_types_nonempty_recursive() {
    let g = recursive_grammar();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    assert!(!output.is_empty(), "node types must not be empty");
}

#[test]
fn property_abi_output_nonempty_simple() {
    let (g, pt) = build_pipeline(simple_grammar());
    let abi = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!abi.is_empty(), "ABI output must not be empty");
}

#[test]
fn property_abi_output_nonempty_nested() {
    let (g, pt) = build_pipeline(nested_grammar());
    let abi = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!abi.is_empty(), "ABI output must not be empty");
}

#[test]
fn property_node_types_json_array_has_entries_two_alt() {
    let g = two_alt_grammar();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&output).expect("node types output must be valid JSON");
    let arr = parsed.as_array().expect("must be an array");
    assert!(!arr.is_empty(), "node types array must have entries");
}
