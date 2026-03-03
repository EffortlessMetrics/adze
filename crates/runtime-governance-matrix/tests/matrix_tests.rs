//! Integration tests for the runtime-governance-matrix crate.

use adze_runtime_governance_matrix::*;

#[test]
fn current_backend_matches_profile_resolution() {
    let profile = parser_feature_profile_for_runtime();
    assert_eq!(current_backend_for(false), profile.resolve_backend(false));
}

#[test]
fn report_for_current_profile_core_contains_title() {
    let report = bdd_progress_report_for_current_profile(BddPhase::Core, "Core Phase");
    assert!(report.contains("Core Phase"));
    assert!(report.contains("Governance status"));
}

#[test]
fn report_for_current_profile_runtime_contains_title() {
    let report = bdd_progress_report_for_current_profile(BddPhase::Runtime, "Runtime Phase");
    assert!(report.contains("Runtime Phase"));
}

#[test]
fn matrix_for_current_profile_uses_runtime_profile() {
    let matrix = bdd_governance_matrix_for_current_profile(BddPhase::Core);
    assert_eq!(matrix.profile, parser_feature_profile_for_runtime());
    assert_eq!(matrix.phase, BddPhase::Core);
}

#[test]
fn runtime2_profile_constructor() {
    let matrix = bdd_governance_matrix_for_runtime2_profile(BddPhase::Core, true);
    assert!(matrix.profile.glr);
    assert_eq!(matrix.phase, BddPhase::Core);

    let off = bdd_governance_matrix_for_runtime2_profile(BddPhase::Runtime, false);
    assert!(!off.profile.glr);
}

#[test]
fn status_line_for_current_profile_format() {
    let core = bdd_status_line_for_current_profile(BddPhase::Core);
    let runtime = bdd_status_line_for_current_profile(BddPhase::Runtime);
    assert!(core.starts_with("core:"));
    assert!(runtime.starts_with("runtime:"));
}

#[test]
fn runtime_governance_snapshot_phase_and_profile() {
    let snap = runtime_governance_snapshot(BddPhase::Runtime);
    assert_eq!(snap.phase, BddPhase::Runtime);
    assert_eq!(snap.profile, parser_feature_profile_for_runtime());
}

#[test]
fn resolve_backend_for_profile_consistent() {
    let profile = parser_feature_profile_for_runtime();
    let backend = resolve_backend_for_profile(profile, false);
    assert_eq!(backend, profile.resolve_backend(false));
}
