use adze::ts_format::{TSActionTag, choose_action, choose_action_with_precedence};
use adze_glr_core::{Action, ParseTable, RuleId, StateId};
use adze_ts_format_core as core_ts_format;

#[test]
fn runtime_reexport_matches_microcrate_behavior() {
    let mut runtime_table = ParseTable {
        dynamic_prec_by_rule: vec![2, 5],
        rule_assoc_by_rule: vec![-1, 1],
        ..ParseTable::default()
    };
    let mut core_table = ParseTable {
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

    assert_eq!(choose_action(&cell), core_ts_format::choose_action(&cell));
    assert_eq!(
        choose_action_with_precedence(&cell, &runtime_table),
        core_ts_format::choose_action_with_precedence(&cell, &core_table)
    );

    runtime_table.dynamic_prec_by_rule[0] = 1;
    core_table.dynamic_prec_by_rule[0] = 1;
    assert_eq!(
        choose_action_with_precedence(&cell, &runtime_table),
        core_ts_format::choose_action_with_precedence(&cell, &core_table)
    );
}

#[test]
fn runtime_reexport_stays_type_compatible() {
    let runtime_tag = TSActionTag::Accept as u8;
    let core_tag = core_ts_format::TSActionTag::Accept as u8;
    assert_eq!(runtime_tag, core_tag);
}
