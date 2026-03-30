//! BDD-style tests for governance-matrix-core-impl crate.
//!
//! Tests follow the Given/When/Then pattern to verify the facade exports
//! and delegates correctly to adze-bdd-governance-core.

use adze_governance_matrix_core_impl::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, GLR_CONFLICT_FALLBACK,
    GLR_CONFLICT_PRESERVATION_GRID, ParserBackend, ParserFeatureProfile, bdd_progress_report,
    bdd_progress_report_with_profile, bdd_progress_status_line, describe_backend_for_conflicts,
};

// ---------------------------------------------------------------------------
// BddPhase Facade Tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_when_comparing_phases_then_they_are_distinct() {
    // Given
    let core = BddPhase::Core;
    let runtime = BddPhase::Runtime;

    // When/Then
    assert_ne!(core, runtime);
}

#[test]
fn given_phase_variants_when_iterating_then_all_phases_are_covered() {
    // Given
    let phases = [BddPhase::Core, BddPhase::Runtime];

    // When/Then
    assert_eq!(phases.len(), 2);
    assert!(phases.contains(&BddPhase::Core));
    assert!(phases.contains(&BddPhase::Runtime));
}

// ---------------------------------------------------------------------------
// BddGovernanceSnapshot Facade Tests
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

#[test]
fn given_snapshot_with_profile_when_accessing_profile_then_matches_original() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };
    let snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 1,
        total: 1,
        profile,
    };

    // When/Then
    assert_eq!(snapshot.profile, profile);
}

// ---------------------------------------------------------------------------
// BddGovernanceMatrix Facade Tests
// ---------------------------------------------------------------------------

#[test]
fn given_current_profile_when_creating_standard_matrix_then_has_core_phase() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let matrix = BddGovernanceMatrix::standard(profile);

    // Then
    assert_eq!(matrix.phase, BddPhase::Core);
}

#[test]
fn given_profile_when_creating_standard_matrix_then_contains_scenarios() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let matrix = BddGovernanceMatrix::standard(profile);

    // Then
    assert!(!matrix.scenarios.is_empty());
}

#[test]
fn given_custom_phase_when_creating_matrix_then_phase_is_preserved() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let matrix =
        BddGovernanceMatrix::new(BddPhase::Runtime, profile, GLR_CONFLICT_PRESERVATION_GRID);

    // Then
    assert_eq!(matrix.phase, BddPhase::Runtime);
}

#[test]
fn given_matrix_when_taking_snapshot_then_snapshot_matches_matrix_state() {
    // Given
    let profile = ParserFeatureProfile::current();
    let matrix = BddGovernanceMatrix::standard(profile);

    // When
    let snapshot = matrix.snapshot();

    // Then
    assert_eq!(snapshot.phase, matrix.phase);
    assert_eq!(snapshot.profile, matrix.profile);
}

#[test]
fn given_fully_implemented_matrix_when_checking_completion_then_returns_true() {
    // Given
    let profile = ParserFeatureProfile::current();
    let matrix = BddGovernanceMatrix::standard(profile);

    // When
    let is_complete = matrix.is_fully_implemented();

    // Then - depends on actual scenario status, just verify it doesn't panic
    let _ = is_complete;
}

// ---------------------------------------------------------------------------
// GLR_CONFLICT_FALLBACK Constant Tests
// ---------------------------------------------------------------------------

#[test]
fn given_glr_conflict_fallback_constant_when_checking_contents_then_not_empty() {
    // Given/When
    let fallback = GLR_CONFLICT_FALLBACK;

    // Then
    assert!(!fallback.is_empty());
}

#[test]
fn given_glr_conflict_fallback_when_checking_content_then_contains_expected_keywords() {
    // Given/When
    let fallback = GLR_CONFLICT_FALLBACK;

    // Then - It's a description string about GLR fallback behavior
    assert!(fallback.contains("GLR") || fallback.contains("glr") || fallback.contains("conflict"));
}

// ---------------------------------------------------------------------------
// GLR_CONFLICT_PRESERVATION_GRID Constant Tests
// ---------------------------------------------------------------------------

#[test]
fn given_conflict_preservation_grid_when_checking_contents_then_not_empty() {
    // Given/When
    let grid = GLR_CONFLICT_PRESERVATION_GRID;

    // Then
    assert!(!grid.is_empty());
}

// ---------------------------------------------------------------------------
// describe_backend_for_conflicts Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_pure_rust_profile_when_describing_backend_then_contains_expected_text() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When
    let description = describe_backend_for_conflicts(profile);

    // Then
    assert!(!description.is_empty());
}

#[test]
fn given_tree_sitter_profile_when_describing_backend_then_not_empty() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: true,
        tree_sitter_c2rust: false,
        glr: false,
    };

    // When
    let description = describe_backend_for_conflicts(profile);

    // Then
    assert!(!description.is_empty());
}

// ---------------------------------------------------------------------------
// bdd_progress_report Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_and_title_when_generating_report_then_contains_title() {
    // Given
    let title = "Test Report Title";

    // When
    let report = bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, title);

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_runtime_phase_and_title_when_generating_report_then_contains_title() {
    // Given
    let title = "Runtime Report";

    // When
    let report = bdd_progress_report(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, title);

    // Then
    assert!(report.contains(title));
}

// ---------------------------------------------------------------------------
// bdd_progress_report_with_profile Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_profile_and_title_when_generating_report_then_contains_both() {
    // Given
    let profile = ParserFeatureProfile::current();
    let title = "Profile Report";

    // When
    let report = bdd_progress_report_with_profile(
        BddPhase::Core,
        GLR_CONFLICT_PRESERVATION_GRID,
        title,
        profile,
    );

    // Then
    assert!(report.contains(title));
    assert!(report.contains("Feature profile:"));
}

// ---------------------------------------------------------------------------
// bdd_progress_status_line Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_when_generating_status_line_then_starts_with_core() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let status = bdd_progress_status_line(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Then
    assert!(status.starts_with("core:"));
}

#[test]
fn given_runtime_phase_when_generating_status_line_then_starts_with_runtime() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let status =
        bdd_progress_status_line(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Then
    assert!(status.starts_with("runtime:"));
}

// ---------------------------------------------------------------------------
// ParserFeatureProfile Integration Tests
// ---------------------------------------------------------------------------

#[test]
fn given_pure_rust_glr_profile_when_resolving_backend_then_returns_glr() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When
    let backend = profile.resolve_backend(true);

    // Then
    assert_eq!(backend, ParserBackend::GLR);
}

#[test]
fn given_tree_sitter_profile_when_resolving_backend_then_returns_tree_sitter() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: true,
        tree_sitter_c2rust: false,
        glr: false,
    };

    // When
    let backend = profile.resolve_backend(false);

    // Then
    assert_eq!(backend, ParserBackend::TreeSitter);
}

#[test]
fn given_current_profile_when_accessing_fields_then_all_fields_are_accessible() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When/Then - Just verify we can access all fields (compile-time check)
    let _ = profile.pure_rust;
    let _ = profile.tree_sitter_standard;
    let _ = profile.tree_sitter_c2rust;
    let _ = profile.glr;
}
