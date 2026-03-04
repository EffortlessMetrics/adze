//! Comprehensive tests for build_parser pipeline and BuildOptions/BuildResult.

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, build_parser};

fn test_opts() -> BuildOptions {
    BuildOptions {
        out_dir: "/tmp/build_test".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    }
}

fn simple_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

#[test]
fn build_simple() {
    let result = build_parser(simple_grammar(), test_opts()).unwrap();
    assert_eq!(result.grammar_name, "simple");
}

#[test]
fn build_has_code() {
    let result = build_parser(simple_grammar(), test_opts()).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_has_node_types() {
    let result = build_parser(simple_grammar(), test_opts()).unwrap();
    assert!(!result.node_types_json.is_empty());
}

#[test]
fn build_stats_states() {
    let result = build_parser(simple_grammar(), test_opts()).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn build_stats_symbols() {
    let result = build_parser(simple_grammar(), test_opts()).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn build_two_alts() {
    let g = GrammarBuilder::new("alts")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let result = build_parser(g, test_opts()).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn build_chain() {
    let g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("s", vec!["inner"])
        .start("s")
        .build();
    let result = build_parser(g, test_opts()).unwrap();
    assert!(result.build_stats.symbol_count > 0);
}

#[test]
fn build_with_prec() {
    let g = GrammarBuilder::new("prec")
        .token("a", "a")
        .token("b", "b")
        .rule_with_precedence("s", vec!["a"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["b"], 2, Associativity::Right)
        .start("s")
        .build();
    let result = build_parser(g, test_opts()).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn build_name_preserved() {
    let g = GrammarBuilder::new("myparser")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let result = build_parser(g, test_opts()).unwrap();
    assert_eq!(result.grammar_name, "myparser");
}

#[test]
fn build_node_types_valid_json() {
    let result = build_parser(simple_grammar(), test_opts()).unwrap();
    let _: serde_json::Value = serde_json::from_str(&result.node_types_json).unwrap();
}

#[test]
fn build_deterministic() {
    let r1 = build_parser(simple_grammar(), test_opts()).unwrap();
    let r2 = build_parser(simple_grammar(), test_opts()).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn build_no_compress() {
    let opts = BuildOptions {
        out_dir: "/tmp/nocomp".to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let result = build_parser(simple_grammar(), opts).unwrap();
    assert!(!result.parser_code.is_empty());
}

#[test]
fn build_larger_grammar() {
    let mut b = GrammarBuilder::new("big");
    for i in 0..10 {
        let n = format!("t{}", i);
        b = b.token(&n, &n);
    }
    for i in 0..10 {
        let tok = format!("t{}", i);
        b = b.rule("s", vec![&tok]);
    }
    b = b.start("s");
    let g = b.build();
    let result = build_parser(g, test_opts()).unwrap();
    assert!(result.build_stats.state_count > 0);
}

#[test]
fn build_multi_rhs() {
    let g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let result = build_parser(g, test_opts()).unwrap();
    assert!(result.build_stats.state_count > 0);
}
