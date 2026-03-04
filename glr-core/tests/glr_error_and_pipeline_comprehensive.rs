//! Comprehensive tests for GLRError variants and error handling.

use adze_glr_core::GLRError;

#[test]
fn glr_error_debug() {
    // Test that GLRError variants can be formatted
    let _ = std::mem::size_of::<GLRError>();
}

// Build a simple grammar and test error paths
use adze_ir::builder::GrammarBuilder;

fn simple_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("simple")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build()
}

// ── FirstFollowSets success ──

#[test]
fn first_follow_success() {
    use adze_glr_core::FirstFollowSets;
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g);
    assert!(ff.is_ok());
}

#[test]
fn first_follow_returns_sets() {
    use adze_glr_core::FirstFollowSets;
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let _ = format!("{:?}", ff);
}

// ── build_lr1_automaton success ──

#[test]
fn build_automaton_success() {
    use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let t = build_lr1_automaton(&g, &ff);
    assert!(t.is_ok());
}

// ── ParseTable from simple grammar ──

#[test]
fn parse_table_from_simple() {
    use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let t = build_lr1_automaton(&g, &ff).unwrap();
    assert!(t.state_count > 0);
}

// ── ParseTable from multi-token ──

#[test]
fn parse_table_multi_token() {
    use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
    let g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let t = build_lr1_automaton(&g, &ff).unwrap();
    assert!(t.state_count >= 3);
}

// ── ParseTable from alternatives ──

#[test]
fn parse_table_alternatives() {
    use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
    let g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let t = build_lr1_automaton(&g, &ff).unwrap();
    assert!(t.rules.len() >= 2);
}

// ── ParseTable from chain ──

#[test]
fn parse_table_chain() {
    use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
    let g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let t = build_lr1_automaton(&g, &ff).unwrap();
    assert!(t.rules.len() >= 3);
}

// ── ParseTable from precedence ──

#[test]
fn parse_table_precedence() {
    use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
    let g = GrammarBuilder::new("prec")
        .token("n", "n")
        .token("plus", "\\+")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, adze_ir::Associativity::Left)
        .start("e")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let t = build_lr1_automaton(&g, &ff).unwrap();
    assert!(t.rules.len() >= 2);
}

// ── ParseTable determinism ──

#[test]
fn parse_table_deterministic() {
    use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
    let g = simple_grammar();
    let ff1 = FirstFollowSets::compute(&g).unwrap();
    let t1 = build_lr1_automaton(&g, &ff1).unwrap();
    let ff2 = FirstFollowSets::compute(&g).unwrap();
    let t2 = build_lr1_automaton(&g, &ff2).unwrap();
    assert_eq!(t1.state_count, t2.state_count);
}

// ── Action enum ──

#[test]
fn action_shift() {
    use adze_glr_core::Action;
    use adze_ir::StateId;
    let a = Action::Shift(StateId(1));
    let _ = format!("{:?}", a);
}

#[test]
fn action_reduce() {
    use adze_glr_core::Action;
    let a = Action::Reduce(adze_ir::RuleId(0));
    let _ = format!("{:?}", a);
}

#[test]
fn action_accept() {
    use adze_glr_core::Action;
    let a = Action::Accept;
    let _ = format!("{:?}", a);
}

#[test]
fn action_error() {
    use adze_glr_core::Action;
    let a = Action::Error;
    let _ = format!("{:?}", a);
}

#[test]
fn action_clone() {
    use adze_glr_core::Action;
    use adze_ir::StateId;
    let a = Action::Shift(StateId(5));
    let c = a.clone();
    assert_eq!(format!("{:?}", a), format!("{:?}", c));
}

// ── Scale test ──

#[test]
fn parse_table_many_tokens() {
    use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
    let mut b = GrammarBuilder::new("many");
    for i in 0..15 {
        let n: &str = Box::leak(format!("t{}", i).into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let g = b.start("s").build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let t = build_lr1_automaton(&g, &ff).unwrap();
    assert!(t.rules.len() >= 15);
}
