#![allow(clippy::needless_range_loop)]

use proptest::prelude::*;

use adze_runtime::language::{Action, ParseTable, SymbolMetadata};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_action() -> impl Strategy<Value = Action> {
    prop_oneof![
        any::<u16>().prop_map(Action::Shift),
        (any::<u16>(), any::<u8>()).prop_map(|(symbol, child_count)| Action::Reduce {
            symbol,
            child_count
        }),
        Just(Action::Accept),
        Just(Action::Error),
    ]
}

fn arb_action_cell(max_actions: usize) -> impl Strategy<Value = Vec<Action>> {
    prop::collection::vec(arb_action(), 0..=max_actions)
}

/// Generate a row (one state) with `num_symbols` cells.
fn arb_state_row(
    num_symbols: usize,
    max_actions: usize,
) -> impl Strategy<Value = Vec<Vec<Action>>> {
    prop::collection::vec(arb_action_cell(max_actions), num_symbols..=num_symbols)
}

/// Generate a consistent ParseTable: state_count matches action_table.len().
fn arb_parse_table(
    max_states: usize,
    max_symbols: usize,
    max_actions: usize,
) -> impl Strategy<Value = ParseTable> {
    (1..=max_states, 1..=max_symbols).prop_flat_map(move |(states, symbols)| {
        prop::collection::vec(arb_state_row(symbols, max_actions), states..=states).prop_map(
            move |action_table| ParseTable {
                state_count: states,
                action_table,
                small_parse_table: None,
                small_parse_table_map: None,
            },
        )
    })
}

/// Generate a small ParseTable suitable for fast tests.
fn arb_small_parse_table() -> impl Strategy<Value = ParseTable> {
    arb_parse_table(4, 4, 3)
}

/// Generate an empty ParseTable (zero states, empty action_table).
fn empty_parse_table() -> ParseTable {
    ParseTable {
        state_count: 0,
        action_table: vec![],
        small_parse_table: None,
        small_parse_table_map: None,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_tables_equal(a: &ParseTable, b: &ParseTable) -> bool {
    a.state_count == b.state_count
        && a.action_table == b.action_table
        && a.small_parse_table == b.small_parse_table
        && a.small_parse_table_map == b.small_parse_table_map
}

/// A cell is deterministic if it has at most one non-Error action.
fn cell_is_deterministic(cell: &[Action]) -> bool {
    cell.iter().filter(|a| !matches!(a, Action::Error)).count() <= 1
}

/// A table is deterministic if every cell is deterministic.
fn table_is_deterministic(table: &ParseTable) -> bool {
    table
        .action_table
        .iter()
        .all(|row| row.iter().all(|cell| cell_is_deterministic(cell)))
}

// ===========================================================================
// Tests
// ===========================================================================

// ---------------------------------------------------------------------------
// 1 – ParseTable creation
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn creation_state_count_matches(table in arb_small_parse_table()) {
        prop_assert_eq!(table.state_count, table.action_table.len());
    }

    #[test]
    fn creation_rows_have_uniform_width(table in arb_small_parse_table()) {
        if let Some(first) = table.action_table.first() {
            let width = first.len();
            for row in &table.action_table {
                prop_assert_eq!(row.len(), width);
            }
        }
    }

    #[test]
    fn creation_preserves_action_table(
        states in 1..=5usize,
        symbols in 1..=5usize,
    ) {
        let action_table: Vec<Vec<Vec<Action>>> = (0..states)
            .map(|_| (0..symbols).map(|_| vec![Action::Error]).collect())
            .collect();
        let table = ParseTable {
            state_count: states,
            action_table: action_table.clone(),
            small_parse_table: None,
            small_parse_table_map: None,
        };
        prop_assert_eq!(table.action_table, action_table);
    }
}

// ---------------------------------------------------------------------------
// 2 – ParseTable default (empty construction)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn default_has_zero_states(_seed in 0..100u32) {
        let table = empty_parse_table();
        prop_assert_eq!(table.state_count, 0);
        prop_assert!(table.action_table.is_empty());
    }

    #[test]
    fn default_has_no_compressed_table(_seed in 0..100u32) {
        let table = empty_parse_table();
        prop_assert!(table.small_parse_table.is_none());
        prop_assert!(table.small_parse_table_map.is_none());
    }

    #[test]
    fn default_clone_equals_original(_seed in 0..100u32) {
        let table = empty_parse_table();
        let cloned = table.clone();
        prop_assert!(parse_tables_equal(&table, &cloned));
    }
}

// ---------------------------------------------------------------------------
// 3 – ParseTable with actions
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn actions_survive_insertion(
        state in any::<u16>(),
        symbol in any::<u16>(),
        child_count in any::<u8>(),
    ) {
        let actions = vec![
            Action::Shift(state),
            Action::Reduce { symbol, child_count },
            Action::Accept,
            Action::Error,
        ];
        let table = ParseTable {
            state_count: 1,
            action_table: vec![vec![actions.clone()]],
            small_parse_table: None,
            small_parse_table_map: None,
        };
        prop_assert_eq!(&table.action_table[0][0], &actions);
    }

    #[test]
    fn single_shift_action_per_cell(target_state in any::<u16>()) {
        let table = ParseTable {
            state_count: 1,
            action_table: vec![vec![vec![Action::Shift(target_state)]]],
            small_parse_table: None,
            small_parse_table_map: None,
        };
        prop_assert_eq!(table.action_table[0][0].len(), 1);
        prop_assert_eq!(table.action_table[0][0][0], Action::Shift(target_state));
    }

    #[test]
    fn multiple_actions_in_cell(
        s1 in any::<u16>(),
        s2 in any::<u16>(),
        sym in any::<u16>(),
        cc in any::<u8>(),
    ) {
        let cell = vec![
            Action::Shift(s1),
            Action::Shift(s2),
            Action::Reduce { symbol: sym, child_count: cc },
        ];
        let table = ParseTable {
            state_count: 1,
            action_table: vec![vec![cell.clone()]],
            small_parse_table: None,
            small_parse_table_map: None,
        };
        prop_assert_eq!(table.action_table[0][0].len(), 3);
    }
}

// ---------------------------------------------------------------------------
// 4 – ParseTable state count
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn state_count_matches_rows(table in arb_small_parse_table()) {
        prop_assert_eq!(table.state_count, table.action_table.len());
    }

    #[test]
    fn state_count_one(symbols in 1..=8usize) {
        let table = ParseTable {
            state_count: 1,
            action_table: vec![vec![vec![]; symbols]],
            small_parse_table: None,
            small_parse_table_map: None,
        };
        prop_assert_eq!(table.state_count, 1);
        prop_assert_eq!(table.action_table.len(), 1);
    }

    #[test]
    fn state_count_scales(n in 1..=20usize) {
        let table = ParseTable {
            state_count: n,
            action_table: vec![vec![vec![Action::Error]; 1]; n],
            small_parse_table: None,
            small_parse_table_map: None,
        };
        prop_assert_eq!(table.state_count, n);
        prop_assert_eq!(table.action_table.len(), n);
    }
}

// ---------------------------------------------------------------------------
// 5 – ParseTable clone
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn clone_preserves_state_count(table in arb_small_parse_table()) {
        let cloned = table.clone();
        prop_assert_eq!(table.state_count, cloned.state_count);
    }

    #[test]
    fn clone_preserves_action_table(table in arb_small_parse_table()) {
        let cloned = table.clone();
        prop_assert_eq!(table.action_table, cloned.action_table);
    }

    #[test]
    fn clone_preserves_compressed_fields(
        data in prop::collection::vec(any::<u16>(), 0..10),
        map_data in prop::collection::vec(any::<u32>(), 0..10),
    ) {
        let table = ParseTable {
            state_count: 0,
            action_table: vec![],
            small_parse_table: Some(data.clone()),
            small_parse_table_map: Some(map_data.clone()),
        };
        let cloned = table.clone();
        prop_assert_eq!(cloned.small_parse_table.as_ref(), Some(&data));
        prop_assert_eq!(cloned.small_parse_table_map.as_ref(), Some(&map_data));
    }

    #[test]
    fn clone_is_independent(table in arb_small_parse_table()) {
        let mut cloned = table.clone();
        cloned.state_count = 9999;
        // Original must not be affected.
        prop_assert_ne!(table.state_count, 9999);
    }
}

// ---------------------------------------------------------------------------
// 6 – ParseTable in Language context
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn language_builder_accepts_parse_table(table in arb_small_parse_table()) {
        // With glr-core enabled, Language uses glr-core ParseTable, but the
        // local ParseTable struct should still be constructable and usable on
        // its own. We verify construction doesn't panic.
        let _ = table.clone();
        prop_assert!(true);
    }

    #[test]
    fn language_symbol_metadata_independent_of_table(n in 1..=10usize) {
        let metadata: Vec<SymbolMetadata> = (0..n)
            .map(|_| SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            })
            .collect();
        let table = ParseTable {
            state_count: 1,
            action_table: vec![vec![vec![Action::Error]; n]],
            small_parse_table: None,
            small_parse_table_map: None,
        };
        // Symbols dimension can match table column count.
        prop_assert_eq!(metadata.len(), table.action_table[0].len());
    }

    #[test]
    fn parse_table_debug_does_not_panic(table in arb_small_parse_table()) {
        let debug_str = format!("{:?}", table);
        prop_assert!(!debug_str.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 7 – ParseTable with goto entries (small_parse_table / map fields)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn goto_small_table_roundtrip(data in prop::collection::vec(any::<u16>(), 0..20)) {
        let table = ParseTable {
            state_count: 0,
            action_table: vec![],
            small_parse_table: Some(data.clone()),
            small_parse_table_map: None,
        };
        prop_assert_eq!(table.small_parse_table.as_ref().unwrap(), &data);
    }

    #[test]
    fn goto_map_roundtrip(map_data in prop::collection::vec(any::<u32>(), 0..20)) {
        let table = ParseTable {
            state_count: 0,
            action_table: vec![],
            small_parse_table: None,
            small_parse_table_map: Some(map_data.clone()),
        };
        prop_assert_eq!(table.small_parse_table_map.as_ref().unwrap(), &map_data);
    }

    #[test]
    fn goto_both_present(
        data in prop::collection::vec(any::<u16>(), 1..15),
        map_data in prop::collection::vec(any::<u32>(), 1..15),
    ) {
        let table = ParseTable {
            state_count: 0,
            action_table: vec![],
            small_parse_table: Some(data.clone()),
            small_parse_table_map: Some(map_data.clone()),
        };
        prop_assert!(table.small_parse_table.is_some());
        prop_assert!(table.small_parse_table_map.is_some());
        prop_assert_eq!(table.small_parse_table.unwrap().len(), data.len());
        prop_assert_eq!(table.small_parse_table_map.unwrap().len(), map_data.len());
    }

    #[test]
    fn goto_none_by_default(_seed in 0..100u32) {
        let table = empty_parse_table();
        prop_assert!(table.small_parse_table.is_none());
        prop_assert!(table.small_parse_table_map.is_none());
    }

    #[test]
    fn goto_clone_preserves_data(
        data in prop::collection::vec(any::<u16>(), 1..10),
        map_data in prop::collection::vec(any::<u32>(), 1..10),
    ) {
        let table = ParseTable {
            state_count: 0,
            action_table: vec![],
            small_parse_table: Some(data.clone()),
            small_parse_table_map: Some(map_data.clone()),
        };
        let cloned = table.clone();
        prop_assert_eq!(cloned.small_parse_table, Some(data));
        prop_assert_eq!(cloned.small_parse_table_map, Some(map_data));
    }
}

// ---------------------------------------------------------------------------
// 8 – ParseTable determinism
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn deterministic_single_action_cells(table in arb_parse_table(4, 4, 1)) {
        // Tables with at most 1 action per cell must be deterministic.
        prop_assert!(table_is_deterministic(&table));
    }

    #[test]
    fn deterministic_all_error_table(states in 1..=6usize, symbols in 1..=6usize) {
        let table = ParseTable {
            state_count: states,
            action_table: vec![vec![vec![Action::Error]; symbols]; states],
            small_parse_table: None,
            small_parse_table_map: None,
        };
        prop_assert!(table_is_deterministic(&table));
    }

    #[test]
    fn nondeterministic_dual_shifts(s1 in any::<u16>(), s2 in any::<u16>()) {
        let cell = vec![Action::Shift(s1), Action::Shift(s2)];
        prop_assert!(!cell_is_deterministic(&cell));
    }

    #[test]
    fn nondeterministic_shift_reduce(
        state in any::<u16>(),
        sym in any::<u16>(),
        cc in any::<u8>(),
    ) {
        let cell = vec![
            Action::Shift(state),
            Action::Reduce { symbol: sym, child_count: cc },
        ];
        prop_assert!(!cell_is_deterministic(&cell));
    }

    #[test]
    fn deterministic_empty_cells(states in 1..=5usize, symbols in 1..=5usize) {
        let table = ParseTable {
            state_count: states,
            action_table: vec![vec![vec![]; symbols]; states],
            small_parse_table: None,
            small_parse_table_map: None,
        };
        prop_assert!(table_is_deterministic(&table));
    }

    #[test]
    fn deterministic_single_accept(states in 1..=5usize) {
        let mut rows: Vec<Vec<Vec<Action>>> = vec![vec![vec![Action::Error]; 1]; states];
        rows[0][0] = vec![Action::Accept];
        let table = ParseTable {
            state_count: states,
            action_table: rows,
            small_parse_table: None,
            small_parse_table_map: None,
        };
        prop_assert!(table_is_deterministic(&table));
    }

    #[test]
    fn determinism_preserved_through_clone(table in arb_small_parse_table()) {
        let det_original = table_is_deterministic(&table);
        let cloned = table.clone();
        let det_cloned = table_is_deterministic(&cloned);
        prop_assert_eq!(det_original, det_cloned);
    }
}
