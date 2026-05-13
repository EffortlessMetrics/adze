use adze::ts_format::{TSActionTag, choose_action, choose_action_with_precedence};
use adze_glr_core::{Action, ParseTable, RuleId, StateId};

#[test]
fn runtime_ts_format_selects_stable_actions() {
    let mut parse_table = ParseTable {
        dynamic_prec_by_rule: vec![2, 5],
        rule_assoc_by_rule: vec![-1, 1],
        ..ParseTable::default()
    };

    let cell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(1)),
        Action::Accept,
        Action::Error,
    ];

    assert_eq!(choose_action(&cell), Some(Action::Accept));
    assert_eq!(
        choose_action_with_precedence(&cell, &parse_table),
        Some(Action::Accept)
    );

    parse_table.dynamic_prec_by_rule[0] = 1;
    assert_eq!(
        choose_action_with_precedence(&cell, &parse_table),
        Some(Action::Accept)
    );
}

#[test]
fn runtime_ts_format_tag_values_match_tree_sitter() {
    assert_eq!(TSActionTag::Error as u8, 0);
    assert_eq!(TSActionTag::Shift as u8, 1);
    assert_eq!(TSActionTag::Recover as u8, 2);
    assert_eq!(TSActionTag::Reduce as u8, 3);
    assert_eq!(TSActionTag::Accept as u8, 4);
}
