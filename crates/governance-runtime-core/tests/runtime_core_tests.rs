//! Integration tests for the governance-runtime-core crate.

use adze_governance_runtime_core::*;

#[test]
fn runtime_profile_pure_rust_matches_cfg() {
    let profile = parser_feature_profile_for_runtime();
    assert_eq!(profile.pure_rust, cfg!(feature = "pure-rust"));
}

#[test]
fn runtime2_profile_enabled_sets_glr_and_pure_rust() {
    let profile = parser_feature_profile_for_runtime2(true);
    assert!(profile.pure_rust);
    assert!(profile.glr);
    assert!(!profile.tree_sitter_standard);
    assert!(!profile.tree_sitter_c2rust);
}

#[test]
fn runtime2_profile_disabled_clears_all_flags() {
    let profile = parser_feature_profile_for_runtime2(false);
    assert!(!profile.pure_rust);
    assert!(!profile.glr);
}

#[test]
fn resolve_backend_delegates_to_profile() {
    let profile = parser_feature_profile_for_runtime2(true);
    let backend = resolve_backend_for_profile(profile, false);
    assert_eq!(backend, profile.resolve_backend(false));
}

#[test]
fn report_for_profile_contains_title() {
    let profile = parser_feature_profile_for_runtime2(true);
    let report = bdd_progress_report_for_profile(BddPhase::Core, "Core GLR", profile);
    assert!(report.contains("Core GLR"));
}

#[test]
fn matrix_for_runtime_uses_current_profile() {
    let matrix = bdd_governance_matrix_for_runtime();
    assert_eq!(matrix.profile, parser_feature_profile_for_runtime());
    assert_eq!(matrix.phase, BddPhase::Runtime);
}

#[test]
fn matrix_for_runtime2_respects_toggle() {
    let enabled = bdd_governance_matrix_for_runtime2(BddPhase::Core, true);
    assert!(enabled.profile.glr);
    assert_eq!(enabled.phase, BddPhase::Core);

    let disabled = bdd_governance_matrix_for_runtime2(BddPhase::Runtime, false);
    assert!(!disabled.profile.glr);
    assert_eq!(disabled.phase, BddPhase::Runtime);
}

#[test]
fn status_line_for_profile_starts_with_phase() {
    let profile = parser_feature_profile_for_runtime2(false);
    let core = bdd_progress_status_line_for_profile(BddPhase::Core, profile);
    let runtime = bdd_progress_status_line_for_profile(BddPhase::Runtime, profile);
    assert!(core.starts_with("core:"));
    assert!(runtime.starts_with("runtime:"));
}
