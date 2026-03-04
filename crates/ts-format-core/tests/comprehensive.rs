// Comprehensive tests for ts-format-core
use adze_glr_core::{Action, ParseTable, RuleId, StateId};
use adze_ts_format_core::*;

// ---------------------------------------------------------------------------
// TSActionTag
// ---------------------------------------------------------------------------

#[test]
fn tag_values() {
    assert_eq!(TSActionTag::Error as u8, 0);
    assert_eq!(TSActionTag::Shift as u8, 1);
    assert_eq!(TSActionTag::Recover as u8, 2);
    assert_eq!(TSActionTag::Reduce as u8, 3);
    assert_eq!(TSActionTag::Accept as u8, 4);
}

#[test]
fn tag_ordering() {
    assert!(TSActionTag::Error < TSActionTag::Shift);
    assert!(TSActionTag::Shift < TSActionTag::Recover);
    assert!(TSActionTag::Recover < TSActionTag::Reduce);
    assert!(TSActionTag::Reduce < TSActionTag::Accept);
}

#[test]
fn tag_eq() {
    assert_eq!(TSActionTag::Shift, TSActionTag::Shift);
    assert_ne!(TSActionTag::Shift, TSActionTag::Reduce);
}

#[test]
fn tag_debug() {
    let d = format!("{:?}", TSActionTag::Shift);
    assert!(d.contains("Shift"));
}

#[test]
fn tag_clone() {
    let t = TSActionTag::Accept;
    let t2 = t;
    assert_eq!(t, t2);
}

// ---------------------------------------------------------------------------
// choose_action
// ---------------------------------------------------------------------------

#[test]
fn choose_empty() {
    assert_eq!(choose_action(&[]), None);
}

#[test]
fn choose_single_shift() {
    let cell = vec![Action::Shift(StateId(1))];
    assert_eq!(choose_action(&cell), Some(Action::Shift(StateId(1))));
}

#[test]
fn choose_single_reduce() {
    let cell = vec![Action::Reduce(RuleId(0))];
    assert_eq!(choose_action(&cell), Some(Action::Reduce(RuleId(0))));
}

#[test]
fn choose_single_accept() {
    let cell = vec![Action::Accept];
    assert_eq!(choose_action(&cell), Some(Action::Accept));
}

#[test]
fn choose_single_error() {
    let cell = vec![Action::Error];
    assert_eq!(choose_action(&cell), Some(Action::Error));
}

#[test]
fn choose_accept_over_shift() {
    let cell = vec![Action::Shift(StateId(1)), Action::Accept];
    assert_eq!(choose_action(&cell), Some(Action::Accept));
}

#[test]
fn choose_shift_over_reduce() {
    let cell = vec![Action::Reduce(RuleId(0)), Action::Shift(StateId(1))];
    assert_eq!(choose_action(&cell), Some(Action::Shift(StateId(1))));
}

#[test]
fn choose_reduce_over_error() {
    let cell = vec![Action::Error, Action::Reduce(RuleId(0))];
    assert_eq!(choose_action(&cell), Some(Action::Reduce(RuleId(0))));
}

#[test]
fn choose_accept_over_all() {
    let cell = vec![
        Action::Error,
        Action::Reduce(RuleId(0)),
        Action::Shift(StateId(1)),
        Action::Accept,
    ];
    assert_eq!(choose_action(&cell), Some(Action::Accept));
}

#[test]
fn choose_only_errors() {
    let cell = vec![Action::Error, Action::Error];
    assert_eq!(choose_action(&cell), Some(Action::Error));
}

// ---------------------------------------------------------------------------
// choose_action_with_precedence
// ---------------------------------------------------------------------------

#[test]
fn choose_with_prec_empty() {
    let table = ParseTable::default();
    assert_eq!(choose_action_with_precedence(&[], &table), None);
}

#[test]
fn choose_with_prec_single_accept() {
    let table = ParseTable::default();
    let cell = vec![Action::Accept];
    assert_eq!(
        choose_action_with_precedence(&cell, &table),
        Some(Action::Accept)
    );
}

#[test]
fn choose_with_prec_shift_and_reduce() {
    let table = ParseTable::default();
    let cell = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
    let result = choose_action_with_precedence(&cell, &table);
    // Without precedence info, shift should win (priority 2M vs 1.5M)
    assert_eq!(result, Some(Action::Shift(StateId(1))));
}

#[test]
fn choose_with_prec_accept_beats_everything() {
    let table = ParseTable::default();
    let cell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(0)),
        Action::Accept,
    ];
    assert_eq!(
        choose_action_with_precedence(&cell, &table),
        Some(Action::Accept)
    );
}
