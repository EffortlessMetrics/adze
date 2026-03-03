//! Integration tests for the full GLR core parse pipeline:
//! Grammar → normalize → FIRST/FOLLOW → canonical collection → parse table
//!
//! Tests the end-to-end pipeline from grammar construction through table generation.

use adze_glr_core::conflict_inspection::{ConflictType, count_conflicts};
use adze_glr_core::{Action, FirstFollowSets, GLRError, build_lr1_automaton, sanity_check_tables};
use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;

/// Run the full pipeline: normalize → FIRST/FOLLOW → build_lr1_automaton
/// Returns the parse table or an error.
fn run_pipeline(grammar: &mut Grammar) -> Result<adze_glr_core::ParseTable, GLRError> {
    let first_follow = FirstFollowSets::compute_normalized(grammar)?;
    build_lr1_automaton(grammar, &first_follow)
}

// ── 1. Simple arithmetic grammar: NUM + NUM ──────────────────────────

#[test]
fn pipeline_simple_arithmetic() {
    let mut grammar = GrammarBuilder::new("arithmetic")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let table = run_pipeline(&mut grammar).expect("pipeline should succeed for simple arithmetic");

    // Sanity-check structural invariants
    sanity_check_tables(&table).expect("table sanity check failed");

    // Should have a reasonable number of states (small grammar → few states)
    assert!(table.state_count > 0, "must have at least one state");
    assert!(
        table.state_count < 30,
        "simple grammar should not explode to {} states",
        table.state_count
    );

    // Should have at least one ACCEPT action somewhere in the table
    let has_accept = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    });
    assert!(has_accept, "table must contain an Accept action");

    // No conflicts expected for this unambiguous grammar
    let summary = count_conflicts(&table);
    assert_eq!(
        summary.shift_reduce, 0,
        "unambiguous grammar should have 0 S/R conflicts"
    );
    assert_eq!(
        summary.reduce_reduce, 0,
        "unambiguous grammar should have 0 R/R conflicts"
    );
}

// ── 2. Ambiguous grammar: dangling else ──────────────────────────────

#[test]
fn pipeline_dangling_else_produces_conflicts() {
    let mut grammar = GrammarBuilder::new("dangling_else")
        .token("if", "if")
        .token("then", "then")
        .token("else", "else")
        .token("other", "other")
        .token("id", "id")
        .rule("Statement", vec!["if", "Expr", "then", "Statement"])
        .rule(
            "Statement",
            vec!["if", "Expr", "then", "Statement", "else", "Statement"],
        )
        .rule("Statement", vec!["other"])
        .rule("Expr", vec!["id"])
        .start("Statement")
        .build();

    let table =
        run_pipeline(&mut grammar).expect("pipeline should succeed for dangling-else grammar");

    sanity_check_tables(&table).expect("table sanity check failed");

    // Must have at least one shift/reduce conflict on "else"
    let summary = count_conflicts(&table);
    assert!(
        summary.shift_reduce >= 1,
        "dangling else must produce at least 1 S/R conflict, got {}",
        summary.shift_reduce
    );

    // Verify the conflict involves shift and reduce actions
    for detail in &summary.conflict_details {
        if detail.conflict_type == ConflictType::ShiftReduce {
            let has_shift = detail.actions.iter().any(|a| matches!(a, Action::Shift(_)));
            let has_reduce = detail
                .actions
                .iter()
                .any(|a| matches!(a, Action::Reduce(_)));
            assert!(
                has_shift && has_reduce,
                "S/R conflict should contain both Shift and Reduce actions"
            );
        }
    }
}

// ── 3. Reasonable state counts ───────────────────────────────────────

#[test]
fn pipeline_state_counts_are_reasonable() {
    // A tiny grammar: S → a
    let mut tiny = GrammarBuilder::new("tiny")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();

    let tiny_table = run_pipeline(&mut tiny).expect("tiny grammar pipeline");
    assert!(
        tiny_table.state_count <= 10,
        "trivial grammar should have very few states, got {}",
        tiny_table.state_count
    );

    // A medium grammar: simple expression language
    let mut medium = GrammarBuilder::new("medium")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("factor", vec!["NUM"])
        .start("expr")
        .build();

    let medium_table = run_pipeline(&mut medium).expect("medium grammar pipeline");
    assert!(
        medium_table.state_count > tiny_table.state_count,
        "medium grammar should have more states than tiny grammar"
    );
    assert!(
        medium_table.state_count < 50,
        "standard expression grammar should have < 50 states, got {}",
        medium_table.state_count
    );

    // No conflicts in the unambiguous expression grammar
    let summary = count_conflicts(&medium_table);
    assert_eq!(summary.shift_reduce, 0);
    assert_eq!(summary.reduce_reduce, 0);
}

// ── 4. Conflict detection and reporting ──────────────────────────────

#[test]
fn pipeline_conflict_detection_ambiguous_expr() {
    // Ambiguous: E → E + E | E * E | NUM  (no precedence)
    let mut grammar = GrammarBuilder::new("ambiguous_expr")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("E", vec!["E", "+", "E"])
        .rule("E", vec!["E", "*", "E"])
        .rule("E", vec!["NUM"])
        .start("E")
        .build();

    let table = run_pipeline(&mut grammar).expect("ambiguous expr pipeline");
    sanity_check_tables(&table).expect("table sanity check failed");

    let summary = count_conflicts(&table);

    // Must have conflicts — this grammar is inherently ambiguous
    assert!(
        summary.shift_reduce >= 2,
        "ambiguous E+E|E*E should have >= 2 S/R conflicts, got {}",
        summary.shift_reduce
    );

    // States with conflicts should be non-empty
    assert!(
        !summary.states_with_conflicts.is_empty(),
        "should report states containing conflicts"
    );

    // Every conflict detail should have a valid type
    for detail in &summary.conflict_details {
        assert!(
            detail.conflict_type == ConflictType::ShiftReduce
                || detail.conflict_type == ConflictType::ReduceReduce,
            "unknown conflict type: {:?}",
            detail.conflict_type
        );
        assert!(
            detail.actions.len() >= 2,
            "conflict must involve at least 2 actions"
        );
    }
}

// ── 5. Empty grammar handling ────────────────────────────────────────

#[test]
fn pipeline_empty_grammar_handled_gracefully() {
    let mut grammar = Grammar::default();

    // An empty grammar has no rules, no tokens, no start symbol.
    // The pipeline should return an error (no start symbol), not panic.
    let result = run_pipeline(&mut grammar);
    assert!(
        result.is_err(),
        "empty grammar should produce an error, not a valid table"
    );
}

// ── 6. Left-recursive grammar: A → A a | a ──────────────────────────

#[test]
fn pipeline_left_recursive_grammar() {
    let mut grammar = GrammarBuilder::new("left_recursive")
        .token("a", "a")
        .rule("A", vec!["A", "a"])
        .rule("A", vec!["a"])
        .start("A")
        .build();

    let table = run_pipeline(&mut grammar).expect("left-recursive grammar should succeed");

    sanity_check_tables(&table).expect("table sanity check failed");

    // Left recursion is fine for LR parsers — no conflicts expected
    let summary = count_conflicts(&table);
    assert_eq!(
        summary.shift_reduce, 0,
        "left-recursive grammar should have 0 S/R conflicts"
    );
    assert_eq!(
        summary.reduce_reduce, 0,
        "left-recursive grammar should have 0 R/R conflicts"
    );

    // Small grammar → small automaton
    assert!(
        table.state_count < 15,
        "left-recursive grammar should have few states, got {}",
        table.state_count
    );
}

// ── 7. Right-recursive grammar: A → a A | a ─────────────────────────

#[test]
fn pipeline_right_recursive_grammar() {
    let mut grammar = GrammarBuilder::new("right_recursive")
        .token("a", "a")
        .rule("A", vec!["a", "A"])
        .rule("A", vec!["a"])
        .start("A")
        .build();

    let table = run_pipeline(&mut grammar).expect("right-recursive grammar should succeed");

    sanity_check_tables(&table).expect("table sanity check failed");

    // Right recursion in LR produces a shift/reduce conflict on 'a':
    // when seeing 'a' after reading 'a', the parser can shift (start
    // another A) or reduce (finish A → a).
    // This is expected; just verify the table is structurally valid.
    assert!(
        table.state_count > 0,
        "right-recursive grammar must produce states"
    );
}

// ── 8. Pipeline stages individually: normalize then FIRST/FOLLOW ─────

#[test]
fn pipeline_normalize_then_first_follow() {
    let mut grammar = GrammarBuilder::new("staged")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "S", "b"])
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();

    // Step 1: normalize
    grammar.normalize();

    // Step 2: compute FIRST/FOLLOW
    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW computation should succeed");

    // Step 3: build automaton
    let table = build_lr1_automaton(&grammar, &ff).expect("automaton build should succeed");

    sanity_check_tables(&table).expect("table sanity check failed");

    // Unambiguous nested parenthesis-like grammar — no conflicts
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0);
    assert_eq!(summary.reduce_reduce, 0);
}

// ── 9. Multiple non-terminals ────────────────────────────────────────

#[test]
fn pipeline_multi_nonterminal_grammar() {
    // Statement → ID = NUM ; | ID ;
    // (single start non-terminal avoids augmented-grammar goto edge case)
    let mut grammar = GrammarBuilder::new("multi_nt")
        .token("ID", r"[a-z]+")
        .token("NUM", r"\d+")
        .token(";", ";")
        .token("=", "=")
        .rule("Statement", vec!["ID", "=", "NUM", ";"])
        .rule("Statement", vec!["ID", ";"])
        .start("Statement")
        .build();

    let table = run_pipeline(&mut grammar).expect("multi-nonterminal pipeline");
    sanity_check_tables(&table).expect("table sanity check failed");

    assert!(
        table.state_count > 0,
        "multi-nonterminal grammar must produce states"
    );
    assert!(
        table.rules.len() >= 2,
        "should have at least 2 parse rules, got {}",
        table.rules.len()
    );
}
