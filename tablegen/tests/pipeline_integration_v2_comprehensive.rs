//! Tests verifying IR → GLR → Tablegen full pipeline integration v2.

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::{NodeTypesGenerator, StaticLanguageGenerator};

fn build_pipeline(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> (adze_ir::Grammar, adze_glr_core::ParseTable) {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    let grammar = b.start(start).build();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    (grammar, table)
}

#[test]
fn pipeline_single_produces_code() {
    let (g, t) = build_pipeline("p1", &[("x", "x")], &[("S", vec!["x"])], "S");
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn pipeline_binary_op_produces_code() {
    let (g, t) = build_pipeline(
        "binop",
        &[("n", r"\d+"), ("plus", r"\+")],
        &[("E", vec!["n"]), ("E", vec!["E", "plus", "n"])],
        "E",
    );
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn pipeline_node_types_valid_json() {
    let (g, _t) = build_pipeline("ntj2", &[("a", "a")], &[("S", vec!["a"])], "S");
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(val.is_array());
}

#[test]
fn pipeline_node_types_has_entries() {
    let (g, _t) = build_pipeline(
        "entries",
        &[("a", "a"), ("b", "b")],
        &[("S", vec!["A"]), ("A", vec!["a", "b"])],
        "S",
    );
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(!val.as_array().unwrap().is_empty());
}

#[test]
fn pipeline_determinism_code() {
    let args = ("det2", &[("x", "x")][..], &[("S", vec!["x"])][..], "S");
    let (g1, t1) = build_pipeline(args.0, args.1, args.2, args.3);
    let (g2, t2) = build_pipeline(args.0, args.1, args.2, args.3);
    let code1 = StaticLanguageGenerator::new(g1, t1).generate_language_code();
    let code2 = StaticLanguageGenerator::new(g2, t2).generate_language_code();
    assert_eq!(code1.to_string(), code2.to_string());
}

#[test]
fn pipeline_determinism_node_types() {
    let (g1, _) = build_pipeline("detnt", &[("x", "x")], &[("S", vec!["x"])], "S");
    let (g2, _) = build_pipeline("detnt", &[("x", "x")], &[("S", vec!["x"])], "S");
    assert_eq!(
        NodeTypesGenerator::new(&g1).generate().unwrap(),
        NodeTypesGenerator::new(&g2).generate().unwrap()
    );
}

#[test]
fn pipeline_state_count_positive() {
    let (_, t) = build_pipeline("sc", &[("a", "a")], &[("S", vec!["a"])], "S");
    assert!(t.state_count > 0);
}

#[test]
fn pipeline_eof_symbol_accessible() {
    let (_, t) = build_pipeline("eof2", &[("a", "a")], &[("S", vec!["a"])], "S");
    let _ = t.eof_symbol;
}

#[test]
fn pipeline_three_levels() {
    let (g, t) = build_pipeline(
        "lev3",
        &[("x", "x")],
        &[("S", vec!["A"]), ("A", vec!["B"]), ("B", vec!["x"])],
        "S",
    );
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn pipeline_epsilon() {
    let (g, t) = build_pipeline(
        "eps2",
        &[("x", "x")],
        &[("S", vec!["x"]), ("S", vec![])],
        "S",
    );
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn pipeline_multiple_tokens() {
    let (g, t) = build_pipeline(
        "multi2",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d")],
        &[("S", vec!["a", "b", "c", "d"])],
        "S",
    );
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn pipeline_five_alternatives() {
    let (g, t) = build_pipeline(
        "alt5",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[
            ("S", vec!["a"]),
            ("S", vec!["b"]),
            ("S", vec!["c"]),
            ("S", vec!["d"]),
            ("S", vec!["e"]),
        ],
        "S",
    );
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn pipeline_generate_node_types_method() {
    let (g, t) = build_pipeline("ntm", &[("a", "a")], &[("S", vec!["a"])], "S");
    let generator = StaticLanguageGenerator::new(g, t);
    let nt = generator.generate_node_types();
    assert!(!nt.is_empty());
}
