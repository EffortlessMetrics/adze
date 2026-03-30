//! Comprehensive BDD-style tests for the runtime2-governance crate.

use adze_runtime2_governance::*;

// ---------------------------------------------------------------------------
// parser_feature_profile_for_runtime2 Tests
// ---------------------------------------------------------------------------

#[test]
fn given_enabled_flag_when_creating_profile_then_has_pure_rust_and_glr() {
    // Given / When
    let profile = parser_feature_profile_for_runtime2(true);

    // Then
    assert!(profile.pure_rust);
    assert!(profile.glr);
    assert!(!profile.tree_sitter_standard);
    assert!(!profile.tree_sitter_c2rust);
}

#[test]
fn given_disabled_flag_when_creating_profile_then_has_no_pure_rust_or_glr() {
    // Given / When
    let profile = parser_feature_profile_for_runtime2(false);

    // Then
    assert!(!profile.pure_rust);
    assert!(!profile.glr);
}

#[test]
fn given_enabled_profile_when_displaying_then_is_non_empty() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let display = format!("{}", profile);

    // Then
    assert!(!display.is_empty());
}

// ---------------------------------------------------------------------------
// resolve_backend_for_runtime2_profile Tests
// ---------------------------------------------------------------------------

#[test]
fn given_glr_profile_with_conflict_when_resolving_backend_then_returns_glr() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let backend = resolve_backend_for_runtime2_profile(profile, true);

    // Then
    assert_eq!(backend, ParserBackend::GLR);
}

#[test]
fn given_glr_profile_without_conflict_when_resolving_backend_then_returns_glr() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let backend = resolve_backend_for_runtime2_profile(profile, false);

    // Then
    assert_eq!(backend, ParserBackend::GLR);
}

#[test]
fn given_non_glr_profile_with_conflict_when_resolving_backend_then_returns_tree_sitter() {
    // Given
    let profile = parser_feature_profile_for_runtime2(false);

    // When
    let backend = resolve_backend_for_runtime2_profile(profile, true);

    // Then
    assert_eq!(backend, ParserBackend::TreeSitter);
}

// ---------------------------------------------------------------------------
// resolve_runtime2_backend Tests
// ---------------------------------------------------------------------------

#[test]
fn given_enabled_with_conflict_when_resolving_backend_then_returns_glr() {
    // Given / When
    let backend = resolve_runtime2_backend(true, true);

    // Then
    assert_eq!(backend, ParserBackend::GLR);
}

#[test]
fn given_disabled_with_conflict_when_resolving_backend_then_returns_tree_sitter() {
    // Given / When
    let backend = resolve_runtime2_backend(false, true);

    // Then
    assert_eq!(backend, ParserBackend::TreeSitter);
}

#[test]
fn given_enabled_without_conflict_when_resolving_backend_then_returns_glr() {
    // Given / When
    let backend = resolve_runtime2_backend(true, false);

    // Then
    assert_eq!(backend, ParserBackend::GLR);
}

// ---------------------------------------------------------------------------
// bdd_progress_report_for_runtime2_profile Tests
// ---------------------------------------------------------------------------

#[test]
fn given_runtime_phase_when_generating_report_then_contains_title() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);
    let title = "Runtime2 Report";

    // When
    let report = bdd_progress_report_for_runtime2_profile(BddPhase::Runtime, title, profile);

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_core_phase_when_generating_report_then_contains_title() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);
    let title = "Core Report";

    // When
    let report = bdd_progress_report_for_runtime2_profile(BddPhase::Core, title, profile);

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_any_phase_when_generating_report_then_contains_governance_status() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let report = bdd_progress_report_for_runtime2_profile(BddPhase::Runtime, "Test", profile);

    // Then
    assert!(report.contains("Governance status"));
}

#[test]
fn given_any_phase_when_generating_report_then_contains_feature_profile() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let report = bdd_progress_report_for_runtime2_profile(BddPhase::Runtime, "Test", profile);

    // Then
    assert!(report.contains("Feature profile:"));
}

// ---------------------------------------------------------------------------
// bdd_progress_status_line_for_runtime2_profile Tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_when_generating_status_line_then_starts_with_core() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let status = bdd_progress_status_line_for_runtime2_profile(BddPhase::Core, profile);

    // Then
    assert!(status.starts_with("core:"));
}

#[test]
fn given_runtime_phase_when_generating_status_line_then_starts_with_runtime() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let status = bdd_progress_status_line_for_runtime2_profile(BddPhase::Runtime, profile);

    // Then
    assert!(status.starts_with("runtime:"));
}

#[test]
fn given_profile_when_generating_status_line_then_contains_profile_info() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let status = bdd_progress_status_line_for_runtime2_profile(BddPhase::Runtime, profile);

    // Then
    assert!(status.contains(&format!("{}", profile)));
}

// ---------------------------------------------------------------------------
// runtime2_governance_snapshot Tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_when_creating_snapshot_then_phase_matches() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let snap = runtime2_governance_snapshot(BddPhase::Core, profile);

    // Then
    assert_eq!(snap.phase, BddPhase::Core);
}

#[test]
fn given_runtime_phase_when_creating_snapshot_then_phase_matches() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let snap = runtime2_governance_snapshot(BddPhase::Runtime, profile);

    // Then
    assert_eq!(snap.phase, BddPhase::Runtime);
}

#[test]
fn given_profile_when_creating_snapshot_then_profile_matches() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let snap = runtime2_governance_snapshot(BddPhase::Core, profile);

    // Then
    assert_eq!(snap.profile, profile);
}

#[test]
fn given_snapshot_when_checking_totals_then_total_is_positive() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let snap = runtime2_governance_snapshot(BddPhase::Runtime, profile);

    // Then
    assert!(snap.total > 0);
    assert!(snap.implemented <= snap.total);
}

// ---------------------------------------------------------------------------
// bdd_governance_matrix_for_profile Tests
// ---------------------------------------------------------------------------

#[test]
fn given_phase_and_profile_when_creating_matrix_then_has_scenarios() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let matrix = bdd_governance_matrix_for_profile(BddPhase::Core, profile);

    // Then
    assert!(!matrix.scenarios.is_empty());
}

#[test]
fn given_phase_and_pure_rust_flag_when_creating_runtime2_matrix_then_has_scenarios() {
    // Given / When
    let matrix = bdd_governance_matrix_for_runtime2_profile(BddPhase::Core, true);

    // Then
    assert!(!matrix.scenarios.is_empty());
}

// ---------------------------------------------------------------------------
// Re-exported Constants Tests
// ---------------------------------------------------------------------------

#[test]
fn given_glr_conflict_fallback_when_checking_then_is_non_empty() {
    // Given / When / Then
    assert!(!GLR_CONFLICT_FALLBACK.is_empty());
}

#[test]
fn given_glr_conflict_preservation_grid_when_checking_then_is_non_empty() {
    // Given / When / Then
    assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}

// ---------------------------------------------------------------------------
// ParserBackend Tests
// ---------------------------------------------------------------------------

#[test]
fn given_glr_backend_when_displaying_then_shows_glr() {
    // Given
    let backend = ParserBackend::GLR;

    // When
    let display = format!("{}", backend);

    // Then
    assert!(display.contains("GLR"));
}

#[test]
fn given_tree_sitter_backend_when_displaying_then_is_non_empty() {
    // Given
    let backend = ParserBackend::TreeSitter;

    // When
    let display = format!("{}", backend);

    // Then
    assert!(!display.is_empty());
}

// ---------------------------------------------------------------------------
// BddScenario Tests
// ---------------------------------------------------------------------------

#[test]
fn given_scenario_when_getting_core_status_then_returns_correct_status() {
    // Given
    let scenario = &GLR_CONFLICT_PRESERVATION_GRID[0];

    // When
    let status = scenario.status(BddPhase::Core);

    // Then
    // Just verify we can access it
    let _ = status.implemented();
}

#[test]
fn given_scenario_when_getting_runtime_status_then_returns_correct_status() {
    // Given
    let scenario = &GLR_CONFLICT_PRESERVATION_GRID[0];

    // When
    let status = scenario.status(BddPhase::Runtime);

    // Then
    // Just verify we can access it
    let _ = status.implemented();
}
