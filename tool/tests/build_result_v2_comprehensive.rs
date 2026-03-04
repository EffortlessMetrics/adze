// Comprehensive tests for BuildResult struct and build_parser outcomes
// Tests the complete build pipeline output

use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, build_parser};

fn basic_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("basic")
        .token("num", "[0-9]+")
        .rule("s", vec!["num"])
        .start("s")
        .build()
}

#[test]
fn build_result_grammar_name_matches() {
    let r = build_parser(basic_grammar(), BuildOptions::default()).unwrap();
    assert_eq!(r.grammar_name, "basic");
}

#[test]
fn build_result_parser_code_non_empty() {
    let r = build_parser(basic_grammar(), BuildOptions::default()).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn build_result_node_types_json_array() {
    let r = build_parser(basic_grammar(), BuildOptions::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array());
}

#[test]
fn build_result_parser_path_non_empty() {
    let r = build_parser(basic_grammar(), BuildOptions::default()).unwrap();
    assert!(!r.parser_path.is_empty());
}

#[test]
fn build_result_stats_debug() {
    let r = build_parser(basic_grammar(), BuildOptions::default()).unwrap();
    let dbg = format!("{:?}", r.build_stats);
    assert!(!dbg.is_empty());
}

#[test]
fn build_with_multiple_tokens() {
    let g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let r = build_parser(g, BuildOptions::default()).unwrap();
    assert_eq!(r.grammar_name, "multi");
}

#[test]
fn build_with_alternatives_succeeds() {
    let g = GrammarBuilder::new("alt")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .start("s")
        .build();
    assert!(build_parser(g, BuildOptions::default()).is_ok());
}

#[test]
fn build_with_nonterminal_chain() {
    let g = GrammarBuilder::new("ch")
        .token("a", "a")
        .rule("s", vec!["mid"])
        .rule("mid", vec!["a"])
        .start("s")
        .build();
    let r = build_parser(g, BuildOptions::default()).unwrap();
    assert!(r.parser_code.len() > 10);
}

#[test]
fn build_left_recursive() {
    let g = GrammarBuilder::new("lr")
        .token("plus", r"\+")
        .token("n", "[0-9]+")
        .rule("expr", vec!["expr", "plus", "n"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    assert!(build_parser(g, BuildOptions::default()).is_ok());
}

#[test]
fn build_result_deterministic() {
    let g1 = basic_grammar();
    let g2 = basic_grammar();
    let r1 = build_parser(g1, BuildOptions::default()).unwrap();
    let r2 = build_parser(g2, BuildOptions::default()).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn build_parser_code_contains_language_data() {
    let r = build_parser(basic_grammar(), BuildOptions::default()).unwrap();
    // Parser code should contain generated data
    assert!(r.parser_code.len() > 50);
}
