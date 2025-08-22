//! Shared constants and types for Tree-sitter format encoding/decoding
//! This ensures the language builder and decoder use exactly the same action tags

/// Tree-sitter action type tags
/// Note: These values must match Tree-sitter's internal constants exactly
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TSActionTag {
    Error = 0,
    Shift = 1,
    Reduce = 3, // Tree-sitter uses 3 for Reduce (not 2)
    // Tree-sitter uses 2 for "Recover" actions, which are not used in rust-sitter
    Accept = 4, // Tree-sitter uses 4 for Accept (not 3)
}

use rust_sitter_glr_core::Action;

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
