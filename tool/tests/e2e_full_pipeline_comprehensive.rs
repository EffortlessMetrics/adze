// Comprehensive tests for the full pipeline: Grammar -> FIRST/FOLLOW -> ParseTable -> Codegen
// End-to-end integration tests

use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, build_parser};

#[test]
fn e2e_minimal_grammar() {
    let g = GrammarBuilder::new("min")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let r = build_parser(g, BuildOptions::default()).unwrap();
    assert!(!r.parser_code.is_empty());
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn e2e_two_token_sequence() {
    let g = GrammarBuilder::new("seq")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x", "y"])
        .start("s")
        .build();
    assert!(build_parser(g, BuildOptions::default()).is_ok());
}

#[test]
fn e2e_alternatives() {
    let g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    assert!(build_parser(g, BuildOptions::default()).is_ok());
}

#[test]
fn e2e_left_recursion() {
    let g = GrammarBuilder::new("lr")
        .token("n", "[0-9]+")
        .token("plus", r"\+")
        .rule("e", vec!["e", "plus", "n"])
        .rule("e", vec!["n"])
        .start("e")
        .build();
    assert!(build_parser(g, BuildOptions::default()).is_ok());
}

#[test]
fn e2e_nonterminal_chain() {
    let g = GrammarBuilder::new("ch")
        .token("a", "a")
        .rule("s", vec!["m"])
        .rule("m", vec!["n"])
        .rule("n", vec!["a"])
        .start("s")
        .build();
    assert!(build_parser(g, BuildOptions::default()).is_ok());
}

#[test]
fn e2e_three_alternatives() {
    let g = GrammarBuilder::new("tri")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    assert!(build_parser(g, BuildOptions::default()).is_ok());
}

#[test]
fn e2e_regex_token() {
    let g = GrammarBuilder::new("re")
        .token("ident", "[a-z]+")
        .rule("s", vec!["ident"])
        .start("s")
        .build();
    assert!(build_parser(g, BuildOptions::default()).is_ok());
}

#[test]
fn e2e_right_recursion() {
    let g = GrammarBuilder::new("rr")
        .token("a", "a")
        .rule("s", vec!["a", "s"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(build_parser(g, BuildOptions::default()).is_ok());
}

#[test]
fn e2e_long_rhs() {
    let g = GrammarBuilder::new("long")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c", "a", "b", "c"])
        .start("s")
        .build();
    assert!(build_parser(g, BuildOptions::default()).is_ok());
}

#[test]
fn e2e_multiple_nonterminals() {
    let g = GrammarBuilder::new("mn")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["x", "y"])
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .start("s")
        .build();
    assert!(build_parser(g, BuildOptions::default()).is_ok());
}

#[test]
fn e2e_determinism() {
    let make = || {
        GrammarBuilder::new("det")
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build()
    };
    let r1 = build_parser(make(), BuildOptions::default()).unwrap();
    let r2 = build_parser(make(), BuildOptions::default()).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn e2e_node_types_valid_json() {
    let g = GrammarBuilder::new("json")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let r = build_parser(g, BuildOptions::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array());
}
