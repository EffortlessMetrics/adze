//! SRP parse-table fixtures and conflict analysis helpers for GLR BDD tests.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use adze_bdd_grammar_core::{dangling_else_grammar, precedence_arithmetic_grammar};
use adze_glr_core::build_lr1_automaton;
use adze_glr_core::{
    Action, Conflict, ConflictResolver, ConflictType, FirstFollowSets, ParseTable, StateId,
};
use adze_ir::{Associativity, Grammar, RuleId, SymbolId};

/// Summary of conflict information from a parse table.
#[derive(Debug, Clone)]
pub struct ConflictAnalysis {
    /// Number of table cells with more than one parser action.
    pub total_conflicts: usize,
    /// Number of shift/reduce conflicts.
    pub shift_reduce_conflicts: usize,
    /// Number of reduce/reduce conflicts.
    pub reduce_reduce_conflicts: usize,
    /// Per-cell conflict detail for targeted assertions and debug logs.
    pub conflict_details: Vec<(usize, usize, Vec<Action>)>,
}

/// Analyze conflict cells in a parse table.
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
                analysis.total_conflicts += 1;

                let has_shift = actions.iter().any(|a| matches!(a, Action::Shift(_)));
                let has_reduce = actions.iter().any(|a| matches!(a, Action::Reduce(_)));

                if has_shift && has_reduce {
                    analysis.shift_reduce_conflicts += 1;
                } else if !has_shift && has_reduce {
                    analysis.reduce_reduce_conflicts += 1;
                }

                analysis
                    .conflict_details
                    .push((state, sym, actions.clone()));
            }
        }
    }

    analysis
}

/// Count parse table cells with more than one action.
pub fn count_multi_action_cells(parse_table: &ParseTable) -> usize {
    parse_table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .filter(|cell| cell.len() > 1)
        .count()
}

/// Resolve a synthetic shift/reduce conflict against a given grammar and symbol.
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

/// Build an LR(1) parse table from the fixture grammar.
pub fn build_lr1_parse_table(grammar: &Grammar) -> Result<ParseTable, String> {
    let first_follow = FirstFollowSets::compute(grammar).map_err(|err| {
        format!(
            "FAILED to compute FIRST/FOLLOW for fixture grammar {}: {}",
            grammar.name, err
        )
    })?;

    build_lr1_automaton(grammar, &first_follow).map_err(|err| {
        format!(
            "FAILED to build LR(1) automaton for fixture grammar {}: {}",
            grammar.name, err
        )
    })
}

/// Build runtime-ready parse table shape used by Runtime2 BDD tests.
pub fn build_runtime_parse_table(grammar: &Grammar) -> Result<ParseTable, String> {
    build_lr1_parse_table(grammar)
        .map(|table| table.normalize_eof_to_zero().with_detected_goto_indexing())
}

/// Build dangling-else parse table in LR(1) form.
pub fn build_dangling_else_parse_table() -> Result<ParseTable, String> {
    build_lr1_parse_table(&dangling_else_grammar())
}

/// Build dangling-else parse table in Runtime2-ready form.
pub fn build_runtime_dangling_else_parse_table() -> Result<ParseTable, String> {
    build_runtime_parse_table(&dangling_else_grammar())
}

/// Build precedence arithmetic parse table in LR(1) form.
pub fn build_precedence_arithmetic_parse_table(
    plus_assoc: Associativity,
) -> Result<ParseTable, String> {
    build_lr1_parse_table(&precedence_arithmetic_grammar(plus_assoc))
}

/// Build precedence arithmetic parse table in Runtime2-ready form.
pub fn build_runtime_precedence_arithmetic_parse_table(
    plus_assoc: Associativity,
) -> Result<ParseTable, String> {
    build_runtime_parse_table(&precedence_arithmetic_grammar(plus_assoc))
}
