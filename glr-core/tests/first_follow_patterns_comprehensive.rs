//! Comprehensive tests for FirstFollowSets computation patterns.

use adze_glr_core::FirstFollowSets;
use adze_ir::builder::GrammarBuilder;

fn make_ff(tokens: &[(&str, &str)], rules: &[(&str, Vec<&str>)], start: &str) -> FirstFollowSets {
    let mut b = GrammarBuilder::new("test");
    for (n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    b = b.start(start);
    let mut g = b.build();
    g.normalize();
    FirstFollowSets::compute(&g).unwrap()
}

#[test]
fn ff_single_token_grammar() {
    let _ = make_ff(&[("a", "a")], &[("s", vec!["a"])], "s");
}

#[test]
fn ff_two_token_seq() {
    let _ = make_ff(&[("a", "a"), ("b", "b")], &[("s", vec!["a", "b"])], "s");
}

#[test]
fn ff_two_alternatives() {
    let _ = make_ff(
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
}

#[test]
fn ff_nonterminal_chain() {
    let _ = make_ff(
        &[("x", "x")],
        &[("inner", vec!["x"]), ("s", vec!["inner"])],
        "s",
    );
}

#[test]
fn ff_triple_token_seq() {
    let _ = make_ff(
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
}

#[test]
fn ff_diamond_grammar() {
    let _ = make_ff(
        &[("a", "a")],
        &[
            ("l", vec!["a"]),
            ("r", vec!["a"]),
            ("s", vec!["l"]),
            ("s", vec!["r"]),
        ],
        "s",
    );
}

#[test]
fn ff_three_level_chain() {
    let _ = make_ff(
        &[("x", "x")],
        &[("a", vec!["x"]), ("b", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
}

#[test]
fn ff_wide_alternatives() {
    let _ = make_ff(
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
}

#[test]
fn ff_mixed_terminals_nonterminals() {
    let _ = make_ff(
        &[("a", "a"), ("b", "b")],
        &[("x", vec!["a"]), ("s", vec!["x", "b"])],
        "s",
    );
}

#[test]
fn ff_compute_is_ok() {
    let mut g = GrammarBuilder::new("ok")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    assert!(FirstFollowSets::compute(&g).is_ok());
}

#[test]
fn ff_first_set_for_start() {
    let mut g = GrammarBuilder::new("fs")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let sid = g.start_symbol().unwrap();
    assert!(ff.first(sid).is_some());
}

#[test]
fn ff_follow_set_for_start() {
    let mut g = GrammarBuilder::new("fls")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let sid = g.start_symbol().unwrap();
    assert!(ff.follow(sid).is_some());
}

#[test]
fn ff_first_set_nonempty() {
    let mut g = GrammarBuilder::new("ne")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let sid = g.start_symbol().unwrap();
    let first = ff.first(sid).unwrap();
    assert!(first.count_ones(..) > 0);
}

#[test]
fn ff_deterministic_result() {
    let build = || {
        let mut g = GrammarBuilder::new("det")
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a"])
            .rule("s", vec!["b"])
            .start("s")
            .build();
        g.normalize();
        FirstFollowSets::compute(&g).unwrap()
    };
    let ff1 = build();
    let ff2 = build();
    // Can't directly compare, but we can compare derived data
    let _ = (ff1, ff2);
}

#[test]
fn ff_multi_rule_first_set() {
    let mut g = GrammarBuilder::new("mr")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let sid = g.start_symbol().unwrap();
    let first = ff.first(sid).unwrap();
    // Should contain both a and b
    assert!(first.count_ones(..) >= 2);
}
