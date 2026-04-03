//! BDD-style tests for governance-runtime-core crate.
//!
//! Tests follow the Given/When/Then pattern to verify public API behavior.

use adze_governance_runtime_core::{
    BddPhase, ParserBackend, ParserFeatureProfile, bdd_governance_matrix_for_profile,
    bdd_governance_matrix_for_runtime, bdd_governance_matrix_for_runtime2,
    bdd_progress_report_for_profile, bdd_progress_status_line_for_profile,
    parser_feature_profile_for_runtime, parser_feature_profile_for_runtime2,
    resolve_backend_for_profile,
};

#[test]
fn given_runtime_profile_when_calling_for_runtime_then_returns_current_cfg() {
    // Given / When
    let profile = parser_feature_profile_for_runtime();

    // Then
    assert_eq!(profile.pure_rust, cfg!(feature = "pure-rust"));
}

#[test]
fn given_enabled_flag_when_creating_runtime2_profile_then_has_pure_rust_and_glr() {
    // Given / When
    let profile = parser_feature_profile_for_runtime2(true);

    // Then
    assert!(profile.pure_rust);
    assert!(profile.glr);
    assert!(!profile.tree_sitter_standard);
    assert!(!profile.tree_sitter_c2rust);
}

#[test]
fn given_disabled_flag_when_creating_runtime2_profile_then_has_no_pure_rust_or_glr() {
    // Given / When
    let profile = parser_feature_profile_for_runtime2(false);

    // Then
    assert!(!profile.pure_rust);
    assert!(!profile.glr);
    assert!(!profile.tree_sitter_standard);
    assert!(!profile.tree_sitter_c2rust);
}

#[test]
fn given_glr_profile_without_conflict_when_resolving_backend_then_returns_glr() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let backend = resolve_backend_for_profile(profile, false);

    // Then
    assert_eq!(backend, ParserBackend::GLR);
}

#[test]
fn given_glr_profile_with_conflict_when_resolving_backend_then_returns_glr() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let backend = resolve_backend_for_profile(profile, true);

    // Then
    assert_eq!(backend, ParserBackend::GLR);
}

#[test]
fn given_non_glr_profile_with_conflict_when_resolving_backend_then_returns_tree_sitter() {
    // Given
    let profile = parser_feature_profile_for_runtime2(false);

    // When
    let backend = resolve_backend_for_profile(profile, true);

    // Then
    assert_eq!(backend, ParserBackend::TreeSitter);
}

#[test]
fn given_non_glr_profile_without_conflict_when_resolving_backend_then_returns_tree_sitter() {
    // Given
    let profile = parser_feature_profile_for_runtime2(false);

    // When
    let backend = resolve_backend_for_profile(profile, false);

    // Then
    assert_eq!(backend, ParserBackend::TreeSitter);
}

#[test]
fn given_profile_when_calling_resolve_backend_then_delegates_correctly() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let backend = resolve_backend_for_profile(profile, false);
    let expected = profile.resolve_backend(false);

    // Then
    assert_eq!(backend, expected);
}

#[test]
fn given_core_phase_when_creating_report_then_contains_title() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);
    let title = "Core Report";

    // When
    let report = bdd_progress_report_for_profile(BddPhase::Core, title, profile);

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_runtime_phase_when_creating_report_then_contains_title() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);
    let title = "Runtime Report";

    // When
    let report = bdd_progress_report_for_profile(BddPhase::Runtime, title, profile);

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_core_phase_when_creating_status_line_then_starts_with_core() {
    // Given
    let profile = parser_feature_profile_for_runtime2(false);

    // When
    let status = bdd_progress_status_line_for_profile(BddPhase::Core, profile);

    // Then
    assert!(status.starts_with("core:"));
}

#[test]
fn given_runtime_phase_when_creating_status_line_then_starts_with_runtime() {
    // Given
    let profile = parser_feature_profile_for_runtime2(false);

    // When
    let status = bdd_progress_status_line_for_profile(BddPhase::Runtime, profile);

    // Then
    assert!(status.starts_with("runtime:"));
}

#[test]
fn given_runtime_matrix_when_calling_for_runtime_then_uses_runtime_phase() {
    // Given / When
    let matrix = bdd_governance_matrix_for_runtime();

    // Then
    assert_eq!(matrix.phase, BddPhase::Runtime);
    assert_eq!(matrix.profile, parser_feature_profile_for_runtime());
    assert!(!matrix.scenarios.is_empty());
}

#[test]
fn given_enabled_flag_when_creating_runtime2_matrix_then_has_glr() {
    // Given / When
    let matrix = bdd_governance_matrix_for_runtime2(BddPhase::Core, true);

    // Then
    assert_eq!(matrix.phase, BddPhase::Core);
    assert!(matrix.profile.glr);
    assert!(matrix.profile.pure_rust);
}

#[test]
fn given_disabled_flag_when_creating_runtime2_matrix_then_has_no_glr() {
    // Given / When
    let matrix = bdd_governance_matrix_for_runtime2(BddPhase::Runtime, false);

    // Then
    assert_eq!(matrix.phase, BddPhase::Runtime);
    assert!(!matrix.profile.glr);
    assert!(!matrix.profile.pure_rust);
}

#[test]
fn given_core_phase_when_creating_matrix_for_profile_then_phase_is_core() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let matrix = bdd_governance_matrix_for_profile(BddPhase::Core, profile);

    // Then
    assert_eq!(matrix.phase, BddPhase::Core);
    assert_eq!(matrix.profile, profile);
}

#[test]
fn given_runtime_phase_when_creating_matrix_for_profile_then_phase_is_runtime() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let matrix = bdd_governance_matrix_for_profile(BddPhase::Runtime, profile);

    // Then
    assert_eq!(matrix.phase, BddPhase::Runtime);
    assert_eq!(matrix.profile, profile);
}

#[test]
fn given_matrix_when_check_scenarios_then_has_scenarios() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let matrix = bdd_governance_matrix_for_profile(BddPhase::Core, profile);

    // Then
    assert!(!matrix.scenarios.is_empty());
}

#[test]
fn given_runtime2_profile_when_checking_tree_sitter_flags_then_are_off() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // Then
    assert!(!profile.tree_sitter_standard);
    assert!(!profile.tree_sitter_c2rust);

    // Also check with disabled
    let profile_off = parser_feature_profile_for_runtime2(false);
    assert!(!profile_off.tree_sitter_standard);
    assert!(!profile_off.tree_sitter_c2rust);
}

#[test]
fn given_profile_and_title_when_creating_report_then_contains_profile_info() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);
    let title = "Profile Test";

    // When
    let report = bdd_progress_report_for_profile(BddPhase::Core, title, profile);

    // Then
    assert!(report.contains(title));
}
