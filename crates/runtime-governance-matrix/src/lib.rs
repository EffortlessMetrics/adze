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
pub use adze_governance_runtime2_core::{
    bdd_governance_matrix_for_runtime2_profile, bdd_progress_report_for_runtime2_profile,
    bdd_progress_status_line_for_runtime2_profile, resolve_backend_for_runtime2_profile,
    resolve_runtime2_backend, runtime2_governance_snapshot,
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
}
