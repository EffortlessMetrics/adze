use adze::ts_format::{TSActionTag as RuntimeTag, choose_action as runtime_choose_action};
use adze_glr_core::Action;
use adze_ir::{RuleId, StateId};
use adze_ts_format_core::{TSActionTag as CoreTag, choose_action as core_choose_action};

#[test]
fn runtime_reexport_matches_core_tag_values() {
    assert_eq!(RuntimeTag::Error as u8, CoreTag::Error as u8);
    assert_eq!(RuntimeTag::Shift as u8, CoreTag::Shift as u8);
    assert_eq!(RuntimeTag::Recover as u8, CoreTag::Recover as u8);
    assert_eq!(RuntimeTag::Reduce as u8, CoreTag::Reduce as u8);
    assert_eq!(RuntimeTag::Accept as u8, CoreTag::Accept as u8);
}

#[test]
fn runtime_reexport_matches_core_action_selection() {
    let cells = [
        vec![Action::Accept, Action::Shift(StateId(1))],
        vec![Action::Shift(StateId(2)), Action::Reduce(RuleId(3))],
        vec![Action::Reduce(RuleId(4)), Action::Error],
        vec![],
    ];

    for cell in cells {
        let runtime = runtime_choose_action(&cell);
        let core = core_choose_action(&cell);
        assert_eq!(runtime, core);
    }
}
