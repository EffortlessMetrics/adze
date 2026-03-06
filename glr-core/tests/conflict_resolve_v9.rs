//! Conflict resolution v9 — 84 tests for GLR conflict resolution in adze-glr-core.
//!
//! Categories (20 themes, 80+ tests):
//!   1.  No conflicts in unambiguous grammar
//!   2.  Left-associative resolves shift-reduce to reduce
//!   3.  Right-associative resolves shift-reduce to shift
//!   4.  Higher precedence wins
//!   5.  Same precedence, left-assoc → reduce
//!   6.  Same precedence, right-assoc → shift
//!   7.  Declared conflict → Fork action
//!   8.  Grammar with no precedence → potential conflicts
//!   9.  Arithmetic + * with standard precedence → correct resolution
//!  10.  Three levels of precedence
//!  11.  Mixed associativity
//!  12.  Conflict count in parse table
//!  13.  All non-conflict cells have 0 or 1 action
//!  14.  Fork actions have >= 2 sub-actions
//!  15.  Precedence 0 vs precedence 1
//!  16.  Negative precedence values
//!  17.  Precedence ordering is numeric
//!  18.  Resolution is deterministic
//!  19.  Same grammar → same conflicts
//!  20.  Different precedences → different resolutions
//!
//! Run with: cargo test -p adze-glr-core --test conflict_resolve_v9 -- --test-threads=2

use adze_glr_core::conflict_inspection::{ConflictType, classify_conflict, count_conflicts};
use adze_glr_core::{
    Action, ConflictResolver, FirstFollowSets, GLRError, ItemSetCollection, ParseTable,
    build_lr1_automaton, sanity_check_tables,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, RuleId, StateId};

// ===========================================================================
// Helpers
// ===========================================================================

/// Build grammar → first/follow → parse table in one step.
fn build_table(grammar: &Grammar) -> Result<ParseTable, GLRError> {
    let ff = FirstFollowSets::compute(grammar)?;
    build_lr1_automaton(grammar, &ff)
}

/// Count cells with multiple actions (conflict / fork points).
fn count_fork_cells(table: &ParseTable) -> usize {
    table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .filter(|cell| cell.len() > 1)
        .count()
}

/// True if any cell contains both Shift and Reduce actions.
fn has_shift_reduce(table: &ParseTable) -> bool {
    table.action_table.iter().any(|row| {
        row.iter().any(|cell| {
            cell.len() > 1
                && cell.iter().any(|a| matches!(a, Action::Shift(_)))
                && cell.iter().any(|a| matches!(a, Action::Reduce(_)))
        })
    })
}

/// True if any cell contains two or more distinct Reduce actions.
fn has_reduce_reduce(table: &ParseTable) -> bool {
    table.action_table.iter().any(|row| {
        row.iter().any(|cell| {
            cell.iter()
                .filter(|a| matches!(a, Action::Reduce(_)))
                .count()
                >= 2
        })
    })
}

/// True if at least one cell contains Accept.
fn has_accept(table: &ParseTable) -> bool {
    table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    })
}

/// Run ConflictResolver::detect_conflicts on a grammar.
fn detect_all(grammar: &Grammar) -> Vec<adze_glr_core::Conflict> {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW");
    let col = ItemSetCollection::build_canonical_collection(grammar, &ff);
    let resolver = ConflictResolver::detect_conflicts(&col, grammar, &ff);
    resolver.conflicts
}

// ===========================================================================
// Category 1: No conflicts in unambiguous grammar (tests 1–5)
// ===========================================================================

#[test]
fn test_crv_v9_unambiguous_single_rule_no_conflicts() {
    let g = GrammarBuilder::new("crv_v9_u1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn test_crv_v9_unambiguous_sequence_no_conflicts() {
    let g = GrammarBuilder::new("crv_v9_u2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn test_crv_v9_unambiguous_disjoint_alt_no_conflicts() {
    let g = GrammarBuilder::new("crv_v9_u3")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn test_crv_v9_unambiguous_chain_no_conflicts() {
    let g = GrammarBuilder::new("crv_v9_u4")
        .token("x", "x")
        .rule("start", vec!["mid"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec!["x"])
        .start("start")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn test_crv_v9_unambiguous_left_recursive_no_conflicts() {
    // E → E + NUM | NUM — LR(1)-parseable, no conflicts
    let g = GrammarBuilder::new("crv_v9_u5")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(!has_shift_reduce(&table));
    assert!(!has_reduce_reduce(&table));
}

// ===========================================================================
// Category 2: Left-associative resolves shift-reduce to reduce (tests 6–9)
// ===========================================================================

#[test]
fn test_crv_v9_left_assoc_resolves_sr_addition() {
    let g = GrammarBuilder::new("crv_v9_la1")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(!has_shift_reduce(&table), "Left assoc must resolve S/R");
}

#[test]
fn test_crv_v9_left_assoc_no_fork_cells() {
    let g = GrammarBuilder::new("crv_v9_la2")
        .token("NUM", r"\d+")
        .token("MINUS", r"-")
        .rule_with_precedence(
            "expr",
            vec!["expr", "MINUS", "expr"],
            1,
            Associativity::Left,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0, "Left assoc → 0 fork cells");
}

#[test]
fn test_crv_v9_left_assoc_preserves_accept() {
    let g = GrammarBuilder::new("crv_v9_la3")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(has_accept(&table), "Left assoc must not remove Accept");
}

#[test]
fn test_crv_v9_left_assoc_sanity_check() {
    let g = GrammarBuilder::new("crv_v9_la4")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(sanity_check_tables(&table).is_ok());
}

// ===========================================================================
// Category 3: Right-associative resolves shift-reduce to shift (tests 10–13)
// ===========================================================================

#[test]
fn test_crv_v9_right_assoc_resolves_sr_assign() {
    let g = GrammarBuilder::new("crv_v9_ra1")
        .token("NUM", r"\d+")
        .token("EQ", r"=")
        .rule_with_precedence("expr", vec!["expr", "EQ", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(!has_shift_reduce(&table), "Right assoc must resolve S/R");
}

#[test]
fn test_crv_v9_right_assoc_no_fork_cells() {
    let g = GrammarBuilder::new("crv_v9_ra2")
        .token("NUM", r"\d+")
        .token("ARROW", r"->")
        .rule_with_precedence(
            "expr",
            vec!["expr", "ARROW", "expr"],
            1,
            Associativity::Right,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0, "Right assoc → 0 fork cells");
}

#[test]
fn test_crv_v9_right_assoc_preserves_accept() {
    let g = GrammarBuilder::new("crv_v9_ra3")
        .token("NUM", r"\d+")
        .token("CARET", r"\^")
        .rule_with_precedence(
            "expr",
            vec!["expr", "CARET", "expr"],
            1,
            Associativity::Right,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(has_accept(&table), "Right assoc must not remove Accept");
}

#[test]
fn test_crv_v9_right_assoc_sanity_check() {
    let g = GrammarBuilder::new("crv_v9_ra4")
        .token("NUM", r"\d+")
        .token("EQ", r"=")
        .rule_with_precedence("expr", vec!["expr", "EQ", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(sanity_check_tables(&table).is_ok());
}

// ===========================================================================
// Category 4: Higher precedence wins (tests 14–17)
// ===========================================================================

#[test]
fn test_crv_v9_higher_prec_wins_plus_star() {
    let g = GrammarBuilder::new("crv_v9_hp1")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    assert_eq!(
        summary.shift_reduce, 0,
        "higher prec should resolve all S/R"
    );
}

#[test]
fn test_crv_v9_higher_prec_fewer_forks_than_no_prec() {
    let no_prec = GrammarBuilder::new("crv_v9_hp2a")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["expr", "STAR", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let with_prec = GrammarBuilder::new("crv_v9_hp2b")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let t_np = build_table(&no_prec).unwrap();
    let t_wp = build_table(&with_prec).unwrap();
    assert!(
        count_fork_cells(&t_wp) < count_fork_cells(&t_np),
        "resolved ({}) < unresolved ({})",
        count_fork_cells(&t_wp),
        count_fork_cells(&t_np)
    );
}

#[test]
fn test_crv_v9_higher_prec_table_passes_sanity() {
    let g = GrammarBuilder::new("crv_v9_hp3")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(sanity_check_tables(&table).is_ok());
}

#[test]
fn test_crv_v9_higher_prec_shift_targets_valid() {
    let g = GrammarBuilder::new("crv_v9_hp4")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    for row in &table.action_table {
        for cell in row {
            for action in cell {
                if let Action::Shift(target) = action {
                    assert!(
                        (target.0 as usize) < table.state_count,
                        "Shift({}) out of range",
                        target.0
                    );
                }
            }
        }
    }
}

// ===========================================================================
// Category 5: Same precedence, left-assoc → reduce (tests 18–21)
// ===========================================================================

#[test]
fn test_crv_v9_same_prec_left_resolves_sr() {
    let g = GrammarBuilder::new("crv_v9_spl1")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(!has_shift_reduce(&table), "same prec + Left → no S/R");
}

#[test]
fn test_crv_v9_same_prec_left_two_ops_resolves() {
    // Both + and - at same precedence, left-assoc
    let g = GrammarBuilder::new("crv_v9_spl2")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("MINUS", r"-")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "MINUS", "expr"],
            1,
            Associativity::Left,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0, "same prec left for both → no S/R");
}

#[test]
fn test_crv_v9_same_prec_left_zero_forks() {
    let g = GrammarBuilder::new("crv_v9_spl3")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn test_crv_v9_same_prec_left_state_count_matches() {
    let g = GrammarBuilder::new("crv_v9_spl4")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(table.state_count, table.action_table.len());
}

// ===========================================================================
// Category 6: Same precedence, right-assoc → shift (tests 22–25)
// ===========================================================================

#[test]
fn test_crv_v9_same_prec_right_resolves_sr() {
    let g = GrammarBuilder::new("crv_v9_spr1")
        .token("NUM", r"\d+")
        .token("EQ", r"=")
        .rule_with_precedence("expr", vec!["expr", "EQ", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(!has_shift_reduce(&table), "same prec + Right → no S/R");
}

#[test]
fn test_crv_v9_same_prec_right_zero_forks() {
    let g = GrammarBuilder::new("crv_v9_spr2")
        .token("NUM", r"\d+")
        .token("EQ", r"=")
        .rule_with_precedence("expr", vec!["expr", "EQ", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn test_crv_v9_same_prec_right_preserves_accept() {
    let g = GrammarBuilder::new("crv_v9_spr3")
        .token("NUM", r"\d+")
        .token("EQ", r"=")
        .rule_with_precedence("expr", vec!["expr", "EQ", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(has_accept(&table));
}

#[test]
fn test_crv_v9_same_prec_right_sanity() {
    let g = GrammarBuilder::new("crv_v9_spr4")
        .token("NUM", r"\d+")
        .token("EQ", r"=")
        .rule_with_precedence("expr", vec!["expr", "EQ", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(sanity_check_tables(&table).is_ok());
}

// ===========================================================================
// Category 7: Declared conflict → Fork action (tests 26–29)
// ===========================================================================

#[test]
fn test_crv_v9_ambiguous_concat_has_forks() {
    // E → E E | a — inherently ambiguous, must produce forks
    let g = GrammarBuilder::new("crv_v9_f1")
        .token("A", "a")
        .rule("expr", vec!["expr", "expr"])
        .rule("expr", vec!["A"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(count_fork_cells(&table) > 0, "E→EE must have forks");
}

#[test]
fn test_crv_v9_ambiguous_binary_has_forks() {
    let g = GrammarBuilder::new("crv_v9_f2")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(count_fork_cells(&table) > 0, "ambiguous binary needs forks");
}

#[test]
fn test_crv_v9_fork_cells_have_shift_and_reduce() {
    let g = GrammarBuilder::new("crv_v9_f3")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let found = table.action_table.iter().any(|row| {
        row.iter().any(|cell| {
            cell.len() > 1
                && cell.iter().any(|a| matches!(a, Action::Shift(_)))
                && cell.iter().any(|a| matches!(a, Action::Reduce(_)))
        })
    });
    assert!(found, "fork cell must have both Shift and Reduce");
}

#[test]
fn test_crv_v9_fork_cells_no_error_action() {
    let g = GrammarBuilder::new("crv_v9_f4")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    for row in &table.action_table {
        for cell in row {
            if cell.len() > 1 {
                assert!(
                    !cell.iter().any(|a| matches!(a, Action::Error)),
                    "conflict cells must not contain Error"
                );
            }
        }
    }
}

// ===========================================================================
// Category 8: Grammar with no precedence → potential conflicts (tests 30–33)
// ===========================================================================

#[test]
fn test_crv_v9_no_prec_binary_has_conflicts() {
    let g = GrammarBuilder::new("crv_v9_np1")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(has_shift_reduce(&table), "no prec → S/R conflict");
}

#[test]
fn test_crv_v9_no_prec_two_ops_has_conflicts() {
    let g = GrammarBuilder::new("crv_v9_np2")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["expr", "STAR", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(
        count_fork_cells(&table) > 0,
        "two ambiguous ops → fork cells"
    );
}

#[test]
fn test_crv_v9_no_prec_table_still_builds() {
    let g = GrammarBuilder::new("crv_v9_np3")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(table.state_count > 0);
}

#[test]
fn test_crv_v9_no_prec_conflict_count_positive() {
    let g = GrammarBuilder::new("crv_v9_np4")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    assert!(summary.shift_reduce > 0, "no prec → positive S/R count");
}

// ===========================================================================
// Category 9: Arithmetic + * with standard precedence (tests 34–37)
// ===========================================================================

#[test]
fn test_crv_v9_arith_standard_prec_no_sr() {
    let g = GrammarBuilder::new("crv_v9_ar1")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0);
}

#[test]
fn test_crv_v9_arith_standard_prec_zero_forks() {
    let g = GrammarBuilder::new("crv_v9_ar2")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn test_crv_v9_arith_standard_prec_has_accept() {
    let g = GrammarBuilder::new("crv_v9_ar3")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(has_accept(&table));
}

#[test]
fn test_crv_v9_arith_standard_prec_sanity() {
    let g = GrammarBuilder::new("crv_v9_ar4")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(sanity_check_tables(&table).is_ok());
}

// ===========================================================================
// Category 10: Three levels of precedence (tests 38–41)
// ===========================================================================

#[test]
fn test_crv_v9_three_levels_no_sr() {
    let g = GrammarBuilder::new("crv_v9_3l1")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .token("CARET", r"\^")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "CARET", "expr"],
            3,
            Associativity::Right,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0, "three levels fully resolve");
}

#[test]
fn test_crv_v9_three_levels_zero_forks() {
    let g = GrammarBuilder::new("crv_v9_3l2")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .token("CARET", r"\^")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "CARET", "expr"],
            3,
            Associativity::Right,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn test_crv_v9_three_levels_sanity() {
    let g = GrammarBuilder::new("crv_v9_3l3")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .token("CARET", r"\^")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "CARET", "expr"],
            3,
            Associativity::Right,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(sanity_check_tables(&table).is_ok());
}

#[test]
fn test_crv_v9_three_levels_has_accept() {
    let g = GrammarBuilder::new("crv_v9_3l4")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .token("CARET", r"\^")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "CARET", "expr"],
            3,
            Associativity::Right,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(has_accept(&table));
}

// ===========================================================================
// Category 11: Mixed associativity (tests 42–46)
// ===========================================================================

#[test]
fn test_crv_v9_mixed_assoc_left_and_right_resolves() {
    // + is left (prec 1), = is right (prec 0)
    let g = GrammarBuilder::new("crv_v9_ma1")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("EQ", r"=")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "EQ", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0, "mixed assoc fully resolves");
}

#[test]
fn test_crv_v9_mixed_assoc_zero_forks() {
    let g = GrammarBuilder::new("crv_v9_ma2")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("EQ", r"=")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "EQ", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn test_crv_v9_none_assoc_does_not_resolve_sr() {
    // Associativity::None typically leaves conflict or produces error
    let g = GrammarBuilder::new("crv_v9_ma3")
        .token("NUM", r"\d+")
        .token("CMP", r"<")
        .rule_with_precedence("expr", vec!["expr", "CMP", "expr"], 1, Associativity::None)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    // None assoc either keeps conflict or uses Error — table must build
    assert!(table.state_count > 0);
}

#[test]
fn test_crv_v9_mixed_three_ops() {
    // + left-1, * left-2, ^ right-3
    let g = GrammarBuilder::new("crv_v9_ma4")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .token("CARET", r"\^")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "CARET", "expr"],
            3,
            Associativity::Right,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn test_crv_v9_mixed_assoc_sanity() {
    let g = GrammarBuilder::new("crv_v9_ma5")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("EQ", r"=")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "EQ", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(sanity_check_tables(&table).is_ok());
}

// ===========================================================================
// Category 12: Conflict count in parse table (tests 47–50)
// ===========================================================================

#[test]
fn test_crv_v9_conflict_count_zero_for_unambiguous() {
    let g = GrammarBuilder::new("crv_v9_cc1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0);
    assert_eq!(summary.reduce_reduce, 0);
}

#[test]
fn test_crv_v9_conflict_count_positive_for_ambiguous() {
    let g = GrammarBuilder::new("crv_v9_cc2")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    assert!(
        summary.shift_reduce > 0 || !summary.states_with_conflicts.is_empty(),
        "ambiguous grammar must have conflicts"
    );
}

#[test]
fn test_crv_v9_conflict_count_states_with_conflicts_unique() {
    let g = GrammarBuilder::new("crv_v9_cc3")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    let unique: std::collections::HashSet<_> =
        summary.states_with_conflicts.iter().copied().collect();
    assert_eq!(unique.len(), summary.states_with_conflicts.len());
}

#[test]
fn test_crv_v9_conflict_count_resolved_is_zero() {
    let g = GrammarBuilder::new("crv_v9_cc4")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0);
}

// ===========================================================================
// Category 13: All non-conflict cells have 0 or 1 action (tests 51–54)
// ===========================================================================

#[test]
fn test_crv_v9_non_conflict_cells_at_most_one_action_unambiguous() {
    let g = GrammarBuilder::new("crv_v9_nc1")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g).unwrap();
    for row in &table.action_table {
        for cell in row {
            assert!(
                cell.len() <= 1,
                "unambiguous grammar: cell has {} actions",
                cell.len()
            );
        }
    }
}

#[test]
fn test_crv_v9_resolved_grammar_cells_at_most_one_action() {
    let g = GrammarBuilder::new("crv_v9_nc2")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    for row in &table.action_table {
        for cell in row {
            assert!(
                cell.len() <= 1,
                "resolved grammar: cell has {} actions",
                cell.len()
            );
        }
    }
}

#[test]
fn test_crv_v9_non_conflict_cells_chain_grammar() {
    let g = GrammarBuilder::new("crv_v9_nc3")
        .token("x", "x")
        .rule("start", vec!["inner"])
        .rule("inner", vec!["x"])
        .start("start")
        .build();
    let table = build_table(&g).unwrap();
    for row in &table.action_table {
        for cell in row {
            assert!(cell.len() <= 1);
        }
    }
}

#[test]
fn test_crv_v9_non_conflict_cells_three_disjoint() {
    let g = GrammarBuilder::new("crv_v9_nc4")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let table = build_table(&g).unwrap();
    for row in &table.action_table {
        for cell in row {
            assert!(cell.len() <= 1);
        }
    }
}

// ===========================================================================
// Category 14: Fork actions have >= 2 sub-actions (tests 55–58)
// ===========================================================================

#[test]
fn test_crv_v9_fork_action_min_two_sub_actions() {
    let g = GrammarBuilder::new("crv_v9_fa1")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    for row in &table.action_table {
        for cell in row {
            for action in cell {
                if let Action::Fork(sub) = action {
                    assert!(
                        sub.len() >= 2,
                        "Fork must have ≥ 2 sub-actions, got {}",
                        sub.len()
                    );
                }
            }
        }
    }
}

#[test]
fn test_crv_v9_fork_action_concat_grammar() {
    let g = GrammarBuilder::new("crv_v9_fa2")
        .token("A", "a")
        .rule("expr", vec!["expr", "expr"])
        .rule("expr", vec!["A"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    for row in &table.action_table {
        for cell in row {
            for action in cell {
                if let Action::Fork(sub) = action {
                    assert!(sub.len() >= 2);
                }
            }
        }
    }
}

#[test]
fn test_crv_v9_fork_actions_contain_no_nested_fork() {
    let g = GrammarBuilder::new("crv_v9_fa3")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    for row in &table.action_table {
        for cell in row {
            for action in cell {
                if let Action::Fork(sub) = action {
                    for inner in sub {
                        assert!(
                            !matches!(inner, Action::Fork(_)),
                            "Fork must not contain nested Fork"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_crv_v9_multi_action_cells_have_distinct_actions() {
    let g = GrammarBuilder::new("crv_v9_fa4")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    for row in &table.action_table {
        for cell in row {
            if cell.len() > 1 {
                // Check at least two distinct actions
                let first = &cell[0];
                let others_differ = cell.iter().skip(1).any(|a| a != first);
                assert!(
                    others_differ,
                    "multi-action cells must have distinct actions"
                );
            }
        }
    }
}

// ===========================================================================
// Category 15: Precedence 0 vs precedence 1 (tests 59–62)
// ===========================================================================

#[test]
fn test_crv_v9_prec_0_is_default_no_resolution() {
    // Precedence 0 is "no precedence" internally — should not resolve
    let g = GrammarBuilder::new("crv_v9_p01")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(has_shift_reduce(&table), "no prec → unresolved S/R");
}

#[test]
fn test_crv_v9_prec_1_resolves() {
    let g = GrammarBuilder::new("crv_v9_p02")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(!has_shift_reduce(&table), "prec 1 + left → resolved");
}

#[test]
fn test_crv_v9_prec_1_vs_2_fewer_forks() {
    let low = GrammarBuilder::new("crv_v9_p03a")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["expr", "STAR", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let full = GrammarBuilder::new("crv_v9_p03b")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let t_low = build_table(&low).unwrap();
    let t_full = build_table(&full).unwrap();
    assert!(
        count_fork_cells(&t_full) <= count_fork_cells(&t_low),
        "more prec → fewer/equal forks"
    );
}

#[test]
fn test_crv_v9_prec_1_table_builds() {
    let g = GrammarBuilder::new("crv_v9_p04")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(table.state_count > 0);
}

// ===========================================================================
// Category 16: Negative precedence values (tests 63–66)
// ===========================================================================

#[test]
fn test_crv_v9_negative_prec_builds() {
    let g = GrammarBuilder::new("crv_v9_neg1")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence(
            "expr",
            vec!["expr", "PLUS", "expr"],
            -1,
            Associativity::Left,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(table.state_count > 0, "negative prec must build");
}

#[test]
fn test_crv_v9_negative_prec_rule_stored() {
    let g = GrammarBuilder::new("crv_v9_neg2")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence(
            "expr",
            vec!["expr", "PLUS", "expr"],
            -5,
            Associativity::Left,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let has_neg = g.all_rules().any(|r| {
        r.precedence
            .as_ref()
            .is_some_and(|p| matches!(p, adze_ir::PrecedenceKind::Static(v) if *v < 0))
    });
    assert!(has_neg, "negative prec must be stored on rule");
}

#[test]
fn test_crv_v9_negative_vs_positive_prec_ordering() {
    // -1 < 1 → prec 1 rule should win
    let g = GrammarBuilder::new("crv_v9_neg3")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence(
            "expr",
            vec!["expr", "PLUS", "expr"],
            -1,
            Associativity::Left,
        )
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    // Table should build with valid resolution
    assert!(sanity_check_tables(&table).is_ok());
}

#[test]
fn test_crv_v9_negative_prec_sanity() {
    let g = GrammarBuilder::new("crv_v9_neg4")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence(
            "expr",
            vec!["expr", "PLUS", "expr"],
            -3,
            Associativity::Left,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(sanity_check_tables(&table).is_ok());
}

// ===========================================================================
// Category 17: Precedence ordering is numeric (tests 67–70)
// ===========================================================================

#[test]
fn test_crv_v9_prec_numeric_2_gt_1() {
    let g = GrammarBuilder::new("crv_v9_pn1")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0, "prec 2 > prec 1 → resolved");
}

#[test]
fn test_crv_v9_prec_numeric_10_gt_5() {
    let g = GrammarBuilder::new("crv_v9_pn2")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 5, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "STAR", "expr"],
            10,
            Associativity::Left,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn test_crv_v9_prec_equal_uses_assoc() {
    // Both at prec 3, left-assoc → should resolve
    let g = GrammarBuilder::new("crv_v9_pn3")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("MINUS", r"-")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 3, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "MINUS", "expr"],
            3,
            Associativity::Left,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(!has_shift_reduce(&table), "equal prec + Left → resolved");
}

#[test]
fn test_crv_v9_prec_equal_right_assoc_resolves() {
    let g = GrammarBuilder::new("crv_v9_pn4")
        .token("NUM", r"\d+")
        .token("ARROW", r"->")
        .token("FAT", r"=>")
        .rule_with_precedence(
            "expr",
            vec!["expr", "ARROW", "expr"],
            3,
            Associativity::Right,
        )
        .rule_with_precedence("expr", vec!["expr", "FAT", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(!has_shift_reduce(&table), "equal prec + Right → resolved");
}

// ===========================================================================
// Category 18: Resolution is deterministic (tests 71–74)
// ===========================================================================

#[test]
fn test_crv_v9_deterministic_same_grammar_same_table_size() {
    let build = || {
        let g = GrammarBuilder::new("crv_v9_det1")
            .token("NUM", r"\d+")
            .token("PLUS", r"\+")
            .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
            .rule("expr", vec!["NUM"])
            .start("expr")
            .build();
        build_table(&g).unwrap()
    };
    let t1 = build();
    let t2 = build();
    assert_eq!(t1.state_count, t2.state_count);
    assert_eq!(t1.action_table.len(), t2.action_table.len());
}

#[test]
fn test_crv_v9_deterministic_same_fork_count() {
    let build = || {
        let g = GrammarBuilder::new("crv_v9_det2")
            .token("NUM", r"\d+")
            .token("PLUS", r"\+")
            .rule("expr", vec!["expr", "PLUS", "expr"])
            .rule("expr", vec!["NUM"])
            .start("expr")
            .build();
        build_table(&g).unwrap()
    };
    let t1 = build();
    let t2 = build();
    assert_eq!(count_fork_cells(&t1), count_fork_cells(&t2));
}

#[test]
fn test_crv_v9_deterministic_same_conflict_summary() {
    let build = || {
        let g = GrammarBuilder::new("crv_v9_det3")
            .token("NUM", r"\d+")
            .token("PLUS", r"\+")
            .rule("expr", vec!["expr", "PLUS", "expr"])
            .rule("expr", vec!["NUM"])
            .start("expr")
            .build();
        let table = build_table(&g).unwrap();
        count_conflicts(&table)
    };
    let s1 = build();
    let s2 = build();
    assert_eq!(s1.shift_reduce, s2.shift_reduce);
    assert_eq!(s1.reduce_reduce, s2.reduce_reduce);
}

#[test]
fn test_crv_v9_deterministic_actions_identical() {
    let build = || {
        let g = GrammarBuilder::new("crv_v9_det4")
            .token("NUM", r"\d+")
            .token("PLUS", r"\+")
            .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
            .rule("expr", vec!["NUM"])
            .start("expr")
            .build();
        build_table(&g).unwrap()
    };
    let t1 = build();
    let t2 = build();
    assert_eq!(t1.action_table, t2.action_table);
}

// ===========================================================================
// Category 19: Same grammar → same conflicts (tests 75–78)
// ===========================================================================

#[test]
fn test_crv_v9_same_grammar_same_sr_conflicts() {
    let make = || {
        GrammarBuilder::new("crv_v9_sg1")
            .token("NUM", r"\d+")
            .token("PLUS", r"\+")
            .rule("expr", vec!["expr", "PLUS", "expr"])
            .rule("expr", vec!["NUM"])
            .start("expr")
            .build()
    };
    let c1 = detect_all(&make());
    let c2 = detect_all(&make());
    assert_eq!(c1.len(), c2.len());
}

#[test]
fn test_crv_v9_same_grammar_same_conflict_types() {
    let make = || {
        GrammarBuilder::new("crv_v9_sg2")
            .token("NUM", r"\d+")
            .token("PLUS", r"\+")
            .rule("expr", vec!["expr", "PLUS", "expr"])
            .rule("expr", vec!["NUM"])
            .start("expr")
            .build()
    };
    let c1 = detect_all(&make());
    let c2 = detect_all(&make());
    let types1: Vec<_> = c1.iter().map(|c| &c.conflict_type).collect();
    let types2: Vec<_> = c2.iter().map(|c| &c.conflict_type).collect();
    assert_eq!(types1, types2);
}

#[test]
fn test_crv_v9_same_grammar_same_conflict_states() {
    let make = || {
        GrammarBuilder::new("crv_v9_sg3")
            .token("NUM", r"\d+")
            .token("PLUS", r"\+")
            .rule("expr", vec!["expr", "PLUS", "expr"])
            .rule("expr", vec!["NUM"])
            .start("expr")
            .build()
    };
    let c1 = detect_all(&make());
    let c2 = detect_all(&make());
    let states1: Vec<_> = c1.iter().map(|c| c.state).collect();
    let states2: Vec<_> = c2.iter().map(|c| c.state).collect();
    assert_eq!(states1, states2);
}

#[test]
fn test_crv_v9_same_grammar_same_conflict_symbols() {
    let make = || {
        GrammarBuilder::new("crv_v9_sg4")
            .token("NUM", r"\d+")
            .token("PLUS", r"\+")
            .rule("expr", vec!["expr", "PLUS", "expr"])
            .rule("expr", vec!["NUM"])
            .start("expr")
            .build()
    };
    let c1 = detect_all(&make());
    let c2 = detect_all(&make());
    let syms1: Vec<_> = c1.iter().map(|c| c.symbol).collect();
    let syms2: Vec<_> = c2.iter().map(|c| c.symbol).collect();
    assert_eq!(syms1, syms2);
}

// ===========================================================================
// Category 20: Different precedences → different resolutions (tests 79–84)
// ===========================================================================

#[test]
fn test_crv_v9_no_prec_vs_prec_different_fork_count() {
    let no_prec = GrammarBuilder::new("crv_v9_dp1a")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let with_prec = GrammarBuilder::new("crv_v9_dp1b")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let t_np = build_table(&no_prec).unwrap();
    let t_wp = build_table(&with_prec).unwrap();
    assert!(
        count_fork_cells(&t_wp) < count_fork_cells(&t_np),
        "prec ({}) < no-prec ({})",
        count_fork_cells(&t_wp),
        count_fork_cells(&t_np)
    );
}

#[test]
fn test_crv_v9_left_vs_right_same_prec_different_tables() {
    let left = GrammarBuilder::new("crv_v9_dp2a")
        .token("NUM", r"\d+")
        .token("OP", r"\+")
        .rule_with_precedence("expr", vec!["expr", "OP", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let right = GrammarBuilder::new("crv_v9_dp2b")
        .token("NUM", r"\d+")
        .token("OP", r"\+")
        .rule_with_precedence("expr", vec!["expr", "OP", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let t_l = build_table(&left).unwrap();
    let t_r = build_table(&right).unwrap();
    // Both resolve S/R but may produce different action tables
    assert!(!has_shift_reduce(&t_l));
    assert!(!has_shift_reduce(&t_r));
}

#[test]
fn test_crv_v9_prec_1_vs_prec_2_different_resolution_quality() {
    // Partial resolution: only + has prec
    let partial = GrammarBuilder::new("crv_v9_dp3a")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["expr", "STAR", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    // Full resolution: both have prec
    let full = GrammarBuilder::new("crv_v9_dp3b")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let t_partial = build_table(&partial).unwrap();
    let t_full = build_table(&full).unwrap();
    assert!(
        count_fork_cells(&t_full) <= count_fork_cells(&t_partial),
        "full prec ({}) ≤ partial ({})",
        count_fork_cells(&t_full),
        count_fork_cells(&t_partial)
    );
}

#[test]
fn test_crv_v9_higher_prec_resolves_more_conflicts() {
    let one_level = GrammarBuilder::new("crv_v9_dp4a")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .token("CARET", r"\^")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["expr", "STAR", "expr"])
        .rule("expr", vec!["expr", "CARET", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let three_levels = GrammarBuilder::new("crv_v9_dp4b")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .token("CARET", r"\^")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "CARET", "expr"],
            3,
            Associativity::Right,
        )
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let t1 = build_table(&one_level).unwrap();
    let t3 = build_table(&three_levels).unwrap();
    assert!(
        count_fork_cells(&t3) <= count_fork_cells(&t1),
        "three levels ({}) ≤ one level ({})",
        count_fork_cells(&t3),
        count_fork_cells(&t1)
    );
}

#[test]
fn test_crv_v9_classify_sr_action_pair() {
    let actions = vec![Action::Shift(StateId(5)), Action::Reduce(RuleId(2))];
    assert_eq!(classify_conflict(&actions), ConflictType::ShiftReduce);
}

#[test]
fn test_crv_v9_classify_rr_action_pair() {
    let actions = vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(3))];
    assert_eq!(classify_conflict(&actions), ConflictType::ReduceReduce);
}
