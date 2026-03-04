//! Comprehensive tests for GrammarBuilder with FirstFollowSets pipeline.

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;

fn build_table(
    name: &str,
    tokens: &[(&str, &str)],
    rules: Vec<(&str, Vec<&str>)>,
    start: &str,
) -> adze_glr_core::ParseTable {
    let mut b = GrammarBuilder::new(name);
    for (n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs);
    }
    let g = b.start(start).build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    build_lr1_automaton(&g, &ff).unwrap()
}

// ── End-to-end grammar to parse table ──

#[test]
fn e2e_single_token() {
    let t = build_table("e2e", &[("x", "x")], vec![("s", vec!["x"])], "s");
    assert!(t.state_count > 0);
    assert!(!t.rules.is_empty());
}

#[test]
fn e2e_two_tokens() {
    let t = build_table(
        "e2e2",
        &[("a", "a"), ("b", "b")],
        vec![("s", vec!["a", "b"])],
        "s",
    );
    assert!(t.state_count >= 3);
}

#[test]
fn e2e_three_alternatives() {
    let t = build_table(
        "e2e3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        vec![("s", vec!["a"]), ("s", vec!["b"]), ("s", vec!["c"])],
        "s",
    );
    assert!(t.rules.len() >= 3);
}

#[test]
fn e2e_chain_three_deep() {
    let t = build_table(
        "chain3",
        &[("x", "x")],
        vec![("a", vec!["x"]), ("b", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(t.rules.len() >= 3);
}

#[test]
fn e2e_five_token_sequence() {
    let t = build_table(
        "seq5",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        vec![("s", vec!["a", "b", "c", "d", "e"])],
        "s",
    );
    assert!(t.state_count >= 6);
}

// ── Precedence pipeline ──

#[test]
fn e2e_left_assoc() {
    let g = GrammarBuilder::new("la")
        .token("n", "n")
        .token("plus", "\\+")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .start("e")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let t = build_lr1_automaton(&g, &ff).unwrap();
    assert!(t.rules.len() >= 2);
}

#[test]
fn e2e_multi_prec() {
    let g = GrammarBuilder::new("mp")
        .token("n", "n")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "star", "e"], 2, Associativity::Left)
        .start("e")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let t = build_lr1_automaton(&g, &ff).unwrap();
    assert!(t.rules.len() >= 3);
}

// ── Normalized grammar pipeline ──

#[test]
fn e2e_normalized() {
    let mut g = GrammarBuilder::new("norm")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let t = build_lr1_automaton(&g, &ff).unwrap();
    assert!(t.state_count > 0);
}

// ── Scale ──

#[test]
fn e2e_scale_25_tokens() {
    let mut b = GrammarBuilder::new("s25");
    for i in 0..25 {
        let n: &str = Box::leak(format!("t{}", i).into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let g = b.start("s").build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let t = build_lr1_automaton(&g, &ff).unwrap();
    assert!(t.rules.len() >= 25);
}

// ── Full pipeline with tablegen ──

#[test]
fn e2e_full_pipeline_static_lang() {
    use adze_tablegen::StaticLanguageGenerator;
    let g = GrammarBuilder::new("full")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let t = build_lr1_automaton(&g, &ff).unwrap();
    let generator = StaticLanguageGenerator::new(g, t);
    let code = generator.generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn e2e_full_pipeline_abi() {
    use adze_tablegen::abi_builder::AbiLanguageBuilder;
    let g = GrammarBuilder::new("abi")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let t = build_lr1_automaton(&g, &ff).unwrap();
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code = builder.generate();
    assert!(!code.is_empty());
}

#[test]
fn e2e_full_pipeline_node_types() {
    use adze_tablegen::node_types::NodeTypesGenerator;
    let g = GrammarBuilder::new("nodes")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let _: serde_json::Value = serde_json::from_str(&json).unwrap();
}
