//! Shared governance primitives for runtime profile selection and BDD reporting.
//!
//! This crate intentionally owns the profile composition helpers so that both
//! `runtime` and `runtime2` consumers can share the same behavior and fixture
//! wiring for BDD progress reporting.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

/// Re-exported governance reporting primitives (BDD grid, parser profiles, report helpers).
pub use adze_governance_runtime_profile_core::{
    ParserBackend, ParserFeatureProfile, parser_feature_profile_for_runtime,
    parser_feature_profile_for_runtime2, resolve_backend_for_profile,
};
pub use adze_governance_runtime_reporting::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, BddScenario, BddScenarioStatus,
    GLR_CONFLICT_FALLBACK, GLR_CONFLICT_PRESERVATION_GRID, bdd_governance_snapshot, bdd_progress,
    bdd_progress_report, bdd_progress_report_with_profile,
    bdd_progress_report_with_profile_runtime, bdd_progress_status_line,
    describe_backend_for_conflicts,
};

/// Build a profile-specific governance report against the canonical GLR scenario grid.
pub fn bdd_progress_report_for_profile(
    phase: BddPhase,
    phase_title: &str,
    profile: ParserFeatureProfile,
) -> String {
    BddGovernanceMatrix::new(phase, profile, GLR_CONFLICT_PRESERVATION_GRID).report(phase_title)
}

/// Build a profile-specific governance matrix for the canonical GLR scenario grid.
pub const fn bdd_governance_matrix_for_profile(
    phase: BddPhase,
    profile: ParserFeatureProfile,
) -> BddGovernanceMatrix {
    BddGovernanceMatrix::new(phase, profile, GLR_CONFLICT_PRESERVATION_GRID)
}

/// Build the active runtime governance matrix from the compiled-in profile.
pub const fn bdd_governance_matrix_for_runtime() -> BddGovernanceMatrix {
    bdd_governance_matrix_for_profile(BddPhase::Runtime, parser_feature_profile_for_runtime())
}

/// Build a runtime2 governance matrix for an explicit `pure-rust-glr` toggle.
pub const fn bdd_governance_matrix_for_runtime2(
    phase: BddPhase,
    pure_rust_glr: bool,
) -> BddGovernanceMatrix {
    bdd_governance_matrix_for_profile(phase, parser_feature_profile_for_runtime2(pure_rust_glr))
}

/// Build a profile-specific governance status line against the canonical GLR grid.
pub fn bdd_progress_status_line_for_profile(
    phase: BddPhase,
    profile: ParserFeatureProfile,
) -> String {
    bdd_governance_matrix_for_profile(phase, profile).status_line()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_backend_helper_matches_report_apis() {
        let profile = parser_feature_profile_for_runtime2(true);
        let report = bdd_progress_report_for_profile(BddPhase::Runtime, "Runtime", profile);
        let status = bdd_progress_status_line_for_profile(BddPhase::Runtime, profile);

        assert!(report.contains("Runtime"));
        assert!(status.contains("runtime:"));
    }

    #[test]
    fn bdd_governance_matrix_for_runtime_uses_runtime_phase() {
        let matrix = bdd_governance_matrix_for_runtime();
        assert_eq!(matrix.phase, BddPhase::Runtime);
        assert_eq!(matrix.profile, parser_feature_profile_for_runtime());
        assert!(!matrix.scenarios.is_empty());
    }

    #[test]
    fn bdd_governance_matrix_for_runtime2_both_toggles() {
        let enabled = bdd_governance_matrix_for_runtime2(BddPhase::Core, true);
        assert_eq!(enabled.phase, BddPhase::Core);
        assert!(enabled.profile.glr);
        assert!(enabled.profile.pure_rust);

        let disabled = bdd_governance_matrix_for_runtime2(BddPhase::Runtime, false);
        assert_eq!(disabled.phase, BddPhase::Runtime);
        assert!(!disabled.profile.glr);
        assert!(!disabled.profile.pure_rust);
    }

    #[test]
    fn bdd_governance_matrix_for_profile_with_both_phases() {
        let profile = ParserFeatureProfile::current();
        let core = bdd_governance_matrix_for_profile(BddPhase::Core, profile);
        let runtime = bdd_governance_matrix_for_profile(BddPhase::Runtime, profile);
        assert_eq!(core.phase, BddPhase::Core);
        assert_eq!(runtime.phase, BddPhase::Runtime);
        assert_eq!(core.profile, runtime.profile);
    }

    #[test]
    fn bdd_progress_status_line_for_profile_format() {
        let profile = parser_feature_profile_for_runtime2(false);
        let core_status = bdd_progress_status_line_for_profile(BddPhase::Core, profile);
        let runtime_status = bdd_progress_status_line_for_profile(BddPhase::Runtime, profile);
        assert!(core_status.starts_with("core:"));
        assert!(runtime_status.starts_with("runtime:"));
    }

    #[test]
    fn bdd_progress_report_for_profile_includes_title() {
        let profile = parser_feature_profile_for_runtime2(true);
        let report = bdd_progress_report_for_profile(BddPhase::Core, "Core GLR", profile);
        assert!(report.contains("Core GLR"));
    }

    #[test]
    fn runtime2_profile_tree_sitter_flags_are_off() {
        let profile = parser_feature_profile_for_runtime2(true);
        assert!(!profile.tree_sitter_standard);
        assert!(!profile.tree_sitter_c2rust);

        let profile_off = parser_feature_profile_for_runtime2(false);
        assert!(!profile_off.tree_sitter_standard);
        assert!(!profile_off.tree_sitter_c2rust);
    }
}
