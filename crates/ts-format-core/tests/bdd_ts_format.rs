use adze_glr_core::{Action, ParseTable, RuleId, StateId};
use adze_ts_format_core::{TSActionTag, choose_action, choose_action_with_precedence};

#[test]
fn given_accept_and_shift_when_choose_action_then_accept_is_selected() {
    // Given
    let cell = vec![
        Action::Shift(StateId(10)),
        Action::Accept,
        Action::Reduce(RuleId(1)),
        Action::Error,
    ];

    // When
    let chosen = choose_action(&cell);

    // Then
    assert_eq!(chosen, Some(Action::Accept));
    assert_eq!(TSActionTag::Accept as u8, 4);
}

#[test]
fn given_reduction_with_dynamic_precedence_when_choose_action_with_precedence_prefers_higher() {
    // Given
    let cell = vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(0))];
    let parse_table = ParseTable {
        dynamic_prec_by_rule: vec![0, 3],
        rule_assoc_by_rule: vec![0, 0],
        ..ParseTable::default()
    };

    // When
    let chosen = choose_action_with_precedence(&cell, &parse_table);

    // Then
    assert_eq!(chosen, Some(Action::Reduce(RuleId(1))));
}

#[test]
fn given_no_actions_when_choose_action_then_none_is_returned() {
    // Given
    let cell: Vec<Action> = Vec::new();

    // When
    let chosen = choose_action(&cell);
    let fallback = choose_action_with_precedence(&cell, &ParseTable::default());

    // Then
    assert!(chosen.is_none());
    assert!(fallback.is_none());
}
