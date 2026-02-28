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
pub use adze_governance_runtime_reporting::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, BddScenario, BddScenarioStatus,
    GLR_CONFLICT_FALLBACK, GLR_CONFLICT_PRESERVATION_GRID, ParserBackend, ParserFeatureProfile,
    bdd_governance_snapshot, bdd_progress, bdd_progress_report, bdd_progress_report_with_profile,
    bdd_progress_report_with_profile_runtime, bdd_progress_status_line,
    describe_backend_for_conflicts,
};

/// Return the compile-time parser feature profile for the runtime crate.
pub const fn parser_feature_profile_for_runtime() -> ParserFeatureProfile {
    ParserFeatureProfile::current()
}

/// Return a parser profile equivalent to the runtime2 `pure-rust-glr` toggle.
pub const fn parser_feature_profile_for_runtime2(pure_rust_glr: bool) -> ParserFeatureProfile {
    ParserFeatureProfile {
        pure_rust: pure_rust_glr,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: pure_rust_glr,
    }
}

/// Resolve a backend using an explicit profile.
pub const fn resolve_backend_for_profile(
    profile: ParserFeatureProfile,
    has_conflicts: bool,
) -> ParserBackend {
    profile.resolve_backend(has_conflicts)
}

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
    fn runtime_profile_matches_current_cfg() {
        assert_eq!(
            parser_feature_profile_for_runtime().pure_rust,
            cfg!(feature = "pure-rust")
        );
    }

    #[test]
    fn runtime2_profile_reflects_glr_toggle() {
        let enabled = parser_feature_profile_for_runtime2(true);
        assert!(enabled.pure_rust);
        assert!(enabled.glr);
        assert!(!enabled.tree_sitter_standard);
        assert!(!enabled.tree_sitter_c2rust);

        let disabled = parser_feature_profile_for_runtime2(false);
        assert!(!disabled.pure_rust);
        assert!(!disabled.glr);
    }

    #[test]
    fn profile_backend_helper_matches_report_apis() {
        let profile = parser_feature_profile_for_runtime2(true);
        let report = bdd_progress_report_for_profile(BddPhase::Runtime, "Runtime", profile);
        let status = bdd_progress_status_line_for_profile(BddPhase::Runtime, profile);

        assert!(report.contains("Runtime"));
        assert!(status.contains("runtime:"));
    }
}
