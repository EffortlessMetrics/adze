//! Comprehensive tests for adze-tool pure_rust_builder module v2.

use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, build_parser};

fn simple_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("simple")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build()
}

fn arith_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("arith")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule("e", vec!["num"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, adze_ir::Associativity::Left)
        .rule_with_precedence("e", vec!["e", "star", "e"], 2, adze_ir::Associativity::Left)
        .start("e")
        .build()
}

// ── BuildOptions ──

#[test]
fn v2_build_options_default() {
    let opts = BuildOptions::default();
    let _ = format!("{:?}", opts);
}

#[test]
fn v2_build_options_clone() {
    let opts = BuildOptions::default();
    let c = opts.clone();
    let _ = format!("{:?}", c);
}

// ── Simple grammar build ──

#[test]
fn v2_build_simple() {
    let r = build_parser(simple_grammar(), BuildOptions::default());
    assert!(r.is_ok());
}

#[test]
fn v2_build_simple_name() {
    let r = build_parser(simple_grammar(), BuildOptions::default()).unwrap();
    assert_eq!(r.grammar_name, "simple");
}

#[test]
fn v2_build_simple_code() {
    let r = build_parser(simple_grammar(), BuildOptions::default()).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn v2_build_simple_nodes() {
    let r = build_parser(simple_grammar(), BuildOptions::default()).unwrap();
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn v2_build_simple_json() {
    let r = build_parser(simple_grammar(), BuildOptions::default()).unwrap();
    let _: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
}

// ── Arith grammar build ──

#[test]
fn v2_build_arith() {
    let r = build_parser(arith_grammar(), BuildOptions::default());
    assert!(r.is_ok());
}

#[test]
fn v2_build_arith_name() {
    let r = build_parser(arith_grammar(), BuildOptions::default()).unwrap();
    assert_eq!(r.grammar_name, "arith");
}

#[test]
fn v2_build_arith_code() {
    let r = build_parser(arith_grammar(), BuildOptions::default()).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn v2_build_arith_json() {
    let r = build_parser(arith_grammar(), BuildOptions::default()).unwrap();
    let _: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
}

// ── Multi-token grammar ──

#[test]
fn v2_build_multi_token() {
    let g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let r = build_parser(g, BuildOptions::default());
    assert!(r.is_ok());
}

// ── Alternative grammar ──

#[test]
fn v2_build_alternatives() {
    let g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let r = build_parser(g, BuildOptions::default());
    assert!(r.is_ok());
}

// ── Chain grammar ──

#[test]
fn v2_build_chain() {
    let g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let r = build_parser(g, BuildOptions::default());
    assert!(r.is_ok());
}

// ── Determinism ──

#[test]
fn v2_deterministic_name() {
    let r1 = build_parser(simple_grammar(), BuildOptions::default()).unwrap();
    let r2 = build_parser(simple_grammar(), BuildOptions::default()).unwrap();
    assert_eq!(r1.grammar_name, r2.grammar_name);
}

#[test]
fn v2_deterministic_nodes() {
    let r1 = build_parser(simple_grammar(), BuildOptions::default()).unwrap();
    let r2 = build_parser(simple_grammar(), BuildOptions::default()).unwrap();
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

// ── Scale ──

#[test]
fn v2_scale_15_tokens() {
    let mut b = GrammarBuilder::new("s15");
    for i in 0..15 {
        let n: &str = Box::leak(format!("t{}", i).into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let g = b.start("s").build();
    assert!(build_parser(g, BuildOptions::default()).is_ok());
}

// ── BuildResult stats ──

#[test]
fn v2_stats_exist() {
    let r = build_parser(simple_grammar(), BuildOptions::default()).unwrap();
    let _ = &r.build_stats;
}
