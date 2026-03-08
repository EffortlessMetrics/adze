use adze_glr_core::{Action, ParseTable, RuleId, StateId};
use adze_ts_format_core::{TSActionTag, choose_action, choose_action_with_precedence};

#[test]
fn microcrate_choose_action_follows_priority_order() {
    let cell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
        Action::Error,
    ];

    assert_eq!(choose_action(cell.as_slice()), Some(Action::Accept));
}

#[test]
fn microcrate_precedence_can_promote_reduce_over_shift() {
    let mut table = ParseTable {
        dynamic_prec_by_rule: vec![0],
        rule_assoc_by_rule: vec![0],
        ..ParseTable::default()
    };
    let cell = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];

    assert_eq!(
        choose_action_with_precedence(cell.as_slice(), &table),
        Some(Action::Shift(StateId(1)))
    );

    table.dynamic_prec_by_rule[0] = 1;

    assert_eq!(
        choose_action_with_precedence(cell.as_slice(), &table),
        Some(Action::Reduce(RuleId(0)))
    );
}

#[test]
fn microcrate_action_tags_match_tree_sitter_encoding() {
    assert_eq!(TSActionTag::Error as u8, 0);
    assert_eq!(TSActionTag::Shift as u8, 1);
    assert_eq!(TSActionTag::Reduce as u8, 3);
    assert_eq!(TSActionTag::Accept as u8, 4);
}
