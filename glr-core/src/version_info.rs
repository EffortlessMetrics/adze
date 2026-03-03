//! Tree-sitter-compatible GLR parse-version comparison logic.
//!
//! This module was previously provided via `adze-glr-versioning`; it is now kept
//! inline in `adze-glr-core` to decouple published crates from that internal crate.

use std::cmp::Ordering;

/// Information about a particular parse version (fork) used for GLR conflict resolution.
#[derive(Debug, Clone, Default)]
pub struct VersionInfo {
    /// Whether this parse path is in error-recovery mode.
    pub in_error: bool,

    /// Cost of skipped/error nodes.
    pub cost: usize,

    /// Number of nodes in error (for cost calculation).
    pub node_count: usize,

    /// Cumulative dynamic precedence for this path.
    ///
    /// This is the sum of all dynamic precedence values in the parse tree.
    pub dynamic_prec: i32,
}

impl VersionInfo {
    /// Create a default parse-version descriptor.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add dynamic precedence from a newly reduced node.
    pub fn add_dynamic_prec(&mut self, prec: i32) {
        // Tree-sitter uses sum to prefer deeper dynamic annotations.
        self.dynamic_prec += prec;
    }

    /// Enter error recovery mode.
    pub fn enter_error(&mut self) {
        self.in_error = true;
    }

    /// Add error cost information.
    pub fn add_error_cost(&mut self, cost: usize, nodes: usize) {
        self.cost += cost;
        self.node_count += nodes;
    }
}

/// Result of comparing two parse versions.
#[derive(Debug, PartialEq, Eq)]
pub enum CompareResult {
    /// Choose the left version unconditionally.
    TakeLeft,
    /// Choose the right version unconditionally.
    TakeRight,
    /// Prefer the left version, but keep the other as alternative.
    PreferLeft,
    /// Prefer the right version, but keep the other as alternative.
    PreferRight,
    /// They are equivalent (need a downstream tie-breaker).
    Tie,
}

/// Compare two parse versions according to Tree-sitter's exact algorithm.
#[must_use]
pub fn compare_versions(a: &VersionInfo, b: &VersionInfo) -> CompareResult {
    // Step 1: Error vs non-error.
    // Non-error paths always win.
    match (a.in_error, b.in_error) {
        (false, true) => return CompareResult::TakeLeft,
        (true, false) => return CompareResult::TakeRight,
        _ => {}
    }

    // Step 2: Error cost comparison.
    // Tree-sitter's exact constants.
    const ERROR_COST_PER_SKIPPED_TREE: usize = 100;
    #[allow(dead_code)]
    const ERROR_COST_PER_SKIPPED_CHAR: usize = 1;
    #[allow(dead_code)]
    const ERROR_COST_PER_RECOVERY: usize = 500;
    const MAX_COST_DIFF_FACTOR: usize = 18;

    if a.cost != b.cost {
        let cost_diff = a.cost.abs_diff(b.cost);

        // Tree-sitter's exact formula for "take" threshold.
        let total_node_count = a.node_count + b.node_count;
        let threshold =
            MAX_COST_DIFF_FACTOR * ERROR_COST_PER_SKIPPED_TREE * total_node_count.max(1);

        if cost_diff >= threshold {
            // Large cost difference: take unconditionally.
            return if a.cost < b.cost {
                CompareResult::TakeLeft
            } else {
                CompareResult::TakeRight
            };
        }

        // Small cost difference: prefer, but keep both alternatives.
        return if a.cost < b.cost {
            CompareResult::PreferLeft
        } else {
            CompareResult::PreferRight
        };
    }

    // Step 3: Dynamic precedence comparison.
    // Higher dynamic precedence wins.
    match a.dynamic_prec.cmp(&b.dynamic_prec) {
        Ordering::Greater => return CompareResult::TakeLeft,
        Ordering::Less => return CompareResult::TakeRight,
        Ordering::Equal => {}
    }

    // Step 4: Fallback to external tie-breaker.
    CompareResult::Tie
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_preference() {
        let a = VersionInfo::new();
        let mut b = VersionInfo::new();
        b.enter_error();

        assert_eq!(compare_versions(&a, &b), CompareResult::TakeLeft);
        assert_eq!(compare_versions(&b, &a), CompareResult::TakeRight);
    }

    #[test]
    fn test_cost_take_threshold() {
        let mut a = VersionInfo::new();
        let mut b = VersionInfo::new();

        // Set up large cost difference.
        a.add_error_cost(0, 1);
        b.add_error_cost(5000, 1);

        // Should exceed threshold (18 * 100 * 2 = 3600).
        assert_eq!(compare_versions(&a, &b), CompareResult::TakeLeft);
    }

    #[test]
    fn test_cost_prefer() {
        let mut a = VersionInfo::new();
        let mut b = VersionInfo::new();

        // Small cost difference.
        a.add_error_cost(100, 1);
        b.add_error_cost(200, 1);

        // Should not exceed threshold.
        assert_eq!(compare_versions(&a, &b), CompareResult::PreferLeft);
    }

    #[test]
    fn test_dynamic_precedence() {
        let mut a = VersionInfo::new();
        let mut b = VersionInfo::new();

        a.add_dynamic_prec(5);
        b.add_dynamic_prec(3);

        assert_eq!(compare_versions(&a, &b), CompareResult::TakeLeft);
        assert_eq!(compare_versions(&b, &a), CompareResult::TakeRight);
    }

    #[test]
    fn test_cumulative_dynamic_precedence() {
        let mut a = VersionInfo::new();
        let mut b = VersionInfo::new();

        a.add_dynamic_prec(2);
        a.add_dynamic_prec(3);

        b.add_dynamic_prec(4);

        // a has total 5, b has total 4.
        assert_eq!(compare_versions(&a, &b), CompareResult::TakeLeft);
    }

    #[test]
    fn test_tie() {
        let a = VersionInfo::new();
        let b = VersionInfo::new();

        assert_eq!(compare_versions(&a, &b), CompareResult::Tie);
    }
}
