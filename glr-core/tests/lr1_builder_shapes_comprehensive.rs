//! Comprehensive tests for build_lr1_automaton with various grammar shapes.

use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;

fn build_table(
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> adze_glr_core::ParseTable {
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
    let ff = FirstFollowSets::compute(&g).unwrap();
    build_lr1_automaton(&g, &ff).unwrap()
}

#[test]
fn single_token_states() {
    let pt = build_table(&[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(pt.state_count >= 2);
}

#[test]
fn two_alts_states() {
    let pt = build_table(
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(pt.state_count >= 2);
}

#[test]
fn sequence_states() {
    let pt = build_table(&[("a", "a"), ("b", "b")], &[("s", vec!["a", "b"])], "s");
    assert!(pt.state_count >= 3);
}

#[test]
fn chain_states() {
    let pt = build_table(
        &[("x", "x")],
        &[("inner", vec!["x"]), ("s", vec!["inner"])],
        "s",
    );
    assert!(pt.state_count >= 2);
}

#[test]
fn action_table_rows_match_states() {
    let pt = build_table(&[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt.action_table.len(), pt.state_count);
}

#[test]
fn goto_table_rows_match_states() {
    let pt = build_table(&[("a", "a")], &[("s", vec!["a"])], "s");
    assert_eq!(pt.goto_table.len(), pt.state_count);
}

#[test]
fn rules_nonempty() {
    let pt = build_table(&[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(!pt.rules.is_empty());
}

#[test]
fn accept_action_exists() {
    let pt = build_table(&[("a", "a")], &[("s", vec!["a"])], "s");
    let has_accept = pt.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    });
    assert!(has_accept);
}

#[test]
fn symbol_count_positive() {
    let pt = build_table(&[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(pt.symbol_count > 0);
}

#[test]
fn eof_symbol_exists() {
    let pt = build_table(&[("a", "a")], &[("s", vec!["a"])], "s");
    let _ = pt.eof_symbol; // just access it
}

#[test]
fn larger_grammar_more_states() {
    let small = build_table(&[("a", "a")], &[("s", vec!["a"])], "s");
    let big = build_table(
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    assert!(big.state_count >= small.state_count);
}

#[test]
fn parse_rules_have_valid_lhs() {
    let pt = build_table(&[("a", "a")], &[("s", vec!["a"])], "s");
    for rule in &pt.rules {
        assert!(rule.lhs.0 < pt.symbol_count as u16);
    }
}
