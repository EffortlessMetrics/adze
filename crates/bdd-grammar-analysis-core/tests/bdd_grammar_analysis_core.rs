//! BDD-style tests for bdd-grammar-analysis-core crate.
//!
//! Tests follow the Given/When/Then pattern to verify public API behavior.

use adze_bdd_grammar_analysis_core::{
    ConflictAnalysis, analyze_conflicts, count_multi_action_cells,
};
use adze_glr_core::{Action, ParseTable, RuleId, StateId};

/// Helper to create a parse table with given action table.
fn make_table(action_table: Vec<Vec<Vec<Action>>>, states: usize, symbols: usize) -> ParseTable {
    ParseTable {
        action_table,
        state_count: states,
        symbol_count: symbols,
        ..Default::default()
    }
}

#[test]
fn given_empty_table_when_counting_multi_action_cells_then_returns_zero() {
    // Given
    let pt = ParseTable::default();

    // When
    let count = count_multi_action_cells(&pt);

    // Then
    assert_eq!(count, 0);
}

#[test]
fn given_table_with_single_action_cells_when_counting_multi_action_cells_then_returns_zero() {
    // Given
    let pt = make_table(
        vec![vec![
            vec![Action::Shift(StateId(1))],
            vec![Action::Reduce(RuleId(0))],
        ]],
        1,
        2,
    );

    // When
    let count = count_multi_action_cells(&pt);

    // Then
    assert_eq!(count, 0);
}

#[test]
fn given_table_with_shift_reduce_conflict_when_counting_multi_action_cells_then_returns_one() {
    // Given
    let pt = make_table(
        vec![vec![vec![
            Action::Shift(StateId(1)),
            Action::Reduce(RuleId(0)),
        ]]],
        1,
        1,
    );

    // When
    let count = count_multi_action_cells(&pt);

    // Then
    assert_eq!(count, 1);
}

#[test]
fn given_table_with_reduce_reduce_conflict_when_counting_multi_action_cells_then_returns_one() {
    // Given
    let pt = make_table(
        vec![vec![vec![
            Action::Reduce(RuleId(0)),
            Action::Reduce(RuleId(1)),
        ]]],
        1,
        1,
    );

    // When
    let count = count_multi_action_cells(&pt);

    // Then
    assert_eq!(count, 1);
}

#[test]
fn given_empty_table_when_analyzing_conflicts_then_returns_zero_counts() {
    // Given
    let pt = ParseTable::default();

    // When
    let analysis = analyze_conflicts(&pt);

    // Then
    assert_eq!(analysis.total_conflicts, 0);
    assert_eq!(analysis.shift_reduce_conflicts, 0);
    assert_eq!(analysis.reduce_reduce_conflicts, 0);
    assert!(analysis.conflict_details.is_empty());
}

#[test]
fn given_table_with_shift_reduce_conflict_when_analyzing_then_classifies_correctly() {
    // Given
    let pt = make_table(
        vec![vec![vec![
            Action::Shift(StateId(1)),
            Action::Reduce(RuleId(0)),
        ]]],
        1,
        1,
    );

    // When
    let analysis = analyze_conflicts(&pt);

    // Then
    assert_eq!(analysis.total_conflicts, 1);
    assert_eq!(analysis.shift_reduce_conflicts, 1);
    assert_eq!(analysis.reduce_reduce_conflicts, 0);
}

#[test]
fn given_table_with_reduce_reduce_conflict_when_analyzing_then_classifies_correctly() {
    // Given
    let pt = make_table(
        vec![vec![vec![
            Action::Reduce(RuleId(0)),
            Action::Reduce(RuleId(1)),
        ]]],
        1,
        1,
    );

    // When
    let analysis = analyze_conflicts(&pt);

    // Then
    assert_eq!(analysis.total_conflicts, 1);
    assert_eq!(analysis.shift_reduce_conflicts, 0);
    assert_eq!(analysis.reduce_reduce_conflicts, 1);
}

#[test]
fn given_table_with_multiple_conflicts_when_analyzing_then_counts_all() {
    // Given
    let pt = make_table(
        vec![
            vec![
                vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
                vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))],
            ],
            vec![vec![Action::Accept], vec![Action::Shift(StateId(2))]],
        ],
        2,
        2,
    );

    // When
    let analysis = analyze_conflicts(&pt);

    // Then
    assert_eq!(analysis.total_conflicts, 2);
    assert_eq!(analysis.shift_reduce_conflicts, 1);
    assert_eq!(analysis.reduce_reduce_conflicts, 1);
}

#[test]
fn given_conflict_analysis_when_formatting_debug_then_contains_field_names() {
    // Given
    let analysis = ConflictAnalysis {
        total_conflicts: 5,
        shift_reduce_conflicts: 3,
        reduce_reduce_conflicts: 2,
        conflict_details: vec![],
    };

    // When
    let debug_str = format!("{:?}", analysis);

    // Then
    assert!(debug_str.contains("total_conflicts"));
    assert!(debug_str.contains("shift_reduce_conflicts"));
    assert!(debug_str.contains("reduce_reduce_conflicts"));
}

#[test]
fn given_table_with_conflicts_when_analyzing_then_details_are_populated() {
    // Given
    let pt = make_table(
        vec![vec![vec![
            Action::Shift(StateId(1)),
            Action::Reduce(RuleId(0)),
        ]]],
        1,
        1,
    );

    // When
    let analysis = analyze_conflicts(&pt);

    // Then
    assert_eq!(analysis.conflict_details.len(), 1);
    let (state, symbol, actions) = &analysis.conflict_details[0];
    assert_eq!(*state, 0);
    assert_eq!(*symbol, 0);
    assert_eq!(actions.len(), 2);
}

#[test]
fn given_table_without_conflicts_when_analyzing_then_details_are_empty() {
    // Given
    let pt = make_table(vec![vec![vec![Action::Shift(StateId(1))]]], 1, 1);

    // When
    let analysis = analyze_conflicts(&pt);

    // Then
    assert!(analysis.conflict_details.is_empty());
}

#[test]
fn given_count_function_when_compared_to_analysis_then_counts_match() {
    // Given
    let pt = make_table(
        vec![
            vec![
                vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
                vec![Action::Accept],
            ],
            vec![vec![Action::Accept], vec![Action::Accept]],
        ],
        2,
        2,
    );

    // When
    let count = count_multi_action_cells(&pt);
    let analysis = analyze_conflicts(&pt);

    // Then
    assert_eq!(count, analysis.total_conflicts);
}
