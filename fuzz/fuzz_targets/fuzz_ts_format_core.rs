#![no_main]

use adze_glr_core::{Action, ParseTable, RuleId, StateId};
use adze_ts_format_core::choose_action_with_precedence;
use libfuzzer_sys::fuzz_target;

fn action_from_byte(byte: u8, idx: u16) -> Action {
    match byte % 5 {
        0 => Action::Error,
        1 => Action::Shift(StateId(idx)),
        2 => Action::Reduce(RuleId(idx)),
        3 => Action::Accept,
        _ => Action::Recover,
    }
}

fn action_has_symbol(table: &ParseTable, action: &Action) -> bool {
    match action {
        Action::Reduce(rule) => {
            let rule_id = rule.0 as usize;
            rule_id <= table.dynamic_prec_by_rule.len() && rule_id <= table.rule_assoc_by_rule.len()
        }
        _ => true,
    }
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let max_rules = (data[0] as usize) % 16 + 1;
    let mut cursor = 1usize;

    let mut cell = Vec::new();
    while cursor < data.len() && cell.len() < 32 {
        let action = action_from_byte(data[cursor], cursor as u16 % 128);
        cell.push(action);
        cursor += 1;
    }

    let table = ParseTable {
        dynamic_prec_by_rule: (0..max_rules)
            .map(|idx| {
                let base = data[idx % data.len()];
                i16::from(base)
            })
            .collect(),
        rule_assoc_by_rule: (0..max_rules)
            .map(|idx| {
                let base = data[(idx + max_rules) % data.len()];
                (base % 3) as i8 - 1
            })
            .collect(),
        ..ParseTable::default()
    };

    let chosen = choose_action_with_precedence(&cell, &table);
    if let Some(action) = chosen {
        assert!(cell.contains(&action));
        assert!(action_has_symbol(&table, &action));
    }
});
