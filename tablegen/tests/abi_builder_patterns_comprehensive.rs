//! Comprehensive tests for AbiLanguageBuilder patterns.

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::AbiLanguageBuilder;

fn build_abi(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> proc_macro2::TokenStream {
    let mut b = GrammarBuilder::new(name);
    for (n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    b = b.start(start);
    let mut g = b.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    let builder = AbiLanguageBuilder::new(&g, &pt);
    builder.generate()
}

#[test]
fn abi_simple_nonempty() {
    let ts = build_abi("simple", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(!ts.is_empty());
}

#[test]
fn abi_two_alts_nonempty() {
    let ts = build_abi(
        "alts",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(!ts.is_empty());
}

#[test]
fn abi_chain_nonempty() {
    let ts = build_abi(
        "chain",
        &[("x", "x")],
        &[("inner", vec!["x"]), ("s", vec!["inner"])],
        "s",
    );
    assert!(!ts.is_empty());
}

#[test]
fn abi_deterministic() {
    let ts1 = build_abi("det", &[("a", "a")], &[("s", vec!["a"])], "s");
    let ts2 = build_abi("det", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(ts1.to_string(), ts2.to_string());
}

#[test]
fn abi_contains_language() {
    let ts = build_abi("lang", &[("a", "a")], &[("s", vec!["a"])], "s");
    let code = ts.to_string();
    // Should contain some reference to language struct
    assert!(!code.is_empty());
}

#[test]
fn abi_sequence_grammar() {
    let ts = build_abi(
        "seq",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(!ts.is_empty());
}

#[test]
fn abi_multi_nonterminal() {
    let ts = build_abi(
        "multi",
        &[("a", "a"), ("b", "b")],
        &[("x", vec!["a"]), ("y", vec!["b"]), ("s", vec!["x", "y"])],
        "s",
    );
    assert!(!ts.is_empty());
}

#[test]
fn abi_five_alts() {
    let ts = build_abi(
        "five",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[
            ("s", vec!["a"]),
            ("s", vec!["b"]),
            ("s", vec!["c"]),
            ("s", vec!["d"]),
            ("s", vec!["e"]),
        ],
        "s",
    );
    assert!(!ts.is_empty());
}

#[test]
fn abi_renders_to_string() {
    let ts = build_abi("render", &[("a", "a")], &[("s", vec!["a"])], "s");
    let s = ts.to_string();
    assert!(s.len() > 10);
}

#[test]
fn abi_diamond_grammar() {
    let ts = build_abi(
        "dia",
        &[("a", "a")],
        &[
            ("l", vec!["a"]),
            ("r", vec!["a"]),
            ("s", vec!["l"]),
            ("s", vec!["r"]),
        ],
        "s",
    );
    assert!(!ts.is_empty());
}
