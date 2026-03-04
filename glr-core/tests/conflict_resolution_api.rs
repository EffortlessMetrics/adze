//! Comprehensive tests for the conflict resolution API.
#![cfg(feature = "test-api")]

use adze_glr_core::conflict_inspection::{
    self, ConflictDetail, ConflictSummary, ConflictType as InspectionConflictType,
};
use adze_glr_core::precedence_compare::{
    PrecedenceComparison, PrecedenceInfo, StaticPrecedenceResolver, compare_precedences,
};
use adze_glr_core::*;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, SymbolId};

// ---------------------------------------------------------------------------
// Helper: build a grammar and compute FIRST/FOLLOW
// ---------------------------------------------------------------------------

fn build_ff(mut g: adze_ir::Grammar) -> (adze_ir::Grammar, FirstFollowSets) {
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW computation failed");
    (g, ff)
}

// ===== 1. ConflictResolver creation and usage =====

#[test]
fn conflict_resolver_on_unambiguous_grammar() {
    let g = GrammarBuilder::new("simple")
        .token("x", "x")
        .rule("start", vec!["x"])
        .build();
    let (g, ff) = build_ff(g);

    let collection = ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);

    assert!(
        resolver.conflicts.is_empty(),
        "unambiguous grammar should have no conflicts"
    );
}

#[test]
fn conflict_resolver_detects_shift_reduce() {
    // Classic dangling-else ambiguity
    let g = GrammarBuilder::new("if_else")
        .token("if", "if")
        .token("then", "then")
        .token("else", "else")
        .token("x", "x")
        .rule("stmt", vec!["if", "x", "then", "stmt"])
        .rule("stmt", vec!["if", "x", "then", "stmt", "else", "stmt"])
        .rule("stmt", vec!["x"])
        .build();
    let (g, ff) = build_ff(g);

    let collection = ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);

    assert!(
        !resolver.conflicts.is_empty(),
        "dangling-else grammar must produce conflicts"
    );

    let has_sr = resolver
        .conflicts
        .iter()
        .any(|c| c.conflict_type == ConflictType::ShiftReduce);
    assert!(has_sr, "should contain a shift/reduce conflict");
}

#[test]
fn conflict_resolver_detects_reduce_reduce() {
    // Ambiguous: two ways to derive the same token
    let g = GrammarBuilder::new("rr")
        .token("a", "a")
        .rule("start", vec!["p"])
        .rule("start", vec!["q"])
        .rule("p", vec!["a"])
        .rule("q", vec!["a"])
        .build();
    let (g, ff) = build_ff(g);

    let collection = ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);

    let has_rr = resolver
        .conflicts
        .iter()
        .any(|c| c.conflict_type == ConflictType::ReduceReduce);
    assert!(has_rr, "should contain a reduce/reduce conflict");
}

// ===== 2. ConflictType enum variants =====

#[test]
fn conflict_type_shift_reduce_variant() {
    let ct = ConflictType::ShiftReduce;
    let debug = format!("{:?}", ct);
    assert!(debug.contains("ShiftReduce"));
}

#[test]
fn conflict_type_reduce_reduce_variant() {
    let ct = ConflictType::ReduceReduce;
    let debug = format!("{:?}", ct);
    assert!(debug.contains("ReduceReduce"));
}

#[test]
fn conflict_type_equality() {
    assert_eq!(ConflictType::ShiftReduce, ConflictType::ShiftReduce);
    assert_eq!(ConflictType::ReduceReduce, ConflictType::ReduceReduce);
    assert_ne!(ConflictType::ShiftReduce, ConflictType::ReduceReduce);
}

// ===== 3. Conflict struct fields =====

#[test]
fn conflict_struct_fields_accessible() {
    let c = Conflict {
        state: StateId(7),
        symbol: SymbolId(3),
        actions: vec![Action::Shift(StateId(4)), Action::Reduce(RuleId(1))],
        conflict_type: ConflictType::ShiftReduce,
    };

    assert_eq!(c.state, StateId(7));
    assert_eq!(c.symbol, SymbolId(3));
    assert_eq!(c.actions.len(), 2);
    assert_eq!(c.conflict_type, ConflictType::ShiftReduce);
}

// ===== 4. Conflict inspection module =====

#[test]
fn classify_conflict_shift_reduce() {
    let actions = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
    let ct = conflict_inspection::classify_conflict(&actions);
    assert_eq!(ct, InspectionConflictType::ShiftReduce);
}

#[test]
fn classify_conflict_reduce_reduce() {
    let actions = vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))];
    let ct = conflict_inspection::classify_conflict(&actions);
    assert_eq!(ct, InspectionConflictType::ReduceReduce);
}

#[test]
fn classify_conflict_mixed() {
    let actions = vec![Action::Shift(StateId(0)), Action::Shift(StateId(1))];
    let ct = conflict_inspection::classify_conflict(&actions);
    assert_eq!(ct, InspectionConflictType::Mixed);
}

#[test]
fn classify_conflict_fork_recursive() {
    let actions = vec![Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(2)),
    ])];
    let ct = conflict_inspection::classify_conflict(&actions);
    assert_eq!(ct, InspectionConflictType::ShiftReduce);
}

#[test]
fn count_conflicts_empty_table() {
    let mut pt = ParseTable::default();
    // Must have at least one state for invariant check
    pt.action_table.push(vec![vec![Action::Shift(StateId(0))]]);
    pt.state_count = 1;

    let summary = conflict_inspection::count_conflicts(&pt);
    assert_eq!(summary.shift_reduce, 0);
    assert_eq!(summary.reduce_reduce, 0);
    assert!(summary.states_with_conflicts.is_empty());
}

#[test]
fn count_conflicts_with_shift_reduce_cell() {
    let mut pt = ParseTable::default();
    pt.action_table.push(vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
    ]]);
    pt.state_count = 1;

    let summary = conflict_inspection::count_conflicts(&pt);
    assert_eq!(summary.shift_reduce, 1);
    assert_eq!(summary.reduce_reduce, 0);
    assert_eq!(summary.states_with_conflicts, vec![StateId(0)]);
    assert_eq!(summary.conflict_details.len(), 1);
    assert_eq!(
        summary.conflict_details[0].conflict_type,
        InspectionConflictType::ShiftReduce
    );
}

#[test]
fn state_has_conflicts_function() {
    let mut pt = ParseTable::default();
    // State 0: conflict
    pt.action_table.push(vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
    ]]);
    // State 1: no conflict
    pt.action_table.push(vec![vec![Action::Accept]]);
    pt.state_count = 2;

    assert!(conflict_inspection::state_has_conflicts(&pt, StateId(0)));
    assert!(!conflict_inspection::state_has_conflicts(&pt, StateId(1)));
    // Out-of-bounds state
    assert!(!conflict_inspection::state_has_conflicts(&pt, StateId(99)));
}

#[test]
fn conflict_summary_display() {
    let summary = ConflictSummary {
        shift_reduce: 2,
        reduce_reduce: 1,
        states_with_conflicts: vec![StateId(0), StateId(3)],
        conflict_details: vec![ConflictDetail {
            state: StateId(0),
            symbol: SymbolId(5),
            symbol_name: "plus".to_string(),
            conflict_type: InspectionConflictType::ShiftReduce,
            actions: vec![Action::Shift(StateId(2)), Action::Reduce(RuleId(1))],
            priorities: vec![0, 0],
        }],
    };

    let display = format!("{}", summary);
    assert!(display.contains("Shift/Reduce conflicts: 2"));
    assert!(display.contains("Reduce/Reduce conflicts: 1"));
    assert!(display.contains("plus"));
}

// ===== 5. Precedence comparison =====

#[test]
fn compare_precedences_higher_shift_wins() {
    let shift = PrecedenceInfo {
        level: 3,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    let reduce = PrecedenceInfo {
        level: 1,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(shift), Some(reduce)),
        PrecedenceComparison::PreferShift,
    );
}

#[test]
fn compare_precedences_higher_reduce_wins() {
    let shift = PrecedenceInfo {
        level: 1,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    let reduce = PrecedenceInfo {
        level: 5,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(shift), Some(reduce)),
        PrecedenceComparison::PreferReduce,
    );
}

#[test]
fn compare_precedences_left_assoc_prefers_reduce() {
    let prec = PrecedenceInfo {
        level: 2,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(prec), Some(prec)),
        PrecedenceComparison::PreferReduce,
    );
}

#[test]
fn compare_precedences_right_assoc_prefers_shift() {
    let shift = PrecedenceInfo {
        level: 2,
        associativity: Associativity::Right,
        is_fragile: false,
    };
    let reduce = PrecedenceInfo {
        level: 2,
        associativity: Associativity::Right,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(shift), Some(reduce)),
        PrecedenceComparison::PreferShift,
    );
}

#[test]
fn compare_precedences_none_assoc_error() {
    let prec = PrecedenceInfo {
        level: 2,
        associativity: Associativity::None,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(Some(prec), Some(prec)),
        PrecedenceComparison::Error,
    );
}

#[test]
fn compare_precedences_missing_info_returns_none() {
    let prec = PrecedenceInfo {
        level: 1,
        associativity: Associativity::Left,
        is_fragile: false,
    };
    assert_eq!(
        compare_precedences(None, Some(prec)),
        PrecedenceComparison::None,
    );
    assert_eq!(
        compare_precedences(Some(prec), None),
        PrecedenceComparison::None,
    );
    assert_eq!(compare_precedences(None, None), PrecedenceComparison::None,);
}

#[test]
fn static_precedence_resolver_from_grammar() {
    let g = GrammarBuilder::new("arith")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .build();

    let resolver = StaticPrecedenceResolver::from_grammar(&g);

    // Rules with precedence should be extractable
    let r0 = resolver.rule_precedence(RuleId(0));
    assert!(r0.is_some());
    assert_eq!(r0.unwrap().level, 1);
    assert_eq!(r0.unwrap().associativity, Associativity::Left);

    let r1 = resolver.rule_precedence(RuleId(1));
    assert!(r1.is_some());
    assert_eq!(r1.unwrap().level, 2);

    // Rule without precedence
    let r2 = resolver.rule_precedence(RuleId(2));
    assert!(r2.is_none());
}

#[test]
fn static_precedence_resolver_token_precedence() {
    use adze_ir::{Grammar, Precedence};

    let mut g = Grammar::new("prec_test".to_string());
    g.precedences.push(Precedence {
        level: 10,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(42)],
    });

    let resolver = StaticPrecedenceResolver::from_grammar(&g);
    let p = resolver.token_precedence(SymbolId(42));
    assert!(p.is_some());
    assert_eq!(p.unwrap().level, 10);
    assert_eq!(p.unwrap().associativity, Associativity::Right);

    assert!(resolver.token_precedence(SymbolId(999)).is_none());
}

// ===== Integration: end-to-end grammar → conflict detection =====

#[test]
fn end_to_end_ambiguous_expr_grammar() {
    // E → E + E | E * E | num  — inherently ambiguous
    let g = GrammarBuilder::new("ambiguous_expr")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["expr", "star", "expr"])
        .rule("expr", vec!["num"])
        .build();
    let (g, ff) = build_ff(g);

    let collection = ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, &g, &ff);

    // An ambiguous expression grammar must produce conflicts
    assert!(
        !resolver.conflicts.is_empty(),
        "ambiguous expr grammar should have conflicts, got none"
    );

    // Every detected conflict should have at least 2 actions
    for c in &resolver.conflicts {
        assert!(
            c.actions.len() >= 2,
            "conflict should have ≥2 actions, got {}",
            c.actions.len()
        );
    }
}
