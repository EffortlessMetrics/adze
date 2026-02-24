//! Shared constants and types for Tree-sitter format encoding/decoding
//! This ensures the language builder and decoder use exactly the same action tags

/// Tree-sitter action type tags
/// Note: These values must match Tree-sitter's internal constants exactly
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
    Reduce = 3, // Tree-sitter uses 3 for Reduce (not 2)
    /// Accept action tag (Tree-sitter uses 4 for Accept, not 3).
    Accept = 4, // Tree-sitter uses 4 for Accept (not 3)
}

use adze_glr_core::{Action, ParseTable};

/// Choose a single action from a GLR cell deterministically
/// Prefers Accept > Shift > Reduce > Error
/// This ensures consistent behavior between builder and runtime
pub fn choose_action(cell: &[Action]) -> Option<Action> {
    if cell.is_empty() {
        return None;
    }

    // 1) Accept (only appears on EOF, terminal state)
    if let Some(a) = cell.iter().find(|a| matches!(a, Action::Accept)) {
        return Some(a.clone());
    }
    // 2) Shift (Tree-sitter default for conflicts)
    if let Some(a) = cell.iter().find(|a| matches!(a, Action::Shift(_))) {
        return Some(a.clone());
    }
    // 3) Reduce (only when no shift)
    if let Some(a) = cell.iter().find(|a| matches!(a, Action::Reduce(_))) {
        return Some(a.clone());
    }
    // 4) Error
    Some(Action::Error)
}

/// Choose a single action from a GLR cell with precedence awareness
/// Uses the same priority logic as the runtime parser
pub fn choose_action_with_precedence(cell: &[Action], parse_table: &ParseTable) -> Option<Action> {
    if cell.is_empty() {
        return None;
    }

    // Sort by priority and take the best one
    let mut sorted = cell.to_vec();
    sorted.sort_by_key(|a| -action_priority(a, parse_table));
    sorted.first().cloned()
}

/// Calculate priority for an action based on precedence
#[inline]
fn action_priority(action: &Action, parse_table: &ParseTable) -> i32 {
    use Action::*;

    // Highest: Accept
    if matches!(action, Accept) {
        return 3_000_000;
    }

    // Pull dynamic precedence if this is a reduce
    let mut prec = 0i32;
    if let Reduce(rid) = action {
        // Get dynamic precedence for this rule
        if (rid.0 as usize) < parse_table.dynamic_prec_by_rule.len() {
            prec = parse_table.dynamic_prec_by_rule[rid.0 as usize] as i32;
        }

        // Get associativity from the rule: +1 left, -1 right, 0 none
        let assoc_bias = if (rid.0 as usize) < parse_table.rule_assoc_by_rule.len() {
            parse_table.rule_assoc_by_rule[rid.0 as usize] as i32
        } else {
            0
        };

        // Combine precedence and associativity
        prec = prec.saturating_add(assoc_bias);

        // Bump reduces with positive precedence above plain shift
        if prec > 0 {
            return 2_000_000 + prec;
        }
        // Neutral reduce (slightly below shift to prefer shift in S/R conflicts)
        return 1_500_000 + prec;
    }

    // Plain Shift (default TS policy prefers shift over no-prec reduce)
    if matches!(action, Shift(_)) {
        return 2_000_000;
    }

    0 // Error/other
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_action_tag_constants() {
        // These values are ABI-critical and must never change
        // They must match Tree-sitter's internal constants exactly
        assert_eq!(TSActionTag::Error as u8, 0, "Error tag must be 0");
        assert_eq!(TSActionTag::Shift as u8, 1, "Shift tag must be 1");
        assert_eq!(
            TSActionTag::Reduce as u8,
            3,
            "Reduce tag must be 3 (TS uses 2 for Recover)"
        );
        assert_eq!(TSActionTag::Accept as u8, 4, "Accept tag must be 4");

        // Also verify ordering for PartialOrd
        assert!(TSActionTag::Error < TSActionTag::Shift);
        assert!(TSActionTag::Shift < TSActionTag::Reduce);
        assert!(TSActionTag::Reduce < TSActionTag::Accept);
    }
}
