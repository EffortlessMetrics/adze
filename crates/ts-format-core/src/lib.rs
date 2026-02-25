//! Shared constants and helpers for Tree-sitter-compatible parsing tables.
//!
//! This crate captures action tags and deterministic action selection behavior used by
//! both runtime and tablegen/parsing code paths.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use adze_glr_core::{Action, ParseTable};

/// Tree-sitter action type tags.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TSActionTag {
    /// Error action tag.
    Error = 0,
    /// Shift action tag.
    Shift = 1,
    /// Recover action tag.
    Recover = 2,
    /// Reduce action tag (Tree-sitter uses `3` for Reduce).
    Reduce = 3,
    /// Accept action tag (Tree-sitter uses `4` for Accept).
    Accept = 4,
}

/// Choose a single action from a GLR cell deterministically.
///
/// Priority is `Accept > Shift > Reduce > Error`, matching existing runtime behavior.
#[must_use]
pub fn choose_action(cell: &[Action]) -> Option<Action> {
    if cell.is_empty() {
        return None;
    }

    if let Some(a) = cell.iter().find(|a| matches!(a, Action::Accept)) {
        return Some(a.clone());
    }
    if let Some(a) = cell.iter().find(|a| matches!(a, Action::Shift(_))) {
        return Some(a.clone());
    }
    if let Some(a) = cell.iter().find(|a| matches!(a, Action::Reduce(_))) {
        return Some(a.clone());
    }
    Some(Action::Error)
}

/// Choose a single action from a GLR cell using precedence-aware scoring.
///
/// This is used when grammar/runtime policy needs tie-breaking with dynamic rule precedence.
#[must_use]
pub fn choose_action_with_precedence(cell: &[Action], parse_table: &ParseTable) -> Option<Action> {
    if cell.is_empty() {
        return None;
    }

    let mut sorted = cell.to_vec();
    sorted.sort_by_key(|a| -action_priority(a, parse_table));
    sorted.first().cloned()
}

#[inline]
fn action_priority(action: &Action, parse_table: &ParseTable) -> i32 {
    use Action::*;

    if matches!(action, Accept) {
        return 3_000_000;
    }

    let mut prec = 0i32;
    if let Reduce(rid) = action {
        if (rid.0 as usize) < parse_table.dynamic_prec_by_rule.len() {
            prec = parse_table.dynamic_prec_by_rule[rid.0 as usize] as i32;
        }

        if (rid.0 as usize) < parse_table.rule_assoc_by_rule.len() {
            prec = prec.saturating_add(parse_table.rule_assoc_by_rule[rid.0 as usize] as i32);
        }

        if prec > 0 {
            return 2_000_000 + prec;
        }
        return 1_500_000 + prec;
    }

    if matches!(action, Shift(_)) {
        return 2_000_000;
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_action_tag_constants() {
        assert_eq!(TSActionTag::Error as u8, 0, "Error tag must be 0");
        assert_eq!(TSActionTag::Shift as u8, 1, "Shift tag must be 1");
        assert_eq!(
            TSActionTag::Reduce as u8,
            3,
            "Reduce tag must be 3 (Tree-sitter uses 2 for Recover)"
        );
        assert_eq!(TSActionTag::Accept as u8, 4, "Accept tag must be 4");

        assert!(TSActionTag::Error < TSActionTag::Shift);
        assert!(TSActionTag::Shift < TSActionTag::Reduce);
        assert!(TSActionTag::Reduce < TSActionTag::Accept);
    }

    #[test]
    fn choose_action_prefers_shift_over_reduce() {
        use adze_glr_core::{RuleId, StateId};

        let cell = vec![
            Action::Shift(StateId(1)),
            Action::Reduce(RuleId(0)),
            Action::Accept,
            Action::Error,
        ];

        let chosen = choose_action(&cell).expect("expected one action");
        assert_eq!(chosen, Action::Accept);
    }
}
