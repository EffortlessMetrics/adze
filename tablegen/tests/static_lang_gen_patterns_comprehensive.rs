//! Comprehensive tests for StaticLanguageGenerator patterns.

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::StaticLanguageGenerator;

fn build_slg(
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
    let slg = StaticLanguageGenerator::new(g, pt);
    slg.generate_language_code()
}

#[test]
fn slg_simple_nonempty() {
    let ts = build_slg("simple", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(!ts.is_empty());
}

#[test]
fn slg_two_alts() {
    let ts = build_slg(
        "alts",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(!ts.is_empty());
}

#[test]
fn slg_chain() {
    let ts = build_slg(
        "chain",
        &[("x", "x")],
        &[("inner", vec!["x"]), ("s", vec!["inner"])],
        "s",
    );
    assert!(!ts.is_empty());
}

#[test]
fn slg_deterministic() {
    let ts1 = build_slg("det", &[("a", "a")], &[("s", vec!["a"])], "s");
    let ts2 = build_slg("det", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(ts1.to_string(), ts2.to_string());
}

#[test]
fn slg_sequence() {
    let ts = build_slg(
        "seq",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(!ts.is_empty());
}

#[test]
fn slg_multi_nt() {
    let ts = build_slg(
        "multi",
        &[("a", "a"), ("b", "b")],
        &[("x", vec!["a"]), ("y", vec!["b"]), ("s", vec!["x", "y"])],
        "s",
    );
    assert!(!ts.is_empty());
}

#[test]
fn slg_five_alts() {
    let ts = build_slg(
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
fn slg_diamond() {
    let ts = build_slg(
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

#[test]
fn slg_renders_long() {
    let ts = build_slg("render", &[("a", "a")], &[("s", vec!["a"])], "s");
    let s = ts.to_string();
    assert!(s.len() > 10);
}

#[test]
fn slg_three_token_seq() {
    let ts = build_slg(
        "three",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    assert!(!ts.is_empty());
}
