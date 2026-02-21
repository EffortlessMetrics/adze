//! Compatibility facade for governance reporting and parser feature policies.
//!
//! This crate keeps governance-related contracts centralized while preserving
//! compatibility with existing users of `adze-parser-governance-contract`.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_bdd_governance_contract::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_line_is_stable_shape() {
        let profile = ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: true,
            tree_sitter_c2rust: false,
            glr: false,
        };

        let status =
            bdd_progress_status_line(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, profile);
        assert!(status.starts_with("runtime:"));
        assert!(status.contains("tree-sitter C runtime"));
        assert!(status.contains("tree-sitter-standard"));
    }

    #[test]
    fn report_with_profile_is_annotated() {
        let profile = ParserFeatureProfile::current();
        let report = bdd_progress_report_with_profile(
            BddPhase::Runtime,
            GLR_CONFLICT_PRESERVATION_GRID,
            "Runtime",
            profile,
        );

        assert!(report.contains("Feature profile:"));
        assert!(report.contains("Non-conflict backend:"));
        assert!(report.contains("Conflict grammars:"));
    }

    #[test]
    fn progress_with_profile_matches_status() {
        let (implemented, total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
        let profile = ParserFeatureProfile::current();
        let status =
            bdd_progress_status_line(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
        assert!(status.contains(&format!("{implemented}/{total}")));
    }
}
