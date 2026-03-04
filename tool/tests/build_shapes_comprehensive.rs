//! Tests for build_parser with various grammar shapes and options.

use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, build_parser};

fn opts() -> BuildOptions {
    BuildOptions {
        out_dir: std::env::temp_dir().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: true,
    }
}

#[test]
fn build_single_token_grammar() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let r = build_parser(g, opts()).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn build_sequence_grammar() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    assert!(build_parser(g, opts()).is_ok());
}

#[test]
fn build_alternative_grammar() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    assert!(build_parser(g, opts()).is_ok());
}

#[test]
fn build_chain_grammar() {
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("c", vec!["x"])
        .rule("b", vec!["c"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    assert!(build_parser(g, opts()).is_ok());
}

#[test]
fn build_recursive_grammar() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("op", "+")
        .rule("expr", vec!["expr", "op", "a"])
        .rule("expr", vec!["a"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    assert!(build_parser(g, opts()).is_ok());
}

#[test]
fn build_deterministic() {
    let mk = || {
        let g = GrammarBuilder::new("t")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        build_parser(g, opts()).unwrap()
    };
    assert_eq!(mk().parser_code, mk().parser_code);
}

#[test]
fn build_node_types_valid_json() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let r = build_parser(g, opts()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array());
}

#[test]
fn build_grammar_name_preserved() {
    let g = GrammarBuilder::new("my_lang")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let r = build_parser(g, opts()).unwrap();
    assert_eq!(r.grammar_name, "my_lang");
}

#[test]
fn build_stats_positive() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let r = build_parser(g, opts()).unwrap();
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn build_no_compress() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let opts = BuildOptions {
        out_dir: std::env::temp_dir().to_string_lossy().to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };
    assert!(build_parser(g, opts).is_ok());
}
