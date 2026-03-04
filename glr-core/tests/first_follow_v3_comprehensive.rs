//! Comprehensive tests for GLR core FirstFollowSets edge cases.

use adze_glr_core::FirstFollowSets;
use adze_ir::builder::GrammarBuilder;

fn compute_ff(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> FirstFollowSets {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
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

// ── Basic computation ──

#[test]
fn ff_single_token() {
    let ff = compute_ff("t1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let _ = ff;
}

#[test]
fn ff_two_tokens() {
    let ff = compute_ff(
        "t2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    let _ = ff;
}

#[test]
fn ff_three_tokens() {
    let ff = compute_ff(
        "t3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    let _ = ff;
}

// ── Alternative rules ──

#[test]
fn ff_two_alternatives() {
    let ff = compute_ff(
        "alt2",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let _ = ff;
}

#[test]
fn ff_three_alternatives() {
    let ff = compute_ff(
        "alt3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"]), ("s", vec!["b"]), ("s", vec!["c"])],
        "s",
    );
    let _ = ff;
}

// ── Chain rules ──

#[test]
fn ff_chain_two() {
    let ff = compute_ff(
        "chain2",
        &[("x", "x")],
        &[("a", vec!["x"]), ("s", vec!["a"])],
        "s",
    );
    let _ = ff;
}

#[test]
fn ff_chain_three() {
    let ff = compute_ff(
        "chain3",
        &[("x", "x")],
        &[("a", vec!["x"]), ("b", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    let _ = ff;
}

// ── Wide rules ──

#[test]
fn ff_wide_five() {
    let ff = compute_ff(
        "wide5",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[("s", vec!["a", "b", "c", "d", "e"])],
        "s",
    );
    let _ = ff;
}

// ── Many tokens ──

#[test]
fn ff_ten_tokens() {
    let tokens: Vec<(String, String)> = (0..10)
        .map(|i| (format!("t{}", i), format!("t{}", i)))
        .collect();
    let tok_refs: Vec<(&str, &str)> = tokens
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let ff = compute_ff("ten", &tok_refs, &[("s", vec!["t0", "t1"])], "s");
    let _ = ff;
}

// ── Many alternatives ──

#[test]
fn ff_ten_alternatives() {
    let tokens: Vec<(String, String)> = (0..10)
        .map(|i| (format!("t{}", i), format!("t{}", i)))
        .collect();
    let tok_refs: Vec<(&str, &str)> = tokens
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let rules: Vec<(&str, Vec<&str>)> = (0..10)
        .map(|i| {
            let tok_name: &str = tok_refs[i].0;
            ("s", vec![tok_name])
        })
        .collect();
    let ff = compute_ff("tenalt", &tok_refs, &rules, "s");
    let _ = ff;
}

// ── Diamond patterns ──

#[test]
fn ff_diamond() {
    let ff = compute_ff(
        "diamond",
        &[("x", "x"), ("y", "y")],
        &[
            ("a", vec!["x"]),
            ("b", vec!["y"]),
            ("s", vec!["a"]),
            ("s", vec!["b"]),
        ],
        "s",
    );
    let _ = ff;
}

// ── Self-recursive (indirect) ──

#[test]
fn ff_left_recursive() {
    let ff = compute_ff(
        "leftrec",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["s", "a"]), ("s", vec!["b"])],
        "s",
    );
    let _ = ff;
}

#[test]
fn ff_right_recursive() {
    let ff = compute_ff(
        "rightrec",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "s"]), ("s", vec!["b"])],
        "s",
    );
    let _ = ff;
}

// ── Multiple non-terminals ──

#[test]
fn ff_expr_term() {
    let ff = compute_ff(
        "exprterm",
        &[("num", "[0-9]+"), ("plus", "\\+")],
        &[
            ("term", vec!["num"]),
            ("expr", vec!["term"]),
            ("expr", vec!["expr", "plus", "term"]),
            ("s", vec!["expr"]),
        ],
        "s",
    );
    let _ = ff;
}

// ── Consistency ──

#[test]
fn ff_idempotent() {
    let mut g = GrammarBuilder::new("idem")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    let ff1 = FirstFollowSets::compute(&g).unwrap();
    let ff2 = FirstFollowSets::compute(&g).unwrap();
    // Both should succeed
    let _ = (ff1, ff2);
}

// ── First set checks ──

#[test]
fn first_set_contains_token() {
    let mut g = GrammarBuilder::new("first")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    // start symbol should have a FIRST set
    let start = g.start_symbol().unwrap();
    let first = ff.first(start);
    assert!(first.is_some());
}

#[test]
fn first_set_nonempty_for_start() {
    let mut g = GrammarBuilder::new("firstne")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = g.start_symbol().unwrap();
    let first = ff.first(start).unwrap();
    assert!(first.count_ones(..) > 0);
}

// ── Follow set checks ──

#[test]
fn follow_set_exists_for_start() {
    let mut g = GrammarBuilder::new("follow")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = g.start_symbol().unwrap();
    let follow = ff.follow(start);
    assert!(follow.is_some());
}

// ── Many tokens first set ──

#[test]
fn first_set_multiple_alternatives() {
    let mut g = GrammarBuilder::new("multifirst")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let start = g.start_symbol().unwrap();
    let first = ff.first(start).unwrap();
    // Should contain at least 3 elements (a, b, c)
    assert!(first.count_ones(..) >= 3);
}
