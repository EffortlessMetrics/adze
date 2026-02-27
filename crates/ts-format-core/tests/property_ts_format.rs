use adze_glr_core::{Action, ParseTable, RuleId, StateId};
use adze_ts_format_core::{choose_action, choose_action_with_precedence};
use proptest::prelude::*;

fn action_from_byte(byte: u8, rule: u16, state: u16) -> Action {
    match byte % 5 {
        0 => Action::Error,
        1 => Action::Shift(StateId(state)),
        2 => Action::Reduce(RuleId(rule)),
        3 => Action::Accept,
        _ => Action::Recover,
    }
}

fn expected_choose_with_precedence(actions: &[Action], parse_table: &ParseTable) -> Option<Action> {
    actions
        .iter()
        .max_by_key(|action| action_priority(action, parse_table))
        .cloned()
}

fn action_priority(action: &Action, parse_table: &ParseTable) -> i32 {
    use Action::*;
    if matches!(action, Accept) {
        return 3_000_000;
    }

    if let Reduce(rid) = action {
        let mut prec = 0i32;
        if (rid.0 as usize) < parse_table.dynamic_prec_by_rule.len() {
            prec = parse_table.dynamic_prec_by_rule[rid.0 as usize] as i32;
        }
        if (rid.0 as usize) < parse_table.rule_assoc_by_rule.len() {
            prec += parse_table.rule_assoc_by_rule[rid.0 as usize] as i32;
        }
        return if prec > 0 {
            2_000_000 + prec
        } else {
            1_500_000 + prec
        };
    }

    if matches!(action, Shift(_)) {
        return 2_000_000;
    }

    0
}

proptest! {
    #[test]
    fn choose_action_matches_manual_priority(
        input in prop::collection::vec(any::<u8>(), 0..32),
        state_seed in 0u16..1024,
        rule_seed in 0u16..1024,
        dyn_prec in 0i16..16,
        assoc in -1i8..=1,
    ) {
        let actions: Vec<Action> = input
            .iter()
            .map(|b| action_from_byte(*b, rule_seed, state_seed))
            .collect();

        let parse_table = ParseTable {
            dynamic_prec_by_rule: vec![dyn_prec],
            rule_assoc_by_rule: vec![assoc],
            ..ParseTable::default()
        };

        let expected = expected_choose_with_precedence(&actions, &parse_table);
        let actual = choose_action_with_precedence(&actions, &parse_table);

        if expected != actual {
            let expected_priority = expected
                .as_ref()
                .map(|action| action_priority(action, &parse_table));
            let actual_priority = actual
                .as_ref()
                .map(|action| action_priority(action, &parse_table));
            assert_eq!(expected_priority, actual_priority);
        }
        if let Some(chosen) = actual.clone() {
            assert!(actions.contains(&chosen));
        }

        let simple = choose_action(&actions);
        if simple.is_none() {
            assert!(actions.is_empty());
        } else {
            let simple = simple.clone().unwrap();
            if !actions.contains(&simple) {
                // `choose_action` intentionally falls back to `Error` when no
                // Accept/Shift/Reduce action exists (e.g. Recover-only cells).
                assert_eq!(simple, Action::Error);
                assert!(actions.iter().all(|a| matches!(a, Action::Recover)));
            }
        }
    }
}
