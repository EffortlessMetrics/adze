//! Conflict-analysis helpers for GLR BDD parse-table tests.
//!
//! This crate owns parse-table conflict inspection so fixture crates can keep
//! grammar construction and conflict metrics responsibilities separated.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use adze_glr_core::{
    Action, Conflict, ConflictResolver, ConflictType, Grammar, ParseTable, RuleId, StateId,
    SymbolId,
};

/// Summary of conflict information from a parse table.
#[derive(Debug, Clone)]
pub struct ConflictAnalysis {
    /// Number of table cells with more than one action.
    pub total_conflicts: usize,
    /// Number of shift/reduce conflicts.
    pub shift_reduce_conflicts: usize,
    /// Number of reduce/reduce conflicts.
    pub reduce_reduce_conflicts: usize,
    /// Per-cell conflict detail for targeted assertions and debug logs.
    pub conflict_details: Vec<(usize, usize, Vec<Action>)>,
}

/// Count parse-table cells where more than one action is present.
pub fn count_multi_action_cells(parse_table: &ParseTable) -> usize {
    parse_table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .filter(|cell| cell.len() > 1)
        .count()
}

/// Analyze conflicts in a parse table and return aggregate classification counts.
pub fn analyze_conflicts(parse_table: &ParseTable) -> ConflictAnalysis {
    let mut analysis = ConflictAnalysis {
        total_conflicts: 0,
        shift_reduce_conflicts: 0,
        reduce_reduce_conflicts: 0,
        conflict_details: vec![],
    };

    for state in 0..parse_table.state_count {
        for sym in 0..parse_table.symbol_count {
            let actions = &parse_table.action_table[state][sym];
            if actions.len() > 1 {
                let has_shift = actions.iter().any(|a| matches!(a, Action::Shift(_)));
                let has_reduce = actions.iter().any(|a| matches!(a, Action::Reduce(_)));

                if has_shift && has_reduce {
                    analysis.shift_reduce_conflicts += 1;
                } else if !has_shift && has_reduce {
                    analysis.reduce_reduce_conflicts += 1;
                }

                analysis.total_conflicts += 1;
                analysis
                    .conflict_details
                    .push((state, sym, actions.clone()));
            }
        }
    }

    analysis
}

/// Resolve a synthetic shift/reduce conflict against precedence metadata.
pub fn resolve_shift_reduce_actions(
    grammar: &Grammar,
    symbol: SymbolId,
    reduce_rule: RuleId,
) -> Vec<Action> {
    let mut resolver = ConflictResolver {
        conflicts: vec![Conflict {
            state: StateId(42),
            symbol,
            actions: vec![Action::Shift(StateId(7)), Action::Reduce(reduce_rule)],
            conflict_type: ConflictType::ShiftReduce,
        }],
    };

    resolver.resolve_conflicts(grammar);
    resolver
        .conflicts
        .first()
        .expect("expected one conflict")
        .actions
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_table(
        action_table: Vec<Vec<Vec<Action>>>,
        states: usize,
        symbols: usize,
    ) -> ParseTable {
        ParseTable {
            action_table,
            state_count: states,
            symbol_count: symbols,
            ..Default::default()
        }
    }

    #[test]
    fn empty_table_has_no_conflicts() {
        let pt = ParseTable::default();
        let analysis = analyze_conflicts(&pt);
        assert_eq!(analysis.total_conflicts, 0);
        assert_eq!(analysis.shift_reduce_conflicts, 0);
        assert_eq!(analysis.reduce_reduce_conflicts, 0);
        assert!(analysis.conflict_details.is_empty());
    }

    #[test]
    fn single_action_cells_have_no_conflicts() {
        let pt = make_table(
            vec![vec![
                vec![Action::Shift(StateId(1))],
                vec![Action::Reduce(RuleId(0))],
            ]],
            1,
            2,
        );
        assert_eq!(count_multi_action_cells(&pt), 0);
        assert_eq!(analyze_conflicts(&pt).total_conflicts, 0);
    }

    #[test]
    fn shift_reduce_conflict_detected() {
        let pt = make_table(
            vec![vec![vec![
                Action::Shift(StateId(1)),
                Action::Reduce(RuleId(0)),
            ]]],
            1,
            1,
        );
        let analysis = analyze_conflicts(&pt);
        assert_eq!(analysis.total_conflicts, 1);
        assert_eq!(analysis.shift_reduce_conflicts, 1);
        assert_eq!(analysis.reduce_reduce_conflicts, 0);
    }

    #[test]
    fn reduce_reduce_conflict_detected() {
        let pt = make_table(
            vec![vec![vec![
                Action::Reduce(RuleId(0)),
                Action::Reduce(RuleId(1)),
            ]]],
            1,
            1,
        );
        let analysis = analyze_conflicts(&pt);
        assert_eq!(analysis.total_conflicts, 1);
        assert_eq!(analysis.shift_reduce_conflicts, 0);
        assert_eq!(analysis.reduce_reduce_conflicts, 1);
    }

    #[test]
    fn count_multi_action_cells_matches_conflicts() {
        let pt = make_table(
            vec![vec![
                vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
                vec![Action::Accept],
            ]],
            1,
            2,
        );
        assert_eq!(count_multi_action_cells(&pt), 1);
    }

    #[test]
    fn conflict_analysis_debug_format() {
        let analysis = ConflictAnalysis {
            total_conflicts: 0,
            shift_reduce_conflicts: 0,
            reduce_reduce_conflicts: 0,
            conflict_details: vec![],
        };
        let dbg = format!("{analysis:?}");
        assert!(dbg.contains("total_conflicts"));
    }
}
