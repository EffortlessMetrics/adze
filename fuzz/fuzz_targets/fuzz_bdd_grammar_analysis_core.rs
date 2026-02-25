#![no_main]

use adze_bdd_grammar_analysis_core::{analyze_conflicts, count_multi_action_cells};
use adze_glr_core::{Action, ParseTable, RuleId, StateId};
use libfuzzer_sys::fuzz_target;

fn action_from_byte(byte: u8, idx: u16) -> Action {
    match byte % 5 {
        0 => Action::Error,
        1 => Action::Shift(StateId(idx)),
        2 => Action::Reduce(RuleId(idx)),
        3 => Action::Accept,
        _ => Action::Recover,
    }
}

fn expected_stats(table: &ParseTable) -> (usize, usize, usize, usize) {
    let mut total = 0usize;
    let mut shift_reduce = 0usize;
    let mut reduce_reduce = 0usize;

    for row in &table.action_table {
        for actions in row {
            if actions.len() > 1 {
                total += 1;

                let has_shift = actions
                    .iter()
                    .any(|action| matches!(action, Action::Shift(_)));
                let has_reduce = actions
                    .iter()
                    .any(|action| matches!(action, Action::Reduce(_)));

                if has_shift && has_reduce {
                    shift_reduce += 1;
                } else if has_reduce {
                    reduce_reduce += 1;
                }
            }
        }
    }

    (
        count_multi_action_cells(table),
        total,
        shift_reduce,
        reduce_reduce,
    )
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }

    let rows = (usize::from(data[0]) % 8) + 1;
    let symbols = (usize::from(data[1]) % 8) + 1;

    let mut table = ParseTable::default();
    table.state_count = rows;
    table.symbol_count = symbols;
    table.action_table = vec![vec![vec![]; symbols]; rows];

    for state in 0..rows {
        for symbol in 0..symbols {
            let base = state.checked_mul(symbols).unwrap_or(0) + symbol;
            let max_actions = (data.get(base % data.len()).copied().unwrap_or(0) % 4) as usize;

            for action_idx in 0..max_actions {
                let payload = data.get(base + action_idx + 2).copied().unwrap_or(data[1]);
                let action = action_from_byte(payload, action_idx as u16);
                table.action_table[state][symbol].push(action);
            }
        }
    }

    let (expected_multi, expected_total, expected_shift_reduce, expected_reduce_reduce) =
        expected_stats(&table);
    let analysis = analyze_conflicts(&table);
    let actual_multi = count_multi_action_cells(&table);

    assert_eq!(actual_multi, expected_multi);
    assert_eq!(analysis.total_conflicts, expected_total);
    assert_eq!(analysis.shift_reduce_conflicts, expected_shift_reduce);
    assert_eq!(analysis.reduce_reduce_conflicts, expected_reduce_reduce);
});
