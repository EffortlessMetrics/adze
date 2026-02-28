//! Shared constants and helpers for Tree-sitter format encoding/decoding.
//! This ensures language builders and decoders use exactly the same action tags.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use adze_glr_core::{Action, ParseTable};

/// Tree-sitter action type tags.
///
/// Note: These values must match Tree-sitter's internal constants exactly.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TSActionTag {
    /// Error action tag.
    Error = 0,
    /// Shift action tag.
    Shift = 1,
    /// Recover action tag.
    Recover = 2,
    /// Reduce action tag (Tree-sitter uses 3 for Reduce, not 2).
    Reduce = 3,
    /// Accept action tag (Tree-sitter uses 4 for Accept, not 3).
    Accept = 4,
}

/// Choose a single action from a GLR cell deterministically.
///
/// Prefers `Accept > Shift > Reduce > Error` for stable behavior.
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

/// Choose a single action from a GLR cell with precedence awareness.
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

    if let Reduce(rid) = action {
        let mut prec = if (rid.0 as usize) < parse_table.dynamic_prec_by_rule.len() {
            parse_table.dynamic_prec_by_rule[rid.0 as usize] as i32
        } else {
            0
        };

        let assoc_bias = if (rid.0 as usize) < parse_table.rule_assoc_by_rule.len() {
            parse_table.rule_assoc_by_rule[rid.0 as usize] as i32
        } else {
            0
        };

        prec = prec.saturating_add(assoc_bias);

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
        assert_eq!(TSActionTag::Error as u8, 0);
        assert_eq!(TSActionTag::Shift as u8, 1);
        assert_eq!(TSActionTag::Reduce as u8, 3);
        assert_eq!(TSActionTag::Accept as u8, 4);

        assert!(TSActionTag::Error < TSActionTag::Shift);
        assert!(TSActionTag::Shift < TSActionTag::Reduce);
        assert!(TSActionTag::Reduce < TSActionTag::Accept);
    }
}
