//! Runtime2-specific governance profile and reporting primitives.
//!
//! This crate isolates runtime2 profile translation helpers and runtime2-focused
//! BDD wiring so façade crates can keep runtime2 APIs stable without carrying
//! runtime profile concerns.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_governance_runtime_core::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, BddScenario, BddScenarioStatus,
    GLR_CONFLICT_FALLBACK, GLR_CONFLICT_PRESERVATION_GRID, ParserBackend, ParserFeatureProfile,
    bdd_governance_matrix_for_runtime2, bdd_governance_snapshot, bdd_progress, bdd_progress_report,
    bdd_progress_report_for_profile, bdd_progress_report_with_profile,
    bdd_progress_report_with_profile_runtime, bdd_progress_status_line,
    bdd_progress_status_line_for_profile, describe_backend_for_conflicts,
    parser_feature_profile_for_runtime2, resolve_backend_for_profile,
};

/// Build a governance matrix for a runtime2-compatible profile.
pub fn bdd_governance_matrix_for_runtime2_profile(
    phase: BddPhase,
    pure_rust_glr: bool,
) -> BddGovernanceMatrix {
    bdd_governance_matrix_for_runtime2(phase, pure_rust_glr)
}

/// Build a BDD report for an explicit runtime2 profile.
pub fn bdd_progress_report_for_runtime2_profile(
    phase: BddPhase,
    phase_title: &str,
    profile: ParserFeatureProfile,
) -> String {
    bdd_progress_report_with_profile_runtime(
        phase,
        GLR_CONFLICT_PRESERVATION_GRID,
        phase_title,
        profile,
    )
}

/// Build a BDD status line for an explicit runtime2 profile.
pub fn bdd_progress_status_line_for_runtime2_profile(
    phase: BddPhase,
    profile: ParserFeatureProfile,
) -> String {
    bdd_progress_status_line_for_profile(phase, profile)
}

/// Resolve runtime2 backend resolution from an explicit profile.
pub const fn resolve_backend_for_runtime2_profile(
    profile: ParserFeatureProfile,
    has_conflicts: bool,
) -> ParserBackend {
    resolve_backend_for_profile(profile, has_conflicts)
}

/// Resolve runtime2 backend resolution directly from the `pure-rust-glr` toggle.
pub const fn resolve_runtime2_backend(pure_rust_glr: bool, has_conflicts: bool) -> ParserBackend {
    resolve_backend_for_profile(
        parser_feature_profile_for_runtime2(pure_rust_glr),
        has_conflicts,
    )
}

/// Build a runtime2 governance snapshot for an explicit profile.
pub fn runtime2_governance_snapshot(
    phase: BddPhase,
    profile: ParserFeatureProfile,
) -> BddGovernanceSnapshot {
    bdd_governance_snapshot(phase, GLR_CONFLICT_PRESERVATION_GRID, profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_runtime2_helpers_are_consistent() {
        let profile = parser_feature_profile_for_runtime2(true);
        let baseline = parser_feature_profile_for_runtime2(false);
        assert_eq!(
            resolve_runtime2_backend(false, true),
            baseline.resolve_backend(true),
            "runtime2 helpers should align with explicit profile resolution"
        );

        let report =
            bdd_progress_report_for_runtime2_profile(BddPhase::Runtime, "Runtime2", profile);
        let status = bdd_progress_status_line_for_runtime2_profile(BddPhase::Runtime, profile);
        let snapshot = runtime2_governance_snapshot(BddPhase::Runtime, profile);
        assert!(report.contains("Runtime2"));
        assert!(status.starts_with("runtime:"));
        assert_eq!(snapshot.profile, profile);
    }

    #[test]
    fn matrix_helpers_are_reachable_from_runtime2_core_crate() {
        let runtime2_matrix = bdd_governance_matrix_for_runtime2_profile(BddPhase::Runtime, false);
        assert_eq!(
            runtime2_matrix.profile,
            parser_feature_profile_for_runtime2(false)
        );
    }

    #[test]
    fn resolve_backend_for_runtime2_profile_works() {
        let profile = parser_feature_profile_for_runtime2(false);
        let backend = resolve_backend_for_runtime2_profile(profile, false);
        assert_eq!(backend, profile.resolve_backend(false));
    }
}
