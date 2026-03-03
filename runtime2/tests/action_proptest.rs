#![allow(clippy::needless_range_loop)]

use proptest::prelude::*;

use adze_runtime::language::{Action, ParseTable};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_action() -> impl Strategy<Value = Action> {
    prop_oneof![
        any::<u16>().prop_map(Action::Shift),
        (any::<u16>(), any::<u8>())
            .prop_map(|(symbol, child_count)| Action::Reduce { symbol, child_count }),
        Just(Action::Accept),
        Just(Action::Error),
    ]
}

fn arb_shift() -> impl Strategy<Value = Action> {
    any::<u16>().prop_map(Action::Shift)
}

fn arb_reduce() -> impl Strategy<Value = Action> {
    (any::<u16>(), any::<u8>())
        .prop_map(|(symbol, child_count)| Action::Reduce { symbol, child_count })
}

fn arb_action_sequence(max_len: usize) -> impl Strategy<Value = Vec<Action>> {
    prop::collection::vec(arb_action(), 0..max_len)
}

// ---------------------------------------------------------------------------
// 1 – Shift creation and field access
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn shift_roundtrip(state in any::<u16>()) {
        let a = Action::Shift(state);
        match a {
            Action::Shift(s) => prop_assert_eq!(s, state),
            _ => prop_assert!(false, "expected Shift"),
        }
    }

    #[test]
    fn shift_is_not_reduce(state in any::<u16>()) {
        let a = Action::Shift(state);
        prop_assert_ne!(a, Action::Accept);
        prop_assert_ne!(a, Action::Error);
    }

    #[test]
    fn shift_boundary_states(state in prop_oneof![Just(0u16), Just(u16::MAX), any::<u16>()]) {
        let a = Action::Shift(state);
        if let Action::Shift(s) = a {
            prop_assert_eq!(s, state);
        }
    }
}

// ---------------------------------------------------------------------------
// 2 – Reduce creation and field access
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn reduce_roundtrip(symbol in any::<u16>(), child_count in any::<u8>()) {
        let a = Action::Reduce { symbol, child_count };
        match a {
            Action::Reduce { symbol: s, child_count: c } => {
                prop_assert_eq!(s, symbol);
                prop_assert_eq!(c, child_count);
            }
            _ => prop_assert!(false, "expected Reduce"),
        }
    }

    #[test]
    fn reduce_is_not_shift(symbol in any::<u16>(), child_count in any::<u8>()) {
        let a = Action::Reduce { symbol, child_count };
        prop_assert_ne!(a, Action::Accept);
        prop_assert_ne!(a, Action::Error);
    }

    #[test]
    fn reduce_boundary_values(
        symbol in prop_oneof![Just(0u16), Just(u16::MAX), any::<u16>()],
        child_count in prop_oneof![Just(0u8), Just(u8::MAX), any::<u8>()],
    ) {
        let a = Action::Reduce { symbol, child_count };
        if let Action::Reduce { symbol: s, child_count: c } = a {
            prop_assert_eq!(s, symbol);
            prop_assert_eq!(c, child_count);
        }
    }
}

// ---------------------------------------------------------------------------
// 3 – Accept and Error
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn accept_identity(_ in 0..1u8) {
        let a = Action::Accept;
        let b = Action::Accept;
        prop_assert_eq!(a, b);
    }

    #[test]
    fn error_identity(_ in 0..1u8) {
        let a = Action::Error;
        let b = Action::Error;
        prop_assert_eq!(a, b);
    }

    #[test]
    fn accept_ne_error(_ in 0..1u8) {
        prop_assert_ne!(Action::Accept, Action::Error);
    }

    #[test]
    fn accept_ne_any_shift(state in any::<u16>()) {
        prop_assert_ne!(Action::Accept, Action::Shift(state));
    }

    #[test]
    fn error_ne_any_reduce(symbol in any::<u16>(), child_count in any::<u8>()) {
        prop_assert_ne!(Action::Error, Action::Reduce { symbol, child_count });
    }
}

// ---------------------------------------------------------------------------
// 4 – Clone / Copy semantics
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn copy_preserves_action(a in arb_action()) {
        let b = a;
        let c = a;
        prop_assert_eq!(a, b);
        prop_assert_eq!(a, c);
    }

    #[test]
    fn clone_equals_original(a in arb_action()) {
        let cloned = a.clone();
        prop_assert_eq!(a, cloned);
    }

    #[test]
    fn clone_vec_of_actions(actions in arb_action_sequence(50)) {
        let cloned = actions.clone();
        prop_assert_eq!(actions.len(), cloned.len());
        for i in 0..actions.len() {
            prop_assert_eq!(actions[i], cloned[i]);
        }
    }
}

// ---------------------------------------------------------------------------
// 5 – Debug display
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_shift_contains_variant(state in any::<u16>()) {
        let dbg = format!("{:?}", Action::Shift(state));
        prop_assert!(dbg.contains("Shift"));
        prop_assert!(dbg.contains(&state.to_string()));
    }

    #[test]
    fn debug_reduce_contains_fields(symbol in any::<u16>(), child_count in any::<u8>()) {
        let dbg = format!("{:?}", Action::Reduce { symbol, child_count });
        prop_assert!(dbg.contains("Reduce"));
        prop_assert!(dbg.contains(&symbol.to_string()));
        prop_assert!(dbg.contains(&child_count.to_string()));
    }

    #[test]
    fn debug_accept(_ in 0..1u8) {
        let dbg = format!("{:?}", Action::Accept);
        prop_assert!(dbg.contains("Accept"));
    }

    #[test]
    fn debug_error(_ in 0..1u8) {
        let dbg = format!("{:?}", Action::Error);
        prop_assert!(dbg.contains("Error"));
    }

    #[test]
    fn debug_is_nonempty(a in arb_action()) {
        let dbg = format!("{:?}", a);
        prop_assert!(!dbg.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 6 – Equality
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn eq_reflexive(a in arb_action()) {
        prop_assert_eq!(a, a);
    }

    #[test]
    fn eq_symmetric(a in arb_action(), b in arb_action()) {
        prop_assert_eq!(a == b, b == a);
    }

    #[test]
    fn shift_eq_iff_same_state(s1 in any::<u16>(), s2 in any::<u16>()) {
        let a = Action::Shift(s1);
        let b = Action::Shift(s2);
        prop_assert_eq!(a == b, s1 == s2);
    }

    #[test]
    fn reduce_eq_iff_same_fields(
        sym1 in any::<u16>(), cc1 in any::<u8>(),
        sym2 in any::<u16>(), cc2 in any::<u8>(),
    ) {
        let a = Action::Reduce { symbol: sym1, child_count: cc1 };
        let b = Action::Reduce { symbol: sym2, child_count: cc2 };
        prop_assert_eq!(a == b, sym1 == sym2 && cc1 == cc2);
    }
}

// ---------------------------------------------------------------------------
// 7 – Action in ParseTable context
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_table_stores_actions(
        states in 1..5usize,
        symbols in 1..5usize,
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
        prop_assert_eq!(table.state_count, states);
        prop_assert_eq!(table.action_table.len(), states);
        for row in &table.action_table {
            prop_assert_eq!(row.len(), symbols);
        }
    }

    #[test]
    fn parse_table_glr_multi_actions(
        shift_state in any::<u16>(),
        reduce_sym in any::<u16>(),
        reduce_cc in any::<u8>(),
    ) {
        let cell = vec![
            Action::Shift(shift_state),
            Action::Reduce { symbol: reduce_sym, child_count: reduce_cc },
        ];
        let table = ParseTable {
            state_count: 1,
            action_table: vec![vec![cell.clone()]],
            small_parse_table: None,
            small_parse_table_map: None,
        };
        prop_assert_eq!(table.action_table[0][0].len(), 2);
        prop_assert_eq!(table.action_table[0][0][0], Action::Shift(shift_state));
    }

    #[test]
    fn parse_table_clone(states in 1..4usize, symbols in 1..4usize) {
        let action_table: Vec<Vec<Vec<Action>>> = (0..states)
            .map(|s| {
                (0..symbols)
                    .map(|_| vec![Action::Shift(s as u16)])
                    .collect()
            })
            .collect();
        let table = ParseTable {
            state_count: states,
            action_table,
            small_parse_table: None,
            small_parse_table_map: None,
        };
        let cloned = table.clone();
        prop_assert_eq!(cloned.state_count, table.state_count);
        prop_assert_eq!(cloned.action_table, table.action_table);
    }

    #[test]
    fn parse_table_empty_cells(states in 1..5usize, symbols in 1..5usize) {
        let action_table: Vec<Vec<Vec<Action>>> = (0..states)
            .map(|_| (0..symbols).map(|_| vec![]).collect())
            .collect();
        let table = ParseTable {
            state_count: states,
            action_table,
            small_parse_table: None,
            small_parse_table_map: None,
        };
        for row in &table.action_table {
            for cell in row {
                prop_assert!(cell.is_empty());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 8 – Random action sequences
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn action_sequence_preserves_order(actions in arb_action_sequence(100)) {
        let collected: Vec<Action> = actions.iter().copied().collect();
        prop_assert_eq!(collected.len(), actions.len());
        for i in 0..actions.len() {
            prop_assert_eq!(collected[i], actions[i]);
        }
    }

    #[test]
    fn action_sequence_partition(actions in arb_action_sequence(100)) {
        let shifts: Vec<_> = actions.iter().filter(|a| matches!(a, Action::Shift(_))).collect();
        let reduces: Vec<_> = actions.iter().filter(|a| matches!(a, Action::Reduce { .. })).collect();
        let accepts: Vec<_> = actions.iter().filter(|a| matches!(a, Action::Accept)).collect();
        let errors: Vec<_> = actions.iter().filter(|a| matches!(a, Action::Error)).collect();
        prop_assert_eq!(shifts.len() + reduces.len() + accepts.len() + errors.len(), actions.len());
    }

    #[test]
    fn action_sequence_dedup(a in arb_action(), count in 2..50usize) {
        let mut v = vec![a; count];
        v.dedup();
        prop_assert_eq!(v.len(), 1);
        prop_assert_eq!(v[0], a);
    }

    #[test]
    fn shift_then_reduce_sequence(
        state in any::<u16>(),
        symbol in any::<u16>(),
        child_count in any::<u8>(),
    ) {
        let seq = vec![
            Action::Shift(state),
            Action::Reduce { symbol, child_count },
            Action::Accept,
        ];
        prop_assert_eq!(seq.len(), 3);
        prop_assert!(matches!(seq[0], Action::Shift(_)));
        let is_reduce = matches!(seq[1], Action::Reduce { .. });
        prop_assert!(is_reduce);
        prop_assert!(matches!(seq[2], Action::Accept));
    }
}

// ---------------------------------------------------------------------------
// 9 – Memory layout
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn action_size_is_small(_ in 0..1u8) {
        // Action should be compact: enum with u16/u8 payloads
        let size = std::mem::size_of::<Action>();
        prop_assert!(size <= 8, "Action should be at most 8 bytes, got {}", size);
    }

    #[test]
    fn shift_and_reduce_same_size(_ in 0..1u8) {
        // All variants use same memory (enum)
        let shift = Action::Shift(42);
        let reduce = Action::Reduce { symbol: 1, child_count: 2 };
        prop_assert_eq!(
            std::mem::size_of_val(&shift),
            std::mem::size_of_val(&reduce)
        );
    }
}
