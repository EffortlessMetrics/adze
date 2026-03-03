//! Shared governance profile and reporting primitives for runtime and runtime2 consumers.
//!
//! This crate consolidates parser-feature profile helpers, backend resolution, and
//! BDD reporting wiring around the canonical GLR preservation grid. It is intended
//! to be used by façade crates that keep historical public APIs stable.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_governance_runtime_core::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, BddScenario, BddScenarioStatus,
    GLR_CONFLICT_FALLBACK, GLR_CONFLICT_PRESERVATION_GRID, ParserBackend, ParserFeatureProfile,
    bdd_governance_matrix_for_profile, bdd_governance_matrix_for_runtime,
    bdd_governance_matrix_for_runtime2, bdd_governance_snapshot, bdd_progress, bdd_progress_report,
    bdd_progress_report_for_profile, bdd_progress_report_with_profile,
    bdd_progress_report_with_profile_runtime, bdd_progress_status_line,
    bdd_progress_status_line_for_profile, describe_backend_for_conflicts,
    parser_feature_profile_for_runtime, parser_feature_profile_for_runtime2,
    resolve_backend_for_profile,
};

/// Select the parser backend for the current compile-time feature profile.
pub const fn current_backend_for(has_conflicts: bool) -> ParserBackend {
    ParserBackend::select(has_conflicts)
}

/// Return a BDD progress report for the active runtime profile.
pub fn bdd_progress_report_for_current_profile(phase: BddPhase, phase_title: &str) -> String {
    bdd_progress_report_with_profile_runtime(
        phase,
        GLR_CONFLICT_PRESERVATION_GRID,
        phase_title,
        parser_feature_profile_for_runtime(),
    )
}

/// Build the active runtime governance matrix for a phase.
pub fn bdd_governance_matrix_for_current_profile(phase: BddPhase) -> BddGovernanceMatrix {
    bdd_governance_matrix_for_profile(phase, parser_feature_profile_for_runtime())
}

/// Build a governance matrix for a runtime2-compatible profile.
pub fn bdd_governance_matrix_for_runtime2_profile(
    phase: BddPhase,
    pure_rust_glr: bool,
) -> BddGovernanceMatrix {
    bdd_governance_matrix_for_runtime2(phase, pure_rust_glr)
}

/// Return a BDD status line for the active runtime profile.
pub fn bdd_status_line_for_current_profile(phase: BddPhase) -> String {
    bdd_progress_status_line_for_profile(phase, parser_feature_profile_for_runtime())
}

/// Build a governance snapshot for the active runtime profile.
pub fn runtime_governance_snapshot(phase: BddPhase) -> BddGovernanceSnapshot {
    bdd_governance_snapshot(
        phase,
        GLR_CONFLICT_PRESERVATION_GRID,
        parser_feature_profile_for_runtime(),
    )
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
    fn resolve_backend_round_trips_current_profile() {
        let profile = parser_feature_profile_for_runtime();
        assert_eq!(current_backend_for(false), profile.resolve_backend(false));

        #[cfg(feature = "glr")]
        {
            assert_eq!(current_backend_for(true), profile.resolve_backend(true));
        }
    }

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
    fn matrix_helpers_are_reachable_from_runtime_matrix_crate() {
        let profile = parser_feature_profile_for_runtime();
        let runtime_matrix = bdd_governance_matrix_for_current_profile(BddPhase::Core);
        assert_eq!(runtime_matrix.profile, profile);
        assert_eq!(runtime_matrix.phase, BddPhase::Core);

        let runtime2_matrix = bdd_governance_matrix_for_runtime2_profile(BddPhase::Runtime, false);
        assert_eq!(
            runtime2_matrix.profile,
            parser_feature_profile_for_runtime2(false)
        );
    }

    #[test]
    fn reports_and_status_for_current_profile() {
        let report = bdd_progress_report_for_current_profile(BddPhase::Core, "Core");
        let status = bdd_status_line_for_current_profile(BddPhase::Runtime);
        let snapshot = runtime_governance_snapshot(BddPhase::Runtime);
        let current = parser_feature_profile_for_runtime();
        assert!(report.contains("Core"));
        assert!(report.contains("Feature profile:"));
        assert!(report.contains("Governance status"));
        assert!(status.starts_with("runtime:"));
        assert_eq!(snapshot.profile, current);
    }

    #[test]
    fn resolve_backend_for_runtime2_profile_works() {
        let profile = parser_feature_profile_for_runtime2(false);
        let backend = resolve_backend_for_runtime2_profile(profile, false);
        assert_eq!(backend, profile.resolve_backend(false));
    }

    #[test]
    fn runtime2_governance_snapshot_consistency() {
        let profile = parser_feature_profile_for_runtime2(true);
        let snap = runtime2_governance_snapshot(BddPhase::Core, profile);
        assert_eq!(snap.phase, BddPhase::Core);
        assert_eq!(snap.profile, profile);
    }

    #[test]
    fn runtime2_governance_snapshot_runtime_phase() {
        let profile = parser_feature_profile_for_runtime2(false);
        let snap = runtime2_governance_snapshot(BddPhase::Runtime, profile);
        assert_eq!(snap.phase, BddPhase::Runtime);
        assert_eq!(snap.profile, profile);
    }

    #[test]
    fn resolve_runtime2_backend_consistent_with_profile() {
        let backend_enabled = resolve_runtime2_backend(true, false);
        let profile = parser_feature_profile_for_runtime2(true);
        assert_eq!(backend_enabled, profile.resolve_backend(false));

        let backend_disabled = resolve_runtime2_backend(false, false);
        let profile_off = parser_feature_profile_for_runtime2(false);
        assert_eq!(backend_disabled, profile_off.resolve_backend(false));
    }

    #[test]
    fn resolve_runtime2_backend_with_conflicts() {
        let backend = resolve_runtime2_backend(true, true);
        let profile = parser_feature_profile_for_runtime2(true);
        assert_eq!(backend, profile.resolve_backend(true));
    }

    #[test]
    fn current_backend_for_no_conflicts() {
        let backend = current_backend_for(false);
        let profile = parser_feature_profile_for_runtime();
        assert_eq!(backend, profile.resolve_backend(false));
    }

    #[test]
    fn bdd_progress_report_for_runtime2_profile_format() {
        let profile = parser_feature_profile_for_runtime2(true);
        let report =
            bdd_progress_report_for_runtime2_profile(BddPhase::Runtime, "RT2 Report", profile);
        assert!(report.contains("RT2 Report"));
        assert!(report.contains("Governance status"));
    }

    #[test]
    fn bdd_progress_status_line_for_runtime2_profile_format() {
        let profile = parser_feature_profile_for_runtime2(true);
        let core = bdd_progress_status_line_for_runtime2_profile(BddPhase::Core, profile);
        let runtime = bdd_progress_status_line_for_runtime2_profile(BddPhase::Runtime, profile);
        assert!(core.starts_with("core:"));
        assert!(runtime.starts_with("runtime:"));
    }

    #[test]
    fn bdd_governance_matrix_for_runtime2_profile_constructor() {
        let matrix = bdd_governance_matrix_for_runtime2_profile(BddPhase::Core, true);
        assert_eq!(matrix.phase, BddPhase::Core);
        assert!(matrix.profile.glr);

        let matrix_off = bdd_governance_matrix_for_runtime2_profile(BddPhase::Runtime, false);
        assert_eq!(matrix_off.phase, BddPhase::Runtime);
        assert!(!matrix_off.profile.glr);
    }

    #[test]
    fn runtime_governance_snapshot_matches_current_profile() {
        let snap = runtime_governance_snapshot(BddPhase::Runtime);
        assert_eq!(snap.phase, BddPhase::Runtime);
        assert_eq!(snap.profile, parser_feature_profile_for_runtime());
    }
}
