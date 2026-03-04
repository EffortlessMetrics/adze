//! Comprehensive tests for ConflictVisualizer.

use adze_glr_core::conflict_visualizer::ConflictVisualizer;
use adze_glr_core::{Action, Conflict, ConflictType};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{RuleId, StateId, SymbolId};

fn shift_reduce_conflict() -> Conflict {
    Conflict {
        state: StateId(0),
        symbol: SymbolId(1),
        actions: vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
        conflict_type: ConflictType::ShiftReduce,
    }
}

fn reduce_reduce_conflict() -> Conflict {
    Conflict {
        state: StateId(0),
        symbol: SymbolId(1),
        actions: vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))],
        conflict_type: ConflictType::ReduceReduce,
    }
}

fn simple_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("test")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["NUM", "PLUS", "NUM"])
        .start("expr")
        .build()
}

// --- ConflictVisualizer tests ---

#[test]
fn visualizer_empty_conflicts() {
    let g = simple_grammar();
    let conflicts: Vec<Conflict> = vec![];
    let viz = ConflictVisualizer::new(&g, &conflicts);
    let report = viz.generate_report();
    assert!(!report.is_empty());
}

#[test]
fn visualizer_single_shift_reduce() {
    let g = simple_grammar();
    let conflicts = vec![shift_reduce_conflict()];
    let viz = ConflictVisualizer::new(&g, &conflicts);
    let report = viz.generate_report();
    assert!(!report.is_empty());
}

#[test]
fn visualizer_single_reduce_reduce() {
    let g = simple_grammar();
    let conflicts = vec![reduce_reduce_conflict()];
    let viz = ConflictVisualizer::new(&g, &conflicts);
    let report = viz.generate_report();
    assert!(!report.is_empty());
}

#[test]
fn visualizer_multiple_conflicts() {
    let g = simple_grammar();
    let conflicts = vec![shift_reduce_conflict(), reduce_reduce_conflict()];
    let viz = ConflictVisualizer::new(&g, &conflicts);
    let report = viz.generate_report();
    assert!(!report.is_empty());
}

#[test]
fn visualizer_report_mentions_state() {
    let g = simple_grammar();
    let conflicts = vec![shift_reduce_conflict()];
    let viz = ConflictVisualizer::new(&g, &conflicts);
    let report = viz.generate_report();
    assert!(
        report.contains("0") || report.contains("state"),
        "Report should mention state: {}",
        report
    );
}

// --- Conflict struct tests ---

#[test]
fn conflict_type_shift_reduce() {
    let ct = ConflictType::ShiftReduce;
    assert_eq!(ct, ConflictType::ShiftReduce);
}

#[test]
fn conflict_type_reduce_reduce() {
    let ct = ConflictType::ReduceReduce;
    assert_eq!(ct, ConflictType::ReduceReduce);
}

#[test]
fn conflict_type_inequality() {
    assert_ne!(ConflictType::ShiftReduce, ConflictType::ReduceReduce);
}

#[test]
fn conflict_type_debug() {
    let ct = ConflictType::ShiftReduce;
    let debug = format!("{:?}", ct);
    assert!(debug.contains("ShiftReduce"));
}

#[test]
fn conflict_type_clone() {
    let ct = ConflictType::ReduceReduce;
    let ct2 = ct.clone();
    assert_eq!(ct, ct2);
}

// --- Conflict struct tests ---

#[test]
fn conflict_struct_fields() {
    let c = shift_reduce_conflict();
    assert_eq!(c.state, StateId(0));
    assert_eq!(c.symbol, SymbolId(1));
    assert_eq!(c.actions.len(), 2);
    assert_eq!(c.conflict_type, ConflictType::ShiftReduce);
}

#[test]
fn conflict_debug() {
    let c = shift_reduce_conflict();
    let debug = format!("{:?}", c);
    assert!(debug.contains("Conflict"));
}

#[test]
fn conflict_clone() {
    let c = shift_reduce_conflict();
    let c2 = c.clone();
    assert_eq!(c.state, c2.state);
    assert_eq!(c.symbol, c2.symbol);
}
