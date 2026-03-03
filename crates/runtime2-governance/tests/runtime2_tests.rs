//! Integration tests for the runtime2-governance crate.

use adze_runtime2_governance::*;

#[test]
fn runtime2_profile_enabled_has_pure_rust_and_glr() {
    let profile = parser_feature_profile_for_runtime2(true);
    assert!(profile.pure_rust);
    assert!(profile.glr);
    assert!(!profile.tree_sitter_standard);
    assert!(!profile.tree_sitter_c2rust);
}

#[test]
fn runtime2_profile_disabled_has_no_flags() {
    let profile = parser_feature_profile_for_runtime2(false);
    assert!(!profile.pure_rust);
    assert!(!profile.glr);
}

#[test]
fn resolve_backend_glr_profile_returns_glr() {
    let profile = parser_feature_profile_for_runtime2(true);
    assert_eq!(
        resolve_backend_for_runtime2_profile(profile, true),
        ParserBackend::GLR
    );
}

#[test]
fn resolve_runtime2_backend_disabled_returns_tree_sitter() {
    assert_eq!(
        resolve_runtime2_backend(false, true),
        ParserBackend::TreeSitter
    );
}

#[test]
fn report_for_runtime2_profile_contains_title() {
    let profile = parser_feature_profile_for_runtime2(true);
    let report =
        bdd_progress_report_for_runtime2_profile(BddPhase::Runtime, "Runtime2 Test", profile);
    assert!(report.contains("Runtime2 Test"));
    assert!(report.contains("Governance status"));
}

#[test]
fn status_line_for_runtime2_profile_format() {
    let profile = parser_feature_profile_for_runtime2(true);
    let core = bdd_progress_status_line_for_runtime2_profile(BddPhase::Core, profile);
    let runtime = bdd_progress_status_line_for_runtime2_profile(BddPhase::Runtime, profile);
    assert!(core.starts_with("core:"));
    assert!(runtime.starts_with("runtime:"));
}

#[test]
fn runtime2_governance_snapshot_core_phase() {
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
    assert!(snap.total > 0);
    assert!(snap.implemented <= snap.total);
}
