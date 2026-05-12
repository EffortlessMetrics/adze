use crate::{Action, ActionCell};
use std::collections::BTreeMap;

pub(super) fn normalize_action_table(action_table: &mut Vec<Vec<ActionCell>>) {
    for row in action_table.iter_mut() {
        for cell in row.iter_mut() {
            normalize_action_cell(cell);
        }
    }
}

fn normalize_action_cell(cell: &mut ActionCell) {
    for action in cell.iter_mut() {
        normalize_action(action);
    }
    cell.sort_by_key(action_sort_key);
    cell.dedup();
}

pub(crate) fn normalize_action(action: &mut Action) {
    if let Action::Fork(inner) = action {
        for inner_action in inner.iter_mut() {
            normalize_action(inner_action);
        }
        inner.sort_by_key(action_sort_key);
        inner.dedup();
    }
}

fn action_sort_key(action: &Action) -> (u8, u16, u16, u16) {
    match action {
        Action::Shift(s) => (0, s.0, 0, 0),
        Action::Reduce(r) => (1, r.0, 0, 0),
        Action::Accept => (2, 0, 0, 0),
        Action::Error => (3, 0, 0, 0),
        Action::Recover => (4, 0, 0, 0),
        Action::Fork(inner) => {
            let first = inner.first().map(action_sort_key).unwrap_or((0, 0, 0, 0));
            (5, first.1, first.2, inner.len() as u16)
        }
    }
}

pub(super) fn add_action_with_conflict(
    action_table: &mut Vec<Vec<ActionCell>>,
    conflicts_by_state: &mut BTreeMap<(usize, usize), Vec<Action>>,
    state_idx: usize,
    symbol_idx: usize,
    new_action: Action,
) {
    if state_idx >= action_table.len() || symbol_idx >= action_table[0].len() {
        panic!(
            "Index out of bounds in add_action_with_conflict: state_idx={}, symbol_idx={}, table_size={}x{}",
            state_idx,
            symbol_idx,
            action_table.len(),
            if action_table.is_empty() {
                0
            } else {
                action_table[0].len()
            }
        );
    }

    let current_cell = &mut action_table[state_idx][symbol_idx];

    if !current_cell.iter().any(|a| action_eq(a, &new_action)) {
        current_cell.push(new_action.clone());

        if current_cell.len() > 1 {
            let entry = conflicts_by_state
                .entry((state_idx, symbol_idx))
                .or_default();
            *entry = current_cell.clone();
        }
    }
}

fn action_eq(a: &Action, b: &Action) -> bool {
    match (a, b) {
        (Action::Shift(s1), Action::Shift(s2)) => s1 == s2,
        (Action::Reduce(r1), Action::Reduce(r2)) => r1 == r2,
        (Action::Accept, Action::Accept) => true,
        (Action::Error, Action::Error) => true,
        (Action::Fork(a1), Action::Fork(a2)) => {
            a1.len() == a2.len() && a1.iter().zip(a2).all(|(x, y)| action_eq(x, y))
        }
        _ => false,
    }
}
