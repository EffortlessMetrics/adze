#![allow(clippy::needless_range_loop)]

use proptest::prelude::*;

use adze_runtime::language::{Action, ParseTable};

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

fn arb_action_vec(max_len: usize) -> impl Strategy<Value = Vec<Action>> {
    prop::collection::vec(arb_action(), 0..=max_len)
}

// ===========================================================================
// 1 – Shift creation
// ===========================================================================

proptest! {
    #[test]
    fn shift_preserves_state(state in any::<u16>()) {
        let a = Action::Shift(state);
        match a {
            Action::Shift(s) => prop_assert_eq!(s, state),
            _ => prop_assert!(false, "expected Shift variant"),
        }
    }

    #[test]
    fn shift_zero_state(_ in 0..1u8) {
        let a = Action::Shift(0);
        prop_assert_eq!(a, Action::Shift(0));
    }

    #[test]
    fn shift_max_state(_ in 0..1u8) {
        let a = Action::Shift(u16::MAX);
        prop_assert_eq!(a, Action::Shift(u16::MAX));
    }

    #[test]
    fn shift_not_accept(state in any::<u16>()) {
        prop_assert_ne!(Action::Shift(state), Action::Accept);
    }

    #[test]
    fn shift_not_error(state in any::<u16>()) {
        prop_assert_ne!(Action::Shift(state), Action::Error);
    }
}

// ===========================================================================
// 2 – Reduce creation
// ===========================================================================

proptest! {
    #[test]
    fn reduce_preserves_fields(symbol in any::<u16>(), child_count in any::<u8>()) {
        let a = Action::Reduce { symbol, child_count };
        match a {
            Action::Reduce { symbol: s, child_count: c } => {
                prop_assert_eq!(s, symbol);
                prop_assert_eq!(c, child_count);
            }
            _ => prop_assert!(false, "expected Reduce variant"),
        }
    }

    #[test]
    fn reduce_zero_children(symbol in any::<u16>()) {
        let a = Action::Reduce { symbol, child_count: 0 };
        if let Action::Reduce { child_count, .. } = a {
            prop_assert_eq!(child_count, 0);
        }
    }

    #[test]
    fn reduce_max_children(symbol in any::<u16>()) {
        let a = Action::Reduce { symbol, child_count: u8::MAX };
        if let Action::Reduce { child_count, .. } = a {
            prop_assert_eq!(child_count, u8::MAX);
        }
    }

    #[test]
    fn reduce_not_accept(symbol in any::<u16>(), cc in any::<u8>()) {
        prop_assert_ne!(Action::Reduce { symbol, child_count: cc }, Action::Accept);
    }

    #[test]
    fn reduce_not_error(symbol in any::<u16>(), cc in any::<u8>()) {
        prop_assert_ne!(Action::Reduce { symbol, child_count: cc }, Action::Error);
    }
}

// ===========================================================================
// 3 – Accept
// ===========================================================================

proptest! {
    #[test]
    fn accept_equals_accept(_ in 0..1u8) {
        prop_assert_eq!(Action::Accept, Action::Accept);
    }

    #[test]
    fn accept_ne_any_shift(state in any::<u16>()) {
        prop_assert_ne!(Action::Accept, Action::Shift(state));
    }

    #[test]
    fn accept_ne_any_reduce(sym in any::<u16>(), cc in any::<u8>()) {
        prop_assert_ne!(Action::Accept, Action::Reduce { symbol: sym, child_count: cc });
    }
}

// ===========================================================================
// 4 – Error
// ===========================================================================

proptest! {
    #[test]
    fn error_equals_error(_ in 0..1u8) {
        prop_assert_eq!(Action::Error, Action::Error);
    }

    #[test]
    fn error_ne_accept(_ in 0..1u8) {
        prop_assert_ne!(Action::Error, Action::Accept);
    }

    #[test]
    fn error_ne_any_shift(state in any::<u16>()) {
        prop_assert_ne!(Action::Error, Action::Shift(state));
    }
}

// ===========================================================================
// 5 – Clone / Copy
// ===========================================================================

proptest! {
    #[test]
    fn clone_identical(a in arb_action()) {
        let cloned = a;
        prop_assert_eq!(a, cloned);
    }

    #[test]
    fn copy_identical(a in arb_action()) {
        let copied = a;
        prop_assert_eq!(a, copied);
    }

    #[test]
    fn clone_vec_preserves_all(actions in arb_action_vec(64)) {
        let cloned = actions.clone();
        prop_assert_eq!(actions.len(), cloned.len());
        for i in 0..actions.len() {
            prop_assert_eq!(actions[i], cloned[i]);
        }
    }
}

// ===========================================================================
// 6 – Equality
// ===========================================================================

proptest! {
    #[test]
    fn eq_is_reflexive(a in arb_action()) {
        prop_assert!(a == a);
    }

    #[test]
    fn eq_is_symmetric(a in arb_action(), b in arb_action()) {
        prop_assert_eq!(a == b, b == a);
    }

    #[test]
    fn shift_eq_iff_same_state(s1 in any::<u16>(), s2 in any::<u16>()) {
        prop_assert_eq!(
            Action::Shift(s1) == Action::Shift(s2),
            s1 == s2,
        );
    }

    #[test]
    fn reduce_eq_iff_both_match(
        sym1 in any::<u16>(), cc1 in any::<u8>(),
        sym2 in any::<u16>(), cc2 in any::<u8>(),
    ) {
        let a = Action::Reduce { symbol: sym1, child_count: cc1 };
        let b = Action::Reduce { symbol: sym2, child_count: cc2 };
        prop_assert_eq!(a == b, sym1 == sym2 && cc1 == cc2);
    }
}

// ===========================================================================
// 7 – Debug formatting
// ===========================================================================

proptest! {
    #[test]
    fn debug_shift_shows_state(state in any::<u16>()) {
        let dbg = format!("{:?}", Action::Shift(state));
        prop_assert!(dbg.contains("Shift"));
        prop_assert!(dbg.contains(&state.to_string()));
    }

    #[test]
    fn debug_reduce_shows_symbol_and_count(sym in any::<u16>(), cc in any::<u8>()) {
        let dbg = format!("{:?}", Action::Reduce { symbol: sym, child_count: cc });
        prop_assert!(dbg.contains("Reduce"));
        prop_assert!(dbg.contains(&sym.to_string()));
        prop_assert!(dbg.contains(&cc.to_string()));
    }

    #[test]
    fn debug_accept_shows_name(_ in 0..1u8) {
        let dbg = format!("{:?}", Action::Accept);
        prop_assert!(dbg.contains("Accept"));
    }

    #[test]
    fn debug_error_shows_name(_ in 0..1u8) {
        let dbg = format!("{:?}", Action::Error);
        prop_assert!(dbg.contains("Error"));
    }

    #[test]
    fn debug_nonempty_for_any_action(a in arb_action()) {
        let dbg = format!("{:?}", a);
        prop_assert!(!dbg.is_empty());
    }
}

// ===========================================================================
// 8 – Action in ParseTable
// ===========================================================================

proptest! {
    #[test]
    fn parse_table_dimensions(states in 1..6usize, symbols in 1..6usize) {
        let action_table: Vec<Vec<Vec<Action>>> = (0..states)
            .map(|_| (0..symbols).map(|_| vec![Action::Error]).collect())
            .collect();
        let table = ParseTable {
            state_count: states,
            action_table,
            small_parse_table: None,
            small_parse_table_map: None,
        };
        prop_assert_eq!(table.action_table.len(), states);
        for row in &table.action_table {
            prop_assert_eq!(row.len(), symbols);
        }
    }

    #[test]
    fn parse_table_multi_action_cell(
        shift_st in any::<u16>(),
        red_sym in any::<u16>(),
        red_cc in any::<u8>(),
    ) {
        let cell = vec![
            Action::Shift(shift_st),
            Action::Reduce { symbol: red_sym, child_count: red_cc },
            Action::Accept,
        ];
        let table = ParseTable {
            state_count: 1,
            action_table: vec![vec![cell]],
            small_parse_table: None,
            small_parse_table_map: None,
        };
        prop_assert_eq!(table.action_table[0][0].len(), 3);
        prop_assert!(matches!(table.action_table[0][0][0], Action::Shift(_)));
        let is_reduce = matches!(table.action_table[0][0][1], Action::Reduce { .. });
        prop_assert!(is_reduce);
        prop_assert_eq!(table.action_table[0][0][2], Action::Accept);
    }

    #[test]
    fn parse_table_clone_equality(states in 1..4usize, symbols in 1..4usize) {
        let action_table: Vec<Vec<Vec<Action>>> = (0..states)
            .map(|s| (0..symbols).map(|_| vec![Action::Shift(s as u16)]).collect())
            .collect();
        let table = ParseTable {
            state_count: states,
            action_table,
            small_parse_table: None,
            small_parse_table_map: None,
        };
        let cloned = table.clone();
        prop_assert_eq!(cloned.state_count, table.state_count);
        for i in 0..states {
            for j in 0..symbols {
                prop_assert_eq!(&cloned.action_table[i][j], &table.action_table[i][j]);
            }
        }
    }

    #[test]
    fn parse_table_empty_cells_allowed(states in 1..4usize, symbols in 1..4usize) {
        let table = ParseTable {
            state_count: states,
            action_table: (0..states)
                .map(|_| (0..symbols).map(|_| vec![]).collect())
                .collect(),
            small_parse_table: None,
            small_parse_table_map: None,
        };
        for row in &table.action_table {
            for cell in row {
                prop_assert!(cell.is_empty());
            }
        }
    }

    #[test]
    fn parse_table_debug_nonempty(_ in 0..1u8) {
        let table = ParseTable {
            state_count: 1,
            action_table: vec![vec![vec![Action::Accept]]],
            small_parse_table: None,
            small_parse_table_map: None,
        };
        let dbg = format!("{:?}", table);
        prop_assert!(!dbg.is_empty());
        prop_assert!(dbg.contains("ParseTable"));
    }
}
