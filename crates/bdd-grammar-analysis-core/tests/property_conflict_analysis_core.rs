use adze_bdd_grammar_analysis_core::{analyze_conflicts, count_multi_action_cells};
use adze_glr_core::{Action, ParseTable, RuleId, StateId};
use proptest::prelude::*;
use std::collections::HashSet;

fn action_strategy() -> impl Strategy<Value = Action> {
    prop_oneof![
        (0u16..=u16::MAX).prop_map(|state| Action::Shift(StateId(state))),
        (0u16..=u16::MAX).prop_map(|rule| Action::Reduce(RuleId(rule))),
        Just(Action::Accept),
        Just(Action::Error),
        Just(Action::Recover),
    ]
}

fn cell_strategy() -> impl Strategy<Value = Vec<Action>> {
    prop::collection::vec(action_strategy(), 0..4)
}

fn table_strategy() -> impl Strategy<Value = ParseTable> {
    (0usize..10, 0usize..10).prop_flat_map(|(rows, cols)| {
        let cell_count = rows * cols;
        prop::collection::vec(cell_strategy(), cell_count).prop_map(move |flat| {
            let mut table = ParseTable::default();
            let mut action_table = vec![vec![vec![]; cols]; rows];

            for (row_idx, row) in action_table.iter_mut().enumerate().take(rows) {
                for (col_idx, cell) in row.iter_mut().enumerate().take(cols) {
                    let offset = row_idx * cols + col_idx;
                    *cell = flat[offset].clone();
                }
            }

            table.state_count = rows;
            table.symbol_count = cols;
            table.action_table = action_table;
            table
        })
    })
}

proptest! {
    #[test]
    fn count_multi_action_cells_tracks_non_singleton_cells(table in table_strategy()) {
        let mut expected_multi_cells = 0usize;
        let mut expected_total_conflicts = 0usize;
        let mut expected_shift_reduce = 0usize;
        let mut expected_reduce_reduce = 0usize;
        let mut expected_coords = HashSet::<(usize, usize)>::new();

        for (state_idx, row) in table.action_table.iter().enumerate() {
            for (symbol_idx, actions) in row.iter().enumerate() {
                if actions.len() > 1 {
                    expected_multi_cells += 1;
                    let has_shift = actions.iter().any(|action| matches!(action, Action::Shift(_)));
                    let has_reduce = actions.iter().any(|action| matches!(action, Action::Reduce(_)));

                    if has_shift && has_reduce {
                        expected_shift_reduce += 1;
                    } else if has_reduce {
                        expected_reduce_reduce += 1;
                    }

                    expected_total_conflicts += 1;
                    expected_coords.insert((state_idx, symbol_idx));
                }
            }
        }

        let analysis = analyze_conflicts(&table);

        assert_eq!(count_multi_action_cells(&table), expected_multi_cells);
        assert_eq!(analysis.total_conflicts, expected_total_conflicts);
        assert_eq!(analysis.shift_reduce_conflicts, expected_shift_reduce);
        assert_eq!(analysis.reduce_reduce_conflicts, expected_reduce_reduce);
        assert_eq!(analysis.conflict_details.len(), expected_total_conflicts);

        let actual_coords = analysis
            .conflict_details
            .iter()
            .map(|(state, symbol, _)| (*state, *symbol))
            .collect::<HashSet<_>>();

        assert_eq!(actual_coords, expected_coords);
    }
}
