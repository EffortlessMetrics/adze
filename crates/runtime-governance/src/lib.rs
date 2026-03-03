//! Runtime-facing governance façade over the shared governance-matrix crate.
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
    bdd_governance_matrix_for_current_profile, bdd_governance_matrix_for_profile,
    bdd_governance_matrix_for_runtime, bdd_governance_matrix_for_runtime2,
    bdd_governance_matrix_for_runtime2_profile, bdd_governance_snapshot, bdd_progress,
    bdd_progress_report, bdd_progress_report_for_current_profile, bdd_progress_report_for_profile,
    bdd_progress_report_with_profile, bdd_progress_report_with_profile_runtime,
    bdd_progress_status_line, bdd_progress_status_line_for_profile,
    bdd_status_line_for_current_profile, current_backend_for, describe_backend_for_conflicts,
    parser_feature_profile_for_runtime, resolve_backend_for_profile, runtime_governance_snapshot,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_matches_contract_current() {
        assert_eq!(
            parser_feature_profile_for_runtime().pure_rust,
            cfg!(feature = "pure-rust")
        );
    }

    #[test]
    fn resolve_backend_round_trips_current_profile() {
        let profile = parser_feature_profile_for_runtime();
        assert_eq!(current_backend_for(false), profile.resolve_backend(false));

        #[cfg(feature = "glr")]
        {
            assert_eq!(current_backend_for(true), profile.resolve_backend(true));
        }
    }

    #[test]
    fn reports_current_profile() {
        let report = bdd_progress_report_for_current_profile(BddPhase::Core, "Core");
        assert!(report.contains("Core"));
        assert!(report.contains("Feature profile:"));
        assert!(report.contains("Governance status"));
    }

    #[test]
    fn status_line_is_stable_shape() {
        let status = bdd_status_line_for_current_profile(BddPhase::Runtime);
        assert!(status.starts_with("runtime:"));
    }

    #[test]
    fn matrix_helpers_are_exposed_from_runtime_governance_api() {
        let profile = parser_feature_profile_for_runtime();
        let matrix = bdd_governance_matrix_for_current_profile(BddPhase::Core);
        let explicit = bdd_governance_matrix_for_profile(BddPhase::Core, profile);
        let runtime_matrix = bdd_governance_matrix_for_runtime();

        assert_eq!(matrix.profile, profile);
        assert_eq!(explicit.profile, profile);
        assert_eq!(runtime_matrix.profile, profile);

        let runtime2_matrix = bdd_governance_matrix_for_runtime2(BddPhase::Core, profile.glr);
        let runtime2_profile =
            adze_runtime_governance_matrix::parser_feature_profile_for_runtime2(profile.glr);
        assert_eq!(runtime2_matrix.profile, runtime2_profile);
    }

    #[test]
    fn resolve_backend_for_profile_works() {
        let profile = parser_feature_profile_for_runtime();
        let backend = resolve_backend_for_profile(profile, false);
        assert_eq!(backend, profile.resolve_backend(false));
    }

    #[test]
    fn bdd_governance_matrix_for_runtime2_works() {
        let profile = parser_feature_profile_for_runtime();
        let matrix = bdd_governance_matrix_for_runtime2(BddPhase::Runtime, profile.glr);
        assert_eq!(matrix.phase, BddPhase::Runtime);
    }

    #[test]
    fn runtime_governance_snapshot_accessible() {
        let snap = runtime_governance_snapshot(BddPhase::Core);
        assert_eq!(snap.phase, BddPhase::Core);
        assert_eq!(snap.profile, parser_feature_profile_for_runtime());
    }

    #[test]
    fn bdd_progress_report_for_current_profile_both_phases() {
        let core = bdd_progress_report_for_current_profile(BddPhase::Core, "Core Phase");
        let runtime = bdd_progress_report_for_current_profile(BddPhase::Runtime, "Runtime Phase");
        assert!(core.contains("Core Phase"));
        assert!(runtime.contains("Runtime Phase"));
    }

    #[test]
    fn bdd_status_line_for_current_profile_both_phases() {
        let core = bdd_status_line_for_current_profile(BddPhase::Core);
        let runtime = bdd_status_line_for_current_profile(BddPhase::Runtime);
        assert!(core.starts_with("core:"));
        assert!(runtime.starts_with("runtime:"));
    }

    #[test]
    fn bdd_governance_matrix_for_runtime_uses_current_profile() {
        let matrix = bdd_governance_matrix_for_runtime();
        assert_eq!(matrix.profile, parser_feature_profile_for_runtime());
    }
}
