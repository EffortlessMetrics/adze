//! BDD-style tests for runtime2-governance crate.
//!
//! Tests follow the Given/When/Then pattern to verify the runtime2-specific
//! governance façade over the shared governance-matrix crate.

use adze_runtime2_governance::{
    BddGovernanceSnapshot, BddPhase, GLR_CONFLICT_FALLBACK, GLR_CONFLICT_PRESERVATION_GRID,
    ParserBackend, ParserFeatureProfile, bdd_governance_matrix_for_profile,
    bdd_governance_matrix_for_runtime2, bdd_governance_matrix_for_runtime2_profile,
    bdd_governance_snapshot, bdd_progress, bdd_progress_report,
    bdd_progress_report_for_runtime2_profile, bdd_progress_report_with_profile,
    bdd_progress_status_line,
    bdd_progress_status_line_for_runtime2_profile, describe_backend_for_conflicts,
    parser_feature_profile_for_runtime2, resolve_backend_for_profile,
    resolve_backend_for_runtime2_profile, resolve_runtime2_backend, runtime2_governance_snapshot,
};

// ---------------------------------------------------------------------------
// parser_feature_profile_for_runtime2 Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_pure_rust_true_when_getting_runtime2_profile_then_pure_rust_and_glr_enabled() {
    // Given
    let pure_rust = true;

    // When
    let profile = parser_feature_profile_for_runtime2(pure_rust);

    // Then
    assert!(profile.pure_rust);
    assert!(profile.glr);
    assert!(!profile.tree_sitter_standard);
    assert!(!profile.tree_sitter_c2rust);
}

#[test]
fn given_pure_rust_false_when_getting_runtime2_profile_then_pure_rust_and_glr_disabled() {
    // Given
    let pure_rust = false;

    // When
    let profile = parser_feature_profile_for_runtime2(pure_rust);

    // Then
    assert!(!profile.pure_rust);
    assert!(!profile.glr);
}

// ---------------------------------------------------------------------------
// resolve_backend_for_runtime2_profile Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_glr_profile_with_conflicts_when_resolving_backend_via_profile_then_returns_glr() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let backend = resolve_backend_for_runtime2_profile(profile, true);

    // Then
    assert_eq!(backend, ParserBackend::GLR);
}

#[test]
fn given_non_glr_profile_without_conflicts_when_resolving_backend_via_profile_then_returns_tree_sitter()
 {
    // Given
    let profile = parser_feature_profile_for_runtime2(false);

    // When
    let backend = resolve_backend_for_runtime2_profile(profile, false);

    // Then
    assert_eq!(backend, ParserBackend::TreeSitter);
}

// ---------------------------------------------------------------------------
// resolve_runtime2_backend Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_pure_rust_true_with_conflicts_when_resolving_runtime2_backend_then_returns_glr() {
    // Given
    let pure_rust = true;
    let has_conflicts = true;

    // When
    let backend = resolve_runtime2_backend(pure_rust, has_conflicts);

    // Then
    assert_eq!(backend, ParserBackend::GLR);
}

#[test]
fn given_pure_rust_false_without_conflicts_when_resolving_runtime2_backend_then_returns_tree_sitter()
 {
    // Given
    let pure_rust = false;
    let has_conflicts = false;

    // When
    let backend = resolve_runtime2_backend(pure_rust, has_conflicts);

    // Then
    assert_eq!(backend, ParserBackend::TreeSitter);
}

// ---------------------------------------------------------------------------
// bdd_governance_matrix_for_runtime2 Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_pure_rust_true_when_creating_runtime2_matrix_then_has_correct_profile() {
    // Given
    let pure_rust = true;

    // When
    let matrix = bdd_governance_matrix_for_runtime2(BddPhase::Runtime, pure_rust);

    // Then
    assert!(matrix.profile.pure_rust);
    assert!(matrix.profile.glr);
}

#[test]
fn given_pure_rust_false_when_creating_runtime2_matrix_then_has_correct_profile() {
    // Given
    let pure_rust = false;

    // When
    let matrix = bdd_governance_matrix_for_runtime2(BddPhase::Core, pure_rust);

    // Then
    assert!(!matrix.profile.pure_rust);
}

// ---------------------------------------------------------------------------
// bdd_governance_matrix_for_runtime2_profile Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_phase_and_pure_rust_when_creating_runtime2_matrix_with_profile_then_profile_is_correct() {
    // Given
    let pure_rust = true;

    // When
    let matrix = bdd_governance_matrix_for_runtime2_profile(BddPhase::Runtime, pure_rust);

    // Then
    assert!(matrix.profile.pure_rust);
    assert!(matrix.profile.glr);
}

// ---------------------------------------------------------------------------
// bdd_governance_matrix_for_profile Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_phase_and_custom_profile_when_creating_matrix_then_profile_is_set() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When
    let matrix = bdd_governance_matrix_for_profile(BddPhase::Core, profile);

    // Then
    assert_eq!(matrix.profile, profile);
}

// ---------------------------------------------------------------------------
// runtime2_governance_snapshot Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_phase_and_profile_when_creating_runtime2_snapshot_then_has_correct_values() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let snapshot = runtime2_governance_snapshot(BddPhase::Runtime, profile);

    // Then
    assert_eq!(snapshot.phase, BddPhase::Runtime);
    assert_eq!(snapshot.profile, profile);
}

// ---------------------------------------------------------------------------
// bdd_progress_report_for_runtime2_profile Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_profile_and_title_when_generating_runtime2_report_then_contains_title() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);
    let title = "Runtime2 Report";

    // When
    let report = bdd_progress_report_for_runtime2_profile(BddPhase::Runtime, title, profile);

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_profile_when_generating_runtime2_report_then_contains_feature_profile() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);
    let title = "Test";

    // When
    let report = bdd_progress_report_for_runtime2_profile(BddPhase::Runtime, title, profile);

    // Then
    assert!(report.contains("Feature profile:"));
}

#[test]
fn given_core_phase_when_generating_runtime2_report_then_is_valid() {
    // Given
    let profile = parser_feature_profile_for_runtime2(false);
    let title = "Core Report";

    // When
    let report = bdd_progress_report_for_runtime2_profile(BddPhase::Core, title, profile);

    // Then
    assert!(report.contains("Core Report"));
}

// ---------------------------------------------------------------------------
// bdd_progress_status_line_for_runtime2_profile Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_runtime_phase_when_generating_runtime2_status_line_then_contains_runtime_prefix() {
    // Given
    let profile = parser_feature_profile_for_runtime2(true);

    // When
    let status = bdd_progress_status_line_for_runtime2_profile(BddPhase::Runtime, profile);

    // Then
    assert!(status.contains("runtime:"));
}

#[test]
fn given_core_phase_when_generating_runtime2_status_line_then_contains_core_prefix() {
    // Given
    let profile = parser_feature_profile_for_runtime2(false);

    // When
    let status = bdd_progress_status_line_for_runtime2_profile(BddPhase::Core, profile);

    // Then
    assert!(status.contains("core:"));
}

// ---------------------------------------------------------------------------
// Re-exported Function Tests: bdd_progress
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_when_calling_bdd_progress_then_returns_valid_counts() {
    // Given/When
    let (implemented, total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);

    // Then
    assert!(total > 0);
    assert!(implemented <= total);
}

#[test]
fn given_runtime_phase_when_calling_bdd_progress_then_returns_valid_counts() {
    // Given/When
    let (implemented, total) = bdd_progress(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID);

    // Then
    assert!(total > 0);
    assert!(implemented <= total);
}

// ---------------------------------------------------------------------------
// Re-exported Function Tests: bdd_governance_snapshot
// ---------------------------------------------------------------------------

#[test]
fn given_phase_and_profile_when_creating_snapshot_then_matches_inputs() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let snapshot = bdd_governance_snapshot(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Then
    assert_eq!(snapshot.phase, BddPhase::Core);
    assert_eq!(snapshot.profile, profile);
}

// ---------------------------------------------------------------------------
// Re-exported Function Tests: bdd_progress_report
// ---------------------------------------------------------------------------

#[test]
fn given_title_when_generating_progress_report_then_contains_title() {
    // Given
    let title = "Progress Report";

    // When
    let report = bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, title);

    // Then
    assert!(report.contains(title));
}

// ---------------------------------------------------------------------------
// Re-exported Function Tests: bdd_progress_report_with_profile
// ---------------------------------------------------------------------------

#[test]
fn given_profile_and_title_when_generating_report_with_profile_then_contains_both() {
    // Given
    let profile = ParserFeatureProfile::current();
    let title = "Profile Report";

    // When
    let report = bdd_progress_report_with_profile(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        title,
        profile,
    );

    // Then
    assert!(report.contains(title));
    assert!(report.contains("Feature profile:"));
}

// ---------------------------------------------------------------------------
// Re-exported Function Tests: bdd_progress_status_line
// ---------------------------------------------------------------------------

#[test]
fn given_phase_when_generating_status_line_then_starts_with_phase_prefix() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let status = bdd_progress_status_line(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Then
    assert!(status.starts_with("core:"));
}

// ---------------------------------------------------------------------------
// Re-exported Function Tests: describe_backend_for_conflicts
// ---------------------------------------------------------------------------

#[test]
fn given_profile_when_describing_backend_for_conflicts_then_not_empty() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let description = describe_backend_for_conflicts(profile);

    // Then
    assert!(!description.is_empty());
}

// ---------------------------------------------------------------------------
// Re-exported Function Tests: resolve_backend_for_profile
// ---------------------------------------------------------------------------

#[test]
fn given_glr_profile_with_conflicts_when_resolving_backend_via_function_then_returns_glr() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When
    let backend = resolve_backend_for_profile(profile, true);

    // Then
    assert_eq!(backend, ParserBackend::GLR);
}

// ---------------------------------------------------------------------------
// Re-exported Constant Tests
// ---------------------------------------------------------------------------

#[test]
fn given_glr_conflict_fallback_when_checking_then_not_empty() {
    // Given/When
    let fallback = GLR_CONFLICT_FALLBACK;

    // Then
    assert!(!fallback.is_empty());
}

#[test]
fn given_conflict_preservation_grid_when_checking_then_not_empty() {
    // Given/When
    let grid = GLR_CONFLICT_PRESERVATION_GRID;

    // Then
    assert!(!grid.is_empty());
}

#[test]
fn given_glr_conflict_fallback_when_checking_content_then_contains_expected_keywords() {
    // Given/When
    let fallback = GLR_CONFLICT_FALLBACK;

    // Then - It's a description string about GLR fallback behavior
    assert!(fallback.contains("GLR") || fallback.contains("glr") || fallback.contains("conflict"));
}

// ---------------------------------------------------------------------------
// BddGovernanceSnapshot Tests
// ---------------------------------------------------------------------------

#[test]
fn given_fully_implemented_snapshot_when_checking_completion_then_returns_true() {
    // Given
    let snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 5,
        total: 5,
        profile: ParserFeatureProfile::current(),
    };

    // When
    let is_complete = snapshot.is_fully_implemented();

    // Then
    assert!(is_complete);
}

#[test]
fn given_partially_implemented_snapshot_when_checking_completion_then_returns_false() {
    // Given
    let snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Runtime,
        implemented: 2,
        total: 5,
        profile: ParserFeatureProfile::current(),
    };

    // When
    let is_complete = snapshot.is_fully_implemented();

    // Then
    assert!(!is_complete);
}

#[test]
fn given_zero_zero_snapshot_when_checking_completion_then_returns_true() {
    // Given - 0/0 is vacuously true
    let snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 0,
        total: 0,
        profile: ParserFeatureProfile::current(),
    };

    // When
    let is_complete = snapshot.is_fully_implemented();

    // Then
    assert!(is_complete);
}

// ---------------------------------------------------------------------------
// ParserBackend Tests
// ---------------------------------------------------------------------------

#[test]
fn given_glr_backend_when_getting_name_then_not_empty() {
    // Given
    let backend = ParserBackend::GLR;

    // When
    let name = backend.name();

    // Then
    assert!(!name.is_empty());
}

#[test]
fn given_tree_sitter_backend_when_getting_name_then_not_empty() {
    // Given
    let backend = ParserBackend::TreeSitter;

    // When
    let name = backend.name();

    // Then
    assert!(!name.is_empty());
}

// ---------------------------------------------------------------------------
// ParserFeatureProfile Tests
// ---------------------------------------------------------------------------

#[test]
fn given_current_profile_when_checking_features_then_all_fields_are_accessible() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When/Then - Just verify we can access all fields (compile-time check)
    let _ = profile.pure_rust;
    let _ = profile.tree_sitter_standard;
    let _ = profile.tree_sitter_c2rust;
    let _ = profile.glr;
}

#[test]
fn given_custom_profile_when_resolving_backend_then_matches_configuration() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When
    let backend = profile.resolve_backend(false);

    // Then
    assert_eq!(backend, ParserBackend::GLR);
}
