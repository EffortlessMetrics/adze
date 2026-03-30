//! Contract lock test - verifies that public API remains stable.

use adze_glr_core::{Action, ParseTable};
use adze_ts_format_core::{TSActionTag, choose_action, choose_action_with_precedence};

/// Verify all public types exist and have expected structure.
#[test]
fn test_contract_lock_types() {
    // Verify TSActionTag enum exists with all variants
    let error = TSActionTag::Error;
    let shift = TSActionTag::Shift;
    let recover = TSActionTag::Recover;
    let reduce = TSActionTag::Reduce;
    let accept = TSActionTag::Accept;

    // Verify repr(u8) values
    assert_eq!(error as u8, 0);
    assert_eq!(shift as u8, 1);
    assert_eq!(recover as u8, 2);
    assert_eq!(reduce as u8, 3);
    assert_eq!(accept as u8, 4);

    // Verify Debug trait is implemented
    let _debug = format!("{shift:?}");

    // Verify Clone trait is implemented
    let _cloned = shift.clone();

    // Verify Copy trait is implemented
    let _copied: TSActionTag = shift;

    // Verify PartialEq trait is implemented
    assert_eq!(shift, shift);
    assert_ne!(shift, reduce);

    // Verify Eq trait is implemented
    assert_eq!(reduce, reduce);

    // Verify PartialOrd trait is implemented
    assert!(shift < reduce);

    // Verify Ord trait is implemented
    assert!(error < shift);
    assert!(shift < recover);
    assert!(recover < reduce);
    assert!(reduce < accept);
}

/// Verify all public functions exist with expected signatures.
#[test]
fn test_contract_lock_functions() {
    // Verify choose_action function exists with expected signature
    let empty_cell: Vec<Action> = vec![];
    let result = choose_action(&empty_cell);
    assert!(result.is_none());

    // Verify choose_action returns Some(Action) for non-empty cells
    let cell = vec![Action::Error];
    let result = choose_action(&cell);
    assert!(result.is_some());

    // Verify choose_action_with_precedence function exists with expected signature
    let pt = ParseTable::default();
    let result = choose_action_with_precedence(&empty_cell, &pt);
    assert!(result.is_none());

    // Verify function signature via function pointer
    let _fn_ptr: Option<fn(&[Action]) -> Option<Action>> = Some(choose_action);
    let _fn_ptr_prec: Option<fn(&[Action], &ParseTable) -> Option<Action>> =
        Some(choose_action_with_precedence);
}

/// Verify choose_action priority order: Accept > Shift > Reduce > Error.
#[test]
fn test_contract_lock_choose_action_priority() {
    use adze_glr_core::{RuleId, StateId};

    // Accept has highest priority
    let cell = vec![
        Action::Error,
        Action::Reduce(RuleId(0)),
        Action::Shift(StateId(1)),
        Action::Accept,
    ];
    assert_eq!(choose_action(&cell), Some(Action::Accept));

    // Shift has second priority
    let cell = vec![
        Action::Error,
        Action::Reduce(RuleId(0)),
        Action::Shift(StateId(1)),
    ];
    assert_eq!(choose_action(&cell), Some(Action::Shift(StateId(1))));

    // Reduce has third priority
    let cell = vec![Action::Error, Action::Reduce(RuleId(0))];
    assert_eq!(choose_action(&cell), Some(Action::Reduce(RuleId(0))));

    // Error is default
    let cell = vec![Action::Error];
    assert_eq!(choose_action(&cell), Some(Action::Error));
}
