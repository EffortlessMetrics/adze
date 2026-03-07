use adze::adze_glr_core as runtime_glr_core;
use adze::ts_format::{TSActionTag, choose_action, choose_action_with_precedence};

use adze_glr_core as core_glr_core;
use adze_ts_format_core as core_ts_format;

fn to_core<T, U>(value: &T) -> U
where
    T: serde::Serialize,
    U: serde::de::DeserializeOwned,
{
    serde_json::from_value(serde_json::to_value(value).expect("serialize to json"))
        .expect("deserialize from json")
}

fn debug_string<T>(value: T) -> String
where
    T: std::fmt::Debug,
{
    format!("{value:?}")
}

#[test]
fn runtime_reexport_matches_microcrate_behavior() {
    let mut runtime_table = runtime_glr_core::ParseTable {
        dynamic_prec_by_rule: vec![2, 5],
        rule_assoc_by_rule: vec![-1, 1],
        ..runtime_glr_core::ParseTable::default()
    };

    let cell = vec![
        runtime_glr_core::Action::Shift(runtime_glr_core::StateId(1)),
        runtime_glr_core::Action::Reduce(runtime_glr_core::RuleId(1)),
        runtime_glr_core::Action::Accept,
        runtime_glr_core::Action::Error,
    ];
    let core_cell: Vec<core_glr_core::Action> = to_core(&cell);
    let mut core_table: core_glr_core::ParseTable = to_core(&runtime_table);

    assert_eq!(
        debug_string(choose_action(cell.as_slice())),
        debug_string(core_ts_format::choose_action(core_cell.as_slice()))
    );
    assert_eq!(
        debug_string(choose_action_with_precedence(
            cell.as_slice(),
            &runtime_table
        )),
        debug_string(core_ts_format::choose_action_with_precedence(
            core_cell.as_slice(),
            &core_table
        ))
    );

    runtime_table.dynamic_prec_by_rule[0] = 1;
    core_table.dynamic_prec_by_rule[0] = 1;
    assert_eq!(
        debug_string(choose_action_with_precedence(
            cell.as_slice(),
            &runtime_table
        )),
        debug_string(core_ts_format::choose_action_with_precedence(
            core_cell.as_slice(),
            &core_table
        ))
    );
}

#[test]
fn runtime_reexport_stays_type_compatible() {
    let runtime_tag = TSActionTag::Accept as u8;
    let core_tag = core_ts_format::TSActionTag::Accept as u8;
    assert_eq!(runtime_tag, core_tag);
}
