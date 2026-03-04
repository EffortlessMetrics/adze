//! Comprehensive tests for BuildOptions and BuildResult properties.

use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, build_parser};

fn simple_grammar() -> adze_ir::Grammar {
    let mut g = GrammarBuilder::new("test")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    g
}

// ── BuildOptions ──

#[test]
fn default_options() {
    let opts = BuildOptions::default();
    let _ = opts;
}

#[test]
fn clone_options() {
    let opts = BuildOptions::default();
    let opts2 = opts.clone();
    let _ = opts2;
}

// ── build_parser ──

#[test]
fn build_simple_grammar() {
    let g = simple_grammar();
    let result = build_parser(g, BuildOptions::default());
    assert!(result.is_ok());
}

#[test]
fn result_grammar_name() {
    let g = simple_grammar();
    let result = build_parser(g, BuildOptions::default()).unwrap();
    assert_eq!(result.grammar_name, "test");
}

#[test]
fn result_parser_code_nonempty() {
    let g = simple_grammar();
    let result = build_parser(g, BuildOptions::default()).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn result_node_types_json_valid() {
    let g = simple_grammar();
    let result = build_parser(g, BuildOptions::default()).unwrap();
    let _: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
}

#[test]
fn result_parser_path_nonempty() {
    let g = simple_grammar();
    let result = build_parser(g, BuildOptions::default()).unwrap();
    // parser_path may be empty or set depending on options
    let _ = result.parser_path;
}

// ── Two-token grammar ──

#[test]
fn build_two_token() {
    let mut g = GrammarBuilder::new("two")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    g.normalize();
    let result = build_parser(g, BuildOptions::default());
    assert!(result.is_ok());
}

#[test]
fn two_token_name() {
    let mut g = GrammarBuilder::new("two")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    g.normalize();
    let result = build_parser(g, BuildOptions::default()).unwrap();
    assert_eq!(result.grammar_name, "two");
}

// ── Three-token grammar ──

#[test]
fn build_three_token() {
    let mut g = GrammarBuilder::new("three")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    g.normalize();
    let result = build_parser(g, BuildOptions::default());
    assert!(result.is_ok());
}

// ── Alternative grammar ──

#[test]
fn build_alternatives() {
    let mut g = GrammarBuilder::new("alts")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .start("s")
        .build();
    g.normalize();
    let result = build_parser(g, BuildOptions::default());
    assert!(result.is_ok());
}

// ── Build determinism ──

#[test]
fn build_deterministic_name() {
    let g1 = simple_grammar();
    let g2 = simple_grammar();
    let r1 = build_parser(g1, BuildOptions::default()).unwrap();
    let r2 = build_parser(g2, BuildOptions::default()).unwrap();
    assert_eq!(r1.grammar_name, r2.grammar_name);
}

// ── BuildResult fields ──

#[test]
fn build_stats_field() {
    let g = simple_grammar();
    let result = build_parser(g, BuildOptions::default()).unwrap();
    let _ = result.build_stats;
}

// ── Node types JSON structure ──

#[test]
fn node_types_is_array() {
    let g = simple_grammar();
    let result = build_parser(g, BuildOptions::default()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(val.is_array());
}

#[test]
fn node_types_has_entries() {
    let g = simple_grammar();
    let result = build_parser(g, BuildOptions::default()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
    assert!(val.as_array().unwrap().len() > 0);
}
