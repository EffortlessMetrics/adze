//! Compatibility shim for the standalone runtime governance API.
//! Kept for backward-compatible public paths (`adze::parser_selection::*`).

pub use adze_governance_runtime_core::*;

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
