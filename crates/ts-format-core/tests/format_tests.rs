use adze_glr_core::{Action, RuleId, StateId};
use adze_ts_format_core::{TSActionTag, choose_action};

#[test]
fn action_tag_values_match_tree_sitter_abi() {
    assert_eq!(TSActionTag::Error as u8, 0);
    assert_eq!(TSActionTag::Shift as u8, 1);
    assert_eq!(TSActionTag::Recover as u8, 2);
    assert_eq!(TSActionTag::Reduce as u8, 3);
    assert_eq!(TSActionTag::Accept as u8, 4);
}

#[test]
fn action_tag_ordering() {
    assert!(TSActionTag::Error < TSActionTag::Shift);
    assert!(TSActionTag::Shift < TSActionTag::Recover);
    assert!(TSActionTag::Recover < TSActionTag::Reduce);
    assert!(TSActionTag::Reduce < TSActionTag::Accept);
}

#[test]
fn choose_action_empty_returns_none() {
    assert_eq!(choose_action(&[]), None);
}

#[test]
fn choose_action_accept_wins_over_all() {
    let cell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
    ];
    assert_eq!(choose_action(&cell), Some(Action::Accept));
}

#[test]
fn choose_action_shift_over_reduce() {
    let cell = vec![Action::Reduce(RuleId(0)), Action::Shift(StateId(5))];
    assert_eq!(choose_action(&cell), Some(Action::Shift(StateId(5))));
}

#[test]
fn choose_action_reduce_over_error() {
    let cell = vec![Action::Error, Action::Reduce(RuleId(3))];
    assert_eq!(choose_action(&cell), Some(Action::Reduce(RuleId(3))));
}

#[test]
fn choose_action_single_error_returns_error() {
    let cell = vec![Action::Error];
    assert_eq!(choose_action(&cell), Some(Action::Error));
}

#[test]
fn choose_action_single_shift() {
    let cell = vec![Action::Shift(StateId(42))];
    assert_eq!(choose_action(&cell), Some(Action::Shift(StateId(42))));
}
