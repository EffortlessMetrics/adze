//! Runtime-facing governance API for parser selection and BDD snapshot reporting.
//!
//! This crate intentionally keeps parser/backend selection and feature-flag
//! diagnostics in one place, so downstream crates (runtime, runtime2,
//! adapters, and fixtures) can share a stable surface.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_runtime_governance::{
    BddGovernanceSnapshot, BddPhase, BddScenario, BddScenarioStatus,
    GLR_CONFLICT_PRESERVATION_GRID, ParserBackend, ParserFeatureProfile,
    bdd_governance_matrix_for_current_profile, bdd_governance_matrix_for_profile,
    bdd_governance_matrix_for_runtime, bdd_governance_matrix_for_runtime2,
    bdd_governance_matrix_for_runtime2_profile, bdd_progress_report_for_current_profile,
    bdd_status_line_for_current_profile, current_backend_for, parser_feature_profile_for_runtime,
    resolve_backend_for_profile, runtime_governance_snapshot,
};

pub use adze_runtime_governance::{
    GLR_CONFLICT_FALLBACK, bdd_governance_snapshot, bdd_progress, bdd_progress_report,
    bdd_progress_report_with_profile, bdd_progress_report_with_profile_runtime,
    bdd_progress_status_line, describe_backend_for_conflicts,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_backend_matches_selection_logic() {
        assert_eq!(current_backend_for(false), ParserBackend::select(false));

        #[cfg(feature = "glr")]
        {
            assert_eq!(current_backend_for(true), ParserBackend::select(true));
        }
    }

    #[test]
    fn current_profile_helpers_are_callable() {
        let report = bdd_progress_report_for_current_profile(BddPhase::Runtime, "Runtime");
        assert!(report.contains("Runtime"));
        assert!(report.contains("Governance status:"));

        let status = bdd_status_line_for_current_profile(BddPhase::Runtime);
        assert!(status.starts_with("runtime:"));

        let profile = parser_feature_profile_for_runtime();
        let snapshot = runtime_governance_snapshot(BddPhase::Runtime);
        assert_eq!(snapshot.profile, profile);
    }

    #[test]
    fn current_profile_matches_cfg() {
        let profile = parser_feature_profile_for_runtime();
        assert_eq!(profile.pure_rust, cfg!(feature = "pure-rust"));
        assert_eq!(
            profile.tree_sitter_standard,
            cfg!(feature = "tree-sitter-standard")
        );
        assert_eq!(
            profile.tree_sitter_c2rust,
            cfg!(feature = "tree-sitter-c2rust")
        );
        assert_eq!(profile.glr, cfg!(feature = "glr"));
    }

    #[test]
    fn status_line_is_stable_shape() {
        let status = bdd_status_line_for_current_profile(BddPhase::Runtime);
        let profile = parser_feature_profile_for_runtime();
        assert!(status.contains("runtime"));
        assert!(status.contains(&format!("{}", profile)));
    }
}
