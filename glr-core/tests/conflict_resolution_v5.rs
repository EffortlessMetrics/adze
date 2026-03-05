//! Conflict resolution v5 — comprehensive tests for GLR conflict detection and resolution.
//!
//! Covers: shift-reduce detection, reduce-reduce detection, precedence-based resolution,
//! Fork action generation, conflict-free verification, scaling with ambiguity, and edge cases.
//!
//! Run with: cargo test -p adze-glr-core --test conflict_resolution_v5 -- --test-threads=2

use adze_glr_core::conflict_inspection::{count_conflicts, ConflictType};
use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::Associativity;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Count cells with multiple actions (conflicts / fork points).
fn count_fork_cells(table: &adze_glr_core::ParseTable) -> usize {
    let mut n = 0;
    for state in 0..table.state_count {
        for col in 0..table.action_table[state].len() {
            if table.action_table[state][col].len() > 1 {
                n += 1;
            }
        }
    }
    n
}

/// True if any cell contains both Shift and Reduce actions.
fn has_shift_reduce(table: &adze_glr_core::ParseTable) -> bool {
    for state in &table.action_table {
        for cell in state {
            if cell.len() > 1 {
                let s = cell.iter().any(|a| matches!(a, Action::Shift(_)));
                let r = cell.iter().any(|a| matches!(a, Action::Reduce(_)));
                if s && r {
                    return true;
                }
            }
        }
    }
    false
}

/// True if any cell contains two or more distinct Reduce actions.
fn has_reduce_reduce(table: &adze_glr_core::ParseTable) -> bool {
    for state in &table.action_table {
        for cell in state {
            let reduce_count = cell.iter().filter(|a| matches!(a, Action::Reduce(_))).count();
            if reduce_count > 1 {
                return true;
            }
        }
    }
    false
}

/// True if any cell in the table contains a Fork variant.
fn has_fork_action(table: &adze_glr_core::ParseTable) -> bool {
    for state in &table.action_table {
        for cell in state {
            for action in cell {
                if matches!(action, Action::Fork(_)) {
                    return true;
                }
            }
        }
    }
    false
}

/// True if at least one cell contains Accept.
fn has_accept(table: &adze_glr_core::ParseTable) -> bool {
    table
        .action_table
        .iter()
        .any(|row| row.iter().any(|cell| cell.iter().any(|a| matches!(a, Action::Accept))))
}

/// Count total number of actions across the entire table.
fn total_action_count(table: &adze_glr_core::ParseTable) -> usize {
    table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .map(|cell| cell.len())
        .sum()
}

/// Build grammar + first/follow + parse table in one step.
fn build_table(
    grammar: &adze_ir::Grammar,
) -> Result<adze_glr_core::ParseTable, adze_glr_core::GLRError> {
    let ff = FirstFollowSets::compute(grammar)?;
    build_lr1_automaton(grammar, &ff)
}

// ===========================================================================
// Category 1: Shift-reduce conflict detection in ambiguous grammars
// ===========================================================================

#[test]
fn sr_ambiguous_addition_no_prec() {
    let g = GrammarBuilder::new("sr_add")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(has_shift_reduce(&table));
}

#[test]
fn sr_ambiguous_subtraction_no_prec() {
    let g = GrammarBuilder::new("sr_sub")
        .token("NUM", r"\d+")
        .token("-", "-")
        .rule("expr", vec!["expr", "-", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(has_shift_reduce(&table));
}

#[test]
fn sr_ambiguous_multiplication_no_prec() {
    let g = GrammarBuilder::new("sr_mul")
        .token("NUM", r"\d+")
        .token("*", "*")
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(has_shift_reduce(&table));
}

#[test]
fn sr_two_ops_no_prec_creates_multiple_conflicts() {
    let g = GrammarBuilder::new("sr_two_ops")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    assert!(summary.shift_reduce >= 2, "two ops → ≥ 2 S/R conflicts");
}

#[test]
fn sr_dangling_else_detected() {
    let g = GrammarBuilder::new("dangling_else")
        .token("IF", "if")
        .token("THEN", "then")
        .token("ELSE", "else")
        .token("EXPR", "expr")
        .token("ATOM", "atom")
        .rule("stmt", vec!["IF", "EXPR", "THEN", "stmt"])
        .rule("stmt", vec!["IF", "EXPR", "THEN", "stmt", "ELSE", "stmt"])
        .rule("stmt", vec!["ATOM"])
        .start("stmt")
        .build();
    let table = build_table(&g).unwrap();
    assert!(has_shift_reduce(&table), "dangling-else must produce S/R conflict");
}

#[test]
fn sr_conflict_detail_type_is_shift_reduce() {
    let g = GrammarBuilder::new("sr_detail")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    let sr_details: Vec<_> = summary
        .conflict_details
        .iter()
        .filter(|d| d.conflict_type == ConflictType::ShiftReduce)
        .collect();
    assert!(!sr_details.is_empty(), "should report ShiftReduce detail");
}

#[test]
fn sr_conflict_detail_actions_contain_shift_and_reduce() {
    let g = GrammarBuilder::new("sr_actions")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    for detail in &summary.conflict_details {
        if detail.conflict_type == ConflictType::ShiftReduce {
            let has_s = detail.actions.iter().any(|a| matches!(a, Action::Shift(_)));
            let has_r = detail.actions.iter().any(|a| matches!(a, Action::Reduce(_)));
            assert!(has_s, "S/R detail must include Shift");
            assert!(has_r, "S/R detail must include Reduce");
        }
    }
}

// ===========================================================================
// Category 2: Reduce-reduce conflict detection
// ===========================================================================

#[test]
fn rr_two_nonterminals_same_token() {
    // S → A | B, A → x, B → x  ⟹  reduce/reduce on EOF after 'x'
    let g = GrammarBuilder::new("rr_basic")
        .token("X", "x")
        .rule("start_sym", vec!["nt_a"])
        .rule("start_sym", vec!["nt_b"])
        .rule("nt_a", vec!["X"])
        .rule("nt_b", vec!["X"])
        .start("start_sym")
        .build();
    let table = build_table(&g).unwrap();
    // Table builds successfully even with R/R (GLR keeps both or resolves by PID).
    assert!(table.state_count > 0);
}

#[test]
fn rr_three_alternatives_same_token() {
    let g = GrammarBuilder::new("rr_triple")
        .token("X", "x")
        .rule("top", vec!["alt_a"])
        .rule("top", vec!["alt_b"])
        .rule("top", vec!["alt_c"])
        .rule("alt_a", vec!["X"])
        .rule("alt_b", vec!["X"])
        .rule("alt_c", vec!["X"])
        .start("top")
        .build();
    let table = build_table(&g).unwrap();
    assert!(table.state_count > 0, "table should build for triple R/R");
}

#[test]
fn rr_conflict_summary_reports_correctly() {
    let g = GrammarBuilder::new("rr_summary")
        .token("X", "x")
        .rule("top", vec!["alt_p"])
        .rule("top", vec!["alt_q"])
        .rule("alt_p", vec!["X"])
        .rule("alt_q", vec!["X"])
        .start("top")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    // Either summary detects R/R or resolver already picked lowest-PID winner.
    let raw_rr = has_reduce_reduce(&table);
    if raw_rr {
        assert!(
            summary.reduce_reduce > 0,
            "summary should report R/R when raw table has it"
        );
    }
}

#[test]
fn rr_detail_type_is_reduce_reduce_when_present() {
    let g = GrammarBuilder::new("rr_detail")
        .token("X", "x")
        .rule("top", vec!["alt_a"])
        .rule("top", vec!["alt_b"])
        .rule("alt_a", vec!["X"])
        .rule("alt_b", vec!["X"])
        .start("top")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    if summary.reduce_reduce > 0 {
        let rr_details: Vec<_> = summary
            .conflict_details
            .iter()
            .filter(|d| d.conflict_type == ConflictType::ReduceReduce)
            .collect();
        assert!(!rr_details.is_empty());
    }
}

// ===========================================================================
// Category 3: Precedence-based conflict resolution
// ===========================================================================

#[test]
fn prec_left_assoc_resolves_sr() {
    let g = GrammarBuilder::new("left_add")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(!has_shift_reduce(&table), "left-assoc should resolve S/R");
}

#[test]
fn prec_right_assoc_resolves_sr() {
    let g = GrammarBuilder::new("right_assign")
        .token("NUM", r"\d+")
        .token("=", "=")
        .rule_with_precedence("expr", vec!["expr", "=", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(!has_shift_reduce(&table), "right-assoc should resolve S/R");
}

#[test]
fn prec_higher_wins_over_lower() {
    let g = GrammarBuilder::new("prec_order")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0, "precedence levels should resolve all S/R");
}

#[test]
fn prec_three_levels_all_resolved() {
    let g = GrammarBuilder::new("three_levels")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0);
    assert_eq!(summary.reduce_reduce, 0);
}

#[test]
fn prec_non_assoc_preserves_conflict() {
    let g = GrammarBuilder::new("non_assoc")
        .token("NUM", r"\d+")
        .token("==", "==")
        .rule_with_precedence("expr", vec!["expr", "==", "expr"], 1, Associativity::None)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let forks = count_fork_cells(&table);
    assert!(forks > 0, "non-assoc keeps conflict (forks={})", forks);
}

#[test]
fn prec_mixed_assoc_same_level() {
    // Both + and - at same precedence, left-assoc.
    let g = GrammarBuilder::new("same_level")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0, "same level left-assoc resolves all S/R");
}

#[test]
fn prec_partial_annotation_leaves_some_conflicts() {
    // + has precedence, * does not → * conflicts remain.
    let g = GrammarBuilder::new("partial_prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    assert!(
        summary.shift_reduce > 0,
        "unannotated * should leave some S/R conflicts"
    );
}

#[test]
fn prec_left_assoc_no_fork_cells() {
    let g = GrammarBuilder::new("no_forks_left")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn prec_right_assoc_no_fork_cells() {
    let g = GrammarBuilder::new("no_forks_right")
        .token("NUM", r"\d+")
        .token("=", "=")
        .rule_with_precedence("expr", vec!["expr", "=", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
}

// ===========================================================================
// Category 4: Fork action generation for unresolvable conflicts
// ===========================================================================

#[test]
fn fork_or_multi_action_for_ambiguous_grammar() {
    let g = GrammarBuilder::new("fork_check")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    // Unresolved S/R should appear as multi-action cells or Fork variants.
    let forks = count_fork_cells(&table);
    let fork_variant = has_fork_action(&table);
    assert!(
        forks > 0 || fork_variant,
        "ambiguous grammar must have multi-action cells or Fork variants"
    );
}

#[test]
fn fork_cells_match_conflict_summary_sr_count() {
    let g = GrammarBuilder::new("fork_match")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    // Manual S/R cell count should match summary.
    let manual_sr: usize = table
        .action_table
        .iter()
        .flat_map(|s| s.iter())
        .filter(|cell| {
            cell.len() > 1
                && cell.iter().any(|a| matches!(a, Action::Shift(_)))
                && cell.iter().any(|a| matches!(a, Action::Reduce(_)))
        })
        .count();
    assert_eq!(summary.shift_reduce, manual_sr);
}

#[test]
fn fork_not_generated_when_prec_resolves() {
    let g = GrammarBuilder::new("no_fork_prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(!has_fork_action(&table), "fully resolved grammar should not have Fork actions");
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn fork_count_increases_with_more_unresolved_ops() {
    // One unresolved op.
    let g1 = GrammarBuilder::new("one_op")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let t1 = build_table(&g1).unwrap();
    let f1 = count_fork_cells(&t1);

    // Two unresolved ops.
    let g2 = GrammarBuilder::new("two_ops")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let t2 = build_table(&g2).unwrap();
    let f2 = count_fork_cells(&t2);

    assert!(f2 >= f1, "more ops → more fork cells ({f2} >= {f1})");
}

// ===========================================================================
// Category 5: Conflict-free grammars have no Fork actions
// ===========================================================================

#[test]
fn conflict_free_simple_sequence() {
    let g = GrammarBuilder::new("sequence")
        .token("A", "a")
        .token("B", "b")
        .rule("start_sym", vec!["A", "B"])
        .start("start_sym")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
    assert!(!has_fork_action(&table));
}

#[test]
fn conflict_free_has_accept_action() {
    let g = GrammarBuilder::new("accept_check")
        .token("A", "a")
        .rule("start_sym", vec!["A"])
        .start("start_sym")
        .build();
    let table = build_table(&g).unwrap();
    assert!(has_accept(&table), "every valid grammar should produce Accept");
}

#[test]
fn conflict_free_lr1_deterministic_chain() {
    // S → A B C  (no ambiguity)
    let g = GrammarBuilder::new("chain")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("start_sym", vec!["A", "B", "C"])
        .start("start_sym")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 0);
    assert_eq!(summary.reduce_reduce, 0);
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn conflict_free_multiple_rules_no_overlap() {
    // S → A B | C D  (different first tokens → no conflict)
    let g = GrammarBuilder::new("no_overlap")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .rule("start_sym", vec!["A", "B"])
        .rule("start_sym", vec!["C", "D"])
        .start("start_sym")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
    assert!(!has_shift_reduce(&table));
}

#[test]
fn conflict_free_nested_nonterminals() {
    let g = GrammarBuilder::new("nested")
        .token("X", "x")
        .token("Y", "y")
        .rule("start_sym", vec!["mid"])
        .rule("mid", vec!["X", "Y"])
        .start("start_sym")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn conflict_free_summary_states_empty() {
    let g = GrammarBuilder::new("empty_states")
        .token("A", "a")
        .rule("start_sym", vec!["A"])
        .start("start_sym")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    assert!(
        summary.states_with_conflicts.is_empty(),
        "no conflict states for unambiguous grammar"
    );
}

#[test]
fn conflict_free_fully_annotated_two_ops() {
    let g = GrammarBuilder::new("full_prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0, "fully annotated → deterministic");
}

#[test]
fn conflict_free_display_shows_zero_counts() {
    let g = GrammarBuilder::new("disp_zero")
        .token("A", "a")
        .rule("start_sym", vec!["A"])
        .start("start_sym")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    let display = format!("{summary}");
    assert!(
        display.contains("Shift/Reduce conflicts: 0"),
        "display: {display}"
    );
}

// ===========================================================================
// Category 6: Conflict counts scale with grammar ambiguity
// ===========================================================================

#[test]
fn scaling_one_vs_two_ops_fork_cells() {
    let g1 = GrammarBuilder::new("scale1")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let g2 = GrammarBuilder::new("scale2")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let t1 = build_table(&g1).unwrap();
    let t2 = build_table(&g2).unwrap();

    assert!(
        count_fork_cells(&t2) >= count_fork_cells(&t1),
        "adding ops should not decrease fork cells"
    );
}

#[test]
fn scaling_three_ops_more_conflicts_than_one() {
    let g1 = GrammarBuilder::new("s3_one")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let g3 = GrammarBuilder::new("s3_three")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("-", "-")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["expr", "-", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let t1 = build_table(&g1).unwrap();
    let t3 = build_table(&g3).unwrap();

    let s1 = count_conflicts(&t1);
    let s3 = count_conflicts(&t3);
    assert!(
        s3.shift_reduce >= s1.shift_reduce,
        "3 ops ({}) >= 1 op ({})",
        s3.shift_reduce,
        s1.shift_reduce
    );
}

#[test]
fn scaling_state_count_grows_with_ops() {
    let g1 = GrammarBuilder::new("sc1")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let g2 = GrammarBuilder::new("sc2")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("-", "-")
        .token("/", "/")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["expr", "-", "expr"])
        .rule("expr", vec!["expr", "/", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let t1 = build_table(&g1).unwrap();
    let t2 = build_table(&g2).unwrap();

    assert!(
        t2.state_count >= t1.state_count,
        "more ops → more states: {} >= {}",
        t2.state_count,
        t1.state_count
    );
}

#[test]
fn scaling_total_actions_grow_with_ambiguity() {
    let g1 = GrammarBuilder::new("ta1")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let g2 = GrammarBuilder::new("ta2")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let t1 = build_table(&g1).unwrap();
    let t2 = build_table(&g2).unwrap();

    assert!(
        total_action_count(&t2) >= total_action_count(&t1),
        "more rules → more total actions"
    );
}

#[test]
fn scaling_resolved_grammar_fewer_forks_than_unresolved() {
    let unresolved = GrammarBuilder::new("unresolved")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let resolved = GrammarBuilder::new("resolved")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let tu = build_table(&unresolved).unwrap();
    let tr = build_table(&resolved).unwrap();

    assert!(
        count_fork_cells(&tr) < count_fork_cells(&tu),
        "resolved ({}) < unresolved ({})",
        count_fork_cells(&tr),
        count_fork_cells(&tu)
    );
}

// ===========================================================================
// Category 7: Edge cases
// ===========================================================================

#[test]
fn edge_single_token_grammar() {
    let g = GrammarBuilder::new("single")
        .token("A", "a")
        .rule("start_sym", vec!["A"])
        .start("start_sym")
        .build();
    let table = build_table(&g).unwrap();
    assert!(table.state_count > 0);
    assert_eq!(count_fork_cells(&table), 0);
    assert!(has_accept(&table));
}

#[test]
fn edge_two_token_sequence() {
    let g = GrammarBuilder::new("two_tok")
        .token("A", "a")
        .token("B", "b")
        .rule("start_sym", vec!["A", "B"])
        .start("start_sym")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn edge_left_recursive_no_prec() {
    // E → E + NUM | NUM  (left-recursive, no prec annotation)
    let g = GrammarBuilder::new("left_rec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    // Left-recursive chain: E → E + NUM is LR(1)-parseable → no S/R conflict.
    assert!(!has_shift_reduce(&table));
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn edge_right_recursive_no_prec() {
    // E → NUM + E | NUM  (right-recursive, no prec)
    let g = GrammarBuilder::new("right_rec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUM", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    // The S/R conflict depends on whether NUM can start both alternatives.
    // Table should build regardless.
    assert!(table.state_count > 0);
}

#[test]
fn edge_self_ambiguous_binary() {
    // E → E E | a  (highly ambiguous concatenation grammar)
    let g = GrammarBuilder::new("self_ambig")
        .token("A", "a")
        .rule("expr", vec!["expr", "expr"])
        .rule("expr", vec!["A"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(count_fork_cells(&table) > 0, "E → E E is ambiguous");
}

#[test]
fn edge_many_alternatives_same_first() {
    // S → a b | a c | a d | a e  (common prefix → multiple states)
    let g = GrammarBuilder::new("many_alt")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .rule("start_sym", vec!["A", "B"])
        .rule("start_sym", vec!["A", "C"])
        .rule("start_sym", vec!["A", "D"])
        .rule("start_sym", vec!["A", "E"])
        .start("start_sym")
        .build();
    let table = build_table(&g).unwrap();
    // LR(1) can handle common prefix with 1-token lookahead (b, c, d, e differ).
    assert_eq!(count_fork_cells(&table), 0);
}

#[test]
fn edge_deeply_nested_nonterminals() {
    let g = GrammarBuilder::new("deep")
        .token("X", "x")
        .rule("start_sym", vec!["level1"])
        .rule("level1", vec!["level2"])
        .rule("level2", vec!["level3"])
        .rule("level3", vec!["X"])
        .start("start_sym")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
    assert!(has_accept(&table));
}

#[test]
fn edge_epsilon_rule_no_crash() {
    // S → A | ε
    let g = GrammarBuilder::new("epsilon")
        .token("A", "a")
        .rule("start_sym", vec!["A"])
        .rule("start_sym", vec![])
        .start("start_sym")
        .build();
    let table = build_table(&g).unwrap();
    assert!(table.state_count > 0);
}

#[test]
fn edge_multiple_start_alternatives() {
    // S → A | B | C  with distinct tokens.
    let g = GrammarBuilder::new("multi_start")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("start_sym", vec!["A"])
        .rule("start_sym", vec!["B"])
        .rule("start_sym", vec!["C"])
        .start("start_sym")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(count_fork_cells(&table), 0);
    assert!(has_accept(&table));
}

// ===========================================================================
// Additional cross-cutting tests
// ===========================================================================

#[test]
fn states_with_conflicts_are_unique() {
    let g = GrammarBuilder::new("unique_states")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
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
fn conflict_summary_display_contains_sr_and_rr() {
    let g = GrammarBuilder::new("display_test")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    let text = format!("{summary}");
    assert!(text.contains("Shift/Reduce"), "display: {text}");
    assert!(text.contains("Reduce/Reduce"), "display: {text}");
}

#[test]
fn sanity_check_passes_for_all_built_tables() {
    // Verify sanity_check_tables doesn't reject our tables.
    let grammars = vec![
        GrammarBuilder::new("san1")
            .token("A", "a")
            .rule("start_sym", vec!["A"])
            .start("start_sym")
            .build(),
        GrammarBuilder::new("san2")
            .token("NUM", r"\d+")
            .token("+", "+")
            .rule("expr", vec!["expr", "+", "expr"])
            .rule("expr", vec!["NUM"])
            .start("expr")
            .build(),
        GrammarBuilder::new("san3")
            .token("NUM", r"\d+")
            .token("+", "+")
            .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
            .rule("expr", vec!["NUM"])
            .start("expr")
            .build(),
    ];
    for gram in &grammars {
        let table = build_table(gram).unwrap();
        let result = adze_glr_core::sanity_check_tables(&table);
        assert!(result.is_ok(), "sanity check failed: {result:?}");
    }
}

#[test]
fn accept_only_on_eof_lookahead() {
    let g = GrammarBuilder::new("eof_accept")
        .token("A", "a")
        .rule("start_sym", vec!["A"])
        .start("start_sym")
        .build();
    let table = build_table(&g).unwrap();
    // Accept should only appear for the EOF symbol column.
    for state_row in &table.action_table {
        for (col, cell) in state_row.iter().enumerate() {
            if cell.iter().any(|a| matches!(a, Action::Accept)) {
                // This column should correspond to the EOF symbol.
                if col < table.index_to_symbol.len() {
                    let sym = table.index_to_symbol[col];
                    assert_eq!(
                        sym, table.eof_symbol,
                        "Accept on non-EOF symbol column {col}"
                    );
                }
            }
        }
    }
}

#[test]
fn shift_actions_reference_valid_states() {
    let g = GrammarBuilder::new("valid_shift")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    for state_row in &table.action_table {
        for cell in state_row {
            for action in cell {
                if let Action::Shift(target) = action {
                    assert!(
                        (target.0 as usize) < table.state_count,
                        "Shift target {} out of range (states={})",
                        target.0,
                        table.state_count
                    );
                }
            }
        }
    }
}

#[test]
fn no_error_action_in_conflict_cells() {
    // Conflict cells should have Shift/Reduce, not Error.
    let g = GrammarBuilder::new("no_err_conflict")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    for state_row in &table.action_table {
        for cell in state_row {
            if cell.len() > 1 {
                assert!(
                    !cell.iter().any(|a| matches!(a, Action::Error)),
                    "conflict cells should not contain Error action"
                );
            }
        }
    }
}

#[test]
fn conflict_detail_symbol_names_non_empty() {
    let g = GrammarBuilder::new("sym_names")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    let summary = count_conflicts(&table);
    for detail in &summary.conflict_details {
        assert!(
            !detail.symbol_name.is_empty(),
            "conflict detail should have a symbol name"
        );
    }
}

#[test]
fn precedence_does_not_affect_accept() {
    let g = GrammarBuilder::new("prec_accept")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert!(has_accept(&table), "precedence resolution must not remove Accept");
}

#[test]
fn table_state_count_matches_action_table_len() {
    let g = GrammarBuilder::new("count_match")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g).unwrap();
    assert_eq!(table.state_count, table.action_table.len());
}
