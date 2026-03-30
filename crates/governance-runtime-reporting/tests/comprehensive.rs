//! Comprehensive BDD-style tests for the governance-runtime-reporting crate.

use adze_governance_runtime_reporting::*;

// ---------------------------------------------------------------------------
// bdd_progress_report_with_profile_runtime Tests
// ---------------------------------------------------------------------------

#[test]
fn given_runtime_phase_when_generating_report_then_contains_phase_title() {
    // Given
    let profile = ParserFeatureProfile::current();
    let title = "Runtime Governance";

    // When
    let report = bdd_progress_report_with_profile_runtime(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        title,
        profile,
    );

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_core_phase_when_generating_report_then_contains_phase_title() {
    // Given
    let profile = ParserFeatureProfile::current();
    let title = "Core Governance";

    // When
    let report = bdd_progress_report_with_profile_runtime(
        BddPhase::Core,
        GLR_CONFLICT_PRESERVATION_GRID,
        title,
        profile,
    );

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_any_phase_when_generating_report_then_contains_governance_status() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let report = bdd_progress_report_with_profile_runtime(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        "Test",
        profile,
    );

    // Then
    assert!(report.contains("Governance status:"));
}

#[test]
fn given_any_phase_when_generating_report_then_contains_feature_profile() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let report = bdd_progress_report_with_profile_runtime(
        BddPhase::Core,
        GLR_CONFLICT_PRESERVATION_GRID,
        "Test",
        profile,
    );

    // Then
    assert!(report.contains("Feature profile:"));
}

#[test]
fn given_profile_when_generating_report_then_contains_non_conflict_backend() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When
    let report = bdd_progress_report_with_profile_runtime(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        "Test",
        profile,
    );

    // Then
    assert!(report.contains("Non-conflict backend:"));
}

#[test]
fn given_profile_when_generating_report_then_contains_conflict_profiles() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let report = bdd_progress_report_with_profile_runtime(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        "Test",
        profile,
    );

    // Then
    assert!(report.contains("Conflict profiles:"));
}

#[test]
fn given_empty_scenarios_when_generating_report_then_shows_zero_counts() {
    // Given
    let profile = ParserFeatureProfile::current();
    let empty_scenarios: &[BddScenario] = &[];

    // When
    let report =
        bdd_progress_report_with_profile_runtime(BddPhase::Core, empty_scenarios, "Empty", profile);

    // Then
    assert!(report.contains("Governance status: 0/0"));
}

#[test]
fn given_glr_profile_when_generating_report_then_backend_is_glr() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When
    let report = bdd_progress_report_with_profile_runtime(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        "GLR Test",
        profile,
    );

    // Then
    assert!(report.contains("GLR Test"));
    // GLR should be mentioned as the backend
    let backend = profile.resolve_backend(false);
    assert!(report.contains(&format!("{}", backend)));
}

// ---------------------------------------------------------------------------
// Re-exported bdd_progress Tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_when_counting_progress_then_returns_valid_counts() {
    // Given / When
    let (implemented, total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);

    // Then
    assert!(total > 0);
    assert!(implemented <= total);
}

#[test]
fn given_runtime_phase_when_counting_progress_then_returns_valid_counts() {
    // Given / When
    let (implemented, total) = bdd_progress(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID);

    // Then
    assert!(total > 0);
    assert!(implemented <= total);
}

#[test]
fn given_both_phases_when_counting_progress_then_totals_match() {
    // Given / When
    let (_, core_total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
    let (_, runtime_total) = bdd_progress(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID);

    // Then
    assert_eq!(core_total, runtime_total);
}

// ---------------------------------------------------------------------------
// Re-exported bdd_progress_status_line Tests
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

#[test]
fn given_profile_when_generating_status_line_then_contains_profile_info() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let status = bdd_progress_status_line(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Then
    assert!(status.contains(&format!("{}", profile)));
}

// ---------------------------------------------------------------------------
// Re-exported describe_backend_for_conflicts Tests
// ---------------------------------------------------------------------------

#[test]
fn given_glr_profile_when_describing_backend_then_returns_non_empty() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When
    let desc = describe_backend_for_conflicts(profile);

    // Then
    assert!(!desc.is_empty());
}

#[test]
fn given_tree_sitter_profile_when_describing_backend_then_returns_non_empty() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: true,
        tree_sitter_c2rust: false,
        glr: false,
    };

    // When
    let desc = describe_backend_for_conflicts(profile);

    // Then
    assert!(!desc.is_empty());
}

// ---------------------------------------------------------------------------
// Re-exported bdd_governance_snapshot Tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_when_creating_snapshot_then_phase_matches() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let snap = bdd_governance_snapshot(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Then
    assert_eq!(snap.phase, BddPhase::Core);
}

#[test]
fn given_runtime_phase_when_creating_snapshot_then_phase_matches() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let snap = bdd_governance_snapshot(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Then
    assert_eq!(snap.phase, BddPhase::Runtime);
}

#[test]
fn given_profile_when_creating_snapshot_then_profile_matches() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When
    let snap = bdd_governance_snapshot(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Then
    assert_eq!(snap.profile, profile);
}

// ---------------------------------------------------------------------------
// Re-exported BddGovernanceMatrix Tests
// ---------------------------------------------------------------------------

#[test]
fn given_standard_matrix_when_creating_then_has_scenarios() {
    // Given / When
    let matrix = BddGovernanceMatrix::standard(ParserFeatureProfile::current());

    // Then
    assert!(!matrix.scenarios.is_empty());
}

#[test]
fn given_matrix_when_checking_implementation_then_returns_boolean() {
    // Given
    let matrix = BddGovernanceMatrix::standard(ParserFeatureProfile::current());

    // When
    let _ = matrix.is_fully_implemented();

    // Then - just verify it doesn't panic
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
