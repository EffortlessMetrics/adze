// Comprehensive tests for end-to-end grammar build consistency
// Tests that the full pipeline produces consistent results

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, build_parser};

fn arithmetic_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("arith")
        .token("num", "[0-9]+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build()
}

#[test]
fn arithmetic_grammar_builds() {
    let g = arithmetic_grammar();
    assert!(build_parser(g, BuildOptions::default()).is_ok());
}

#[test]
fn arithmetic_grammar_has_tokens() {
    let g = arithmetic_grammar();
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn arithmetic_grammar_has_rules() {
    let g = arithmetic_grammar();
    assert_eq!(g.all_rules().count(), 3);
}

#[test]
fn arithmetic_parser_code_non_empty() {
    let r = build_parser(arithmetic_grammar(), BuildOptions::default()).unwrap();
    assert!(r.parser_code.len() > 100);
}

#[test]
fn arithmetic_node_types_valid() {
    let r = build_parser(arithmetic_grammar(), BuildOptions::default()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array());
}

#[test]
fn arithmetic_deterministic() {
    let r1 = build_parser(arithmetic_grammar(), BuildOptions::default()).unwrap();
    let r2 = build_parser(arithmetic_grammar(), BuildOptions::default()).unwrap();
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn simple_list_grammar_builds() {
    let g = GrammarBuilder::new("list")
        .token("item", "[a-z]+")
        .token("comma", ",")
        .rule("list", vec!["item"])
        .rule("list", vec!["list", "comma", "item"])
        .start("list")
        .build();
    assert!(build_parser(g, BuildOptions::default()).is_ok());
}

#[test]
fn nested_grammar_builds() {
    let g = GrammarBuilder::new("nest")
        .token("lp", r"\(")
        .token("rp", r"\)")
        .token("x", "x")
        .rule("s", vec!["x"])
        .rule("s", vec!["lp", "s", "rp"])
        .start("s")
        .build();
    assert!(build_parser(g, BuildOptions::default()).is_ok());
}

#[test]
fn statement_grammar_builds() {
    let g = GrammarBuilder::new("stmt")
        .token("ident", "[a-z]+")
        .token("eq", "=")
        .token("num", "[0-9]+")
        .token("semi", ";")
        .rule("stmt", vec!["ident", "eq", "num", "semi"])
        .rule("prog", vec!["stmt"])
        .rule("prog", vec!["prog", "stmt"])
        .start("prog")
        .build();
    assert!(build_parser(g, BuildOptions::default()).is_ok());
}

#[test]
fn grammar_name_in_result() {
    let r = build_parser(arithmetic_grammar(), BuildOptions::default()).unwrap();
    assert_eq!(r.grammar_name, "arith");
}
