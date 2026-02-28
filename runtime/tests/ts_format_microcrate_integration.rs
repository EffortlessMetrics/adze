use adze::ts_format::{
    TSActionTag as RuntimeTSActionTag, choose_action as runtime_choose_action,
    choose_action_with_precedence as runtime_choose_action_with_precedence,
};
use adze_glr_core::{Action, ParseTable};
use adze_ir::{RuleId, StateId};
use adze_ts_format_core::{
    TSActionTag as CoreTSActionTag, choose_action as core_choose_action,
    choose_action_with_precedence as core_choose_action_with_precedence,
};

#[test]
fn runtime_reexport_matches_microcrate_action_tags() {
    assert_eq!(
        RuntimeTSActionTag::Error as u8,
        CoreTSActionTag::Error as u8
    );
    assert_eq!(
        RuntimeTSActionTag::Shift as u8,
        CoreTSActionTag::Shift as u8
    );
    assert_eq!(
        RuntimeTSActionTag::Recover as u8,
        CoreTSActionTag::Recover as u8
    );
    assert_eq!(
        RuntimeTSActionTag::Reduce as u8,
        CoreTSActionTag::Reduce as u8
    );
    assert_eq!(
        RuntimeTSActionTag::Accept as u8,
        CoreTSActionTag::Accept as u8
    );
}

#[test]
fn runtime_reexport_matches_microcrate_choose_action_contract() {
    let cells = vec![
        vec![],
        vec![Action::Error],
        vec![Action::Shift(StateId(7))],
        vec![Action::Reduce(RuleId(5))],
        vec![Action::Reduce(RuleId(2)), Action::Shift(StateId(4))],
        vec![Action::Shift(StateId(1)), Action::Accept],
    ];

    for cell in cells {
        assert_eq!(runtime_choose_action(&cell), core_choose_action(&cell));
    }
}

#[test]
fn runtime_reexport_matches_microcrate_choose_action_with_precedence() {
    let parse_table = ParseTable::default();
    let cells = vec![
        vec![Action::Error],
        vec![Action::Shift(StateId(7))],
        vec![Action::Reduce(RuleId(5)), Action::Shift(StateId(4))],
        vec![Action::Shift(StateId(1)), Action::Accept],
    ];

    for cell in cells {
        assert_eq!(
            runtime_choose_action_with_precedence(&cell, &parse_table),
            core_choose_action_with_precedence(&cell, &parse_table)
        );
    }
}
