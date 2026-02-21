//! Runtime2-specific governance façade over the shared governance-matrix crate.
//!
//! This crate preserves historical API shape while delegating implementation to
//! `adze-runtime-governance-matrix`.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_runtime_governance_matrix::{
    BddGovernanceSnapshot, BddPhase, BddScenario, BddScenarioStatus, GLR_CONFLICT_FALLBACK,
    GLR_CONFLICT_PRESERVATION_GRID, ParserBackend, ParserFeatureProfile,
    bdd_governance_matrix_for_profile, bdd_governance_matrix_for_runtime2,
    bdd_governance_matrix_for_runtime2_profile, bdd_governance_snapshot, bdd_progress,
    bdd_progress_report, bdd_progress_report_for_profile, bdd_progress_report_for_runtime2_profile,
    bdd_progress_report_with_profile, bdd_progress_status_line,
    bdd_progress_status_line_for_profile, bdd_progress_status_line_for_runtime2_profile,
    describe_backend_for_conflicts, parser_feature_profile_for_runtime2,
    resolve_backend_for_profile, resolve_backend_for_runtime2_profile, resolve_runtime2_backend,
    runtime2_governance_snapshot,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime2_profile_maps_pure_rust_glr_toggle() {
        let profile = parser_feature_profile_for_runtime2(true);
        assert!(profile.pure_rust);
        assert!(profile.glr);
        assert!(!profile.tree_sitter_standard);
        assert!(!profile.tree_sitter_c2rust);

        let fallback = parser_feature_profile_for_runtime2(false);
        assert!(!fallback.pure_rust);
        assert!(!fallback.glr);
    }

    #[test]
    fn runtime2_backend_for_conflicts_respects_profile() {
        let profile = parser_feature_profile_for_runtime2(true);
        assert_eq!(
            resolve_backend_for_runtime2_profile(profile, true),
            ParserBackend::GLR
        );
        assert_eq!(
            resolve_runtime2_backend(false, true),
            ParserBackend::TreeSitter
        );
    }

    #[test]
    fn runtime2_report_with_explicit_profile_renders_expected_shape() {
        let profile = parser_feature_profile_for_runtime2(true);
        let report =
            bdd_progress_report_for_runtime2_profile(BddPhase::Runtime, "Runtime2", profile);
        assert!(report.contains("Runtime2"));
        assert!(report.contains("Feature profile:"));
        assert!(report.contains("Governance status"));
    }

    #[test]
    fn runtime2_status_line_is_stable_shape() {
        let profile = parser_feature_profile_for_runtime2(false);
        let status = bdd_progress_status_line_for_runtime2_profile(BddPhase::Runtime, profile);
        assert!(status.contains("runtime:"));
    }
}
