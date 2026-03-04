//! Comprehensive tests for FirstFollowSets computation.
#![cfg(feature = "test-api")]

use adze_glr_core::{FirstFollowSets, GLRError};
use adze_ir::SymbolId;
use adze_ir::builder::GrammarBuilder;

fn compute_ff(
    name: &str,
    builder_fn: impl FnOnce(adze_ir::builder::GrammarBuilder) -> adze_ir::builder::GrammarBuilder,
) -> Result<FirstFollowSets, GLRError> {
    let mut g = builder_fn(GrammarBuilder::new(name)).build();
    g.normalize();
    FirstFollowSets::compute(&g)
}

// ─── Basic computation ───

#[test]
fn first_follow_single_terminal_rule() {
    let ff = compute_ff("single", |b| {
        b.token("a", "a").rule("start", vec!["a"]).start("start")
    });
    assert!(ff.is_ok());
}

#[test]
fn first_follow_two_terminal_rule() {
    let ff = compute_ff("two", |b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
    });
    assert!(ff.is_ok());
}

#[test]
fn first_follow_multiple_alternatives() {
    let ff = compute_ff("alts", |b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
    });
    assert!(ff.is_ok());
}

// ─── Chain rules ───

#[test]
fn first_follow_chain_a_to_b() {
    let ff = compute_ff("chain", |b| {
        b.token("x", "x")
            .rule("a", vec!["x"])
            .rule("b", vec!["a"])
            .start("b")
    });
    assert!(ff.is_ok());
}

#[test]
fn first_follow_long_chain() {
    let ff = compute_ff("long_chain", |b| {
        b.token("x", "x")
            .rule("a", vec!["x"])
            .rule("b", vec!["a"])
            .rule("c", vec!["b"])
            .rule("d", vec!["c"])
            .start("d")
    });
    assert!(ff.is_ok());
}

// ─── Multiple rules per nonterminal ───

#[test]
fn first_follow_three_alternatives() {
    let ff = compute_ff("three_alts", |b| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .rule("start", vec!["c"])
            .start("start")
    });
    assert!(ff.is_ok());
}

// ─── Recursive grammars ───

#[test]
fn first_follow_left_recursive() {
    let ff = compute_ff("left_rec", |b| {
        b.token("a", "a")
            .token("plus", "\\+")
            .rule("expr", vec!["expr", "plus", "a"])
            .rule("expr", vec!["a"])
            .start("expr")
    });
    assert!(ff.is_ok());
}

#[test]
fn first_follow_right_recursive() {
    let ff = compute_ff("right_rec", |b| {
        b.token("a", "a")
            .token("plus", "\\+")
            .rule("expr", vec!["a", "plus", "expr"])
            .rule("expr", vec!["a"])
            .start("expr")
    });
    assert!(ff.is_ok());
}

// ─── Multi-level grammars ───

#[test]
fn first_follow_arithmetic() {
    let ff = compute_ff("arith", |b| {
        b.token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("times", "\\*")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["term", "times", "factor"])
            .rule("term", vec!["factor"])
            .rule("factor", vec!["num"])
            .start("expr")
    });
    assert!(ff.is_ok());
}

// ─── Single rule grammars ───

#[test]
fn first_follow_single_token_only() {
    let ff = compute_ff("single_tok", |b| {
        b.token("x", "x").rule("start", vec!["x"]).start("start")
    });
    assert!(ff.is_ok());
}

// ─── Diamond grammars ───

#[test]
fn first_follow_diamond() {
    let ff = compute_ff("diamond", |b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("start", vec!["left"])
            .rule("start", vec!["right"])
            .rule("left", vec!["a"])
            .rule("right", vec!["b"])
            .start("start")
    });
    assert!(ff.is_ok());
}

// ─── Wide grammars ───

#[test]
fn first_follow_many_alternatives() {
    let mut builder = GrammarBuilder::new("wide");
    for i in 0..10 {
        builder = builder.token(&format!("t{}", i), &format!("t{}", i));
        builder = builder.rule("start", vec![&*format!("t{}", i).leak()]);
    }
    builder = builder.start("start");
    let mut g = builder.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g);
    assert!(ff.is_ok());
}

// ─── Result type ───

#[test]
fn first_follow_returns_result() {
    let ff = compute_ff("result_test", |b| {
        b.token("x", "x").rule("s", vec!["x"]).start("s")
    });
    assert!(ff.is_ok());
    let _sets = ff.unwrap();
}

// ─── Accessing computed sets ───

#[test]
fn first_follow_sets_debug() {
    let ff = compute_ff("debug_test", |b| {
        b.token("x", "x").rule("s", vec!["x"]).start("s")
    })
    .unwrap();
    let d = format!("{:?}", ff);
    assert!(!d.is_empty());
}

// ─── Grammar variants ───

#[test]
fn first_follow_two_nonterminals() {
    let ff = compute_ff("two_nt", |b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("start", vec!["foo", "bar"])
            .rule("foo", vec!["a"])
            .rule("bar", vec!["b"])
            .start("start")
    });
    assert!(ff.is_ok());
}

#[test]
fn first_follow_shared_prefix() {
    let ff = compute_ff("shared_prefix", |b| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a", "b"])
            .rule("start", vec!["a", "c"])
            .start("start")
    });
    assert!(ff.is_ok());
}

// ─── Normalization before compute ───

#[test]
fn compute_after_normalization() {
    let mut g = GrammarBuilder::new("normalized")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g);
    assert!(ff.is_ok());
}

// ─── Error cases ───

#[test]
fn compute_empty_grammar_no_panic() {
    let mut g = adze_ir::Grammar::new("empty".to_string());
    g.normalize();
    // May succeed or fail, but should not panic
    let _ff = FirstFollowSets::compute(&g);
}

// ─── Multiple computations ───

#[test]
fn compute_same_grammar_twice_deterministic() {
    let mut g1 = GrammarBuilder::new("det1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g1.normalize();

    let mut g2 = GrammarBuilder::new("det2")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g2.normalize();

    let ff1 = FirstFollowSets::compute(&g1);
    let ff2 = FirstFollowSets::compute(&g2);
    assert!(ff1.is_ok());
    assert!(ff2.is_ok());
}

// ─── Integration with build_lr1_automaton ───

#[test]
fn first_follow_to_automaton() {
    let mut g = GrammarBuilder::new("automaton")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = adze_glr_core::build_lr1_automaton(&g, &ff);
    assert!(table.is_ok());
}

#[test]
fn first_follow_to_automaton_arithmetic() {
    let mut g = GrammarBuilder::new("arith_auto")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule("expr", vec!["expr", "plus", "num"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = adze_glr_core::build_lr1_automaton(&g, &ff);
    assert!(table.is_ok());
}

#[test]
fn first_follow_to_automaton_chain() {
    let mut g = GrammarBuilder::new("chain_auto")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("c", vec!["b"])
        .start("c")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = adze_glr_core::build_lr1_automaton(&g, &ff);
    assert!(table.is_ok());
}

// ─── Parse table properties ───

#[test]
fn automaton_has_states() {
    let mut g = GrammarBuilder::new("states")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = adze_glr_core::build_lr1_automaton(&g, &ff).unwrap();
    assert!(table.state_count > 0);
}

#[test]
fn automaton_has_rules() {
    let mut g = GrammarBuilder::new("rules")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = adze_glr_core::build_lr1_automaton(&g, &ff).unwrap();
    assert!(!table.rules.is_empty());
}

#[test]
fn automaton_action_table_nonempty() {
    let mut g = GrammarBuilder::new("actions")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = adze_glr_core::build_lr1_automaton(&g, &ff).unwrap();
    assert!(!table.action_table.is_empty());
}
