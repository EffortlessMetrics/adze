//! BDD-style tests for governance-matrix-contract crate.
//!
//! Tests follow the Given/When/Then pattern to verify public API behavior.

use adze_governance_matrix_contract::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, GLR_CONFLICT_FALLBACK,
    GLR_CONFLICT_PRESERVATION_GRID, ParserFeatureProfile, bdd_governance_snapshot, bdd_progress,
    bdd_progress_report, bdd_progress_report_with_profile, bdd_progress_status_line,
    describe_backend_for_conflicts,
};

#[test]
fn given_core_phase_when_creating_matrix_with_new_then_phase_is_set() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let matrix = BddGovernanceMatrix::new(BddPhase::Core, profile, GLR_CONFLICT_PRESERVATION_GRID);

    // Then
    assert_eq!(matrix.phase, BddPhase::Core);
}

#[test]
fn given_runtime_phase_when_creating_matrix_with_new_then_phase_is_set() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let matrix =
        BddGovernanceMatrix::new(BddPhase::Runtime, profile, GLR_CONFLICT_PRESERVATION_GRID);

    // Then
    assert_eq!(matrix.phase, BddPhase::Runtime);
}

#[test]
fn given_standard_matrix_when_creating_then_uses_core_phase() {
    // Given / When
    let matrix = BddGovernanceMatrix::standard(ParserFeatureProfile::current());

    // Then
    assert_eq!(matrix.phase, BddPhase::Core);
}

#[test]
fn given_matrix_when_generating_report_then_contains_title() {
    // Given
    let profile = ParserFeatureProfile::current();
    let matrix = BddGovernanceMatrix::standard(profile);
    let title = "Test Matrix Report";

    // When
    let report = matrix.report(title);

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_matrix_when_generating_status_line_then_starts_with_phase() {
    // Given
    let profile = ParserFeatureProfile::current();
    let matrix = BddGovernanceMatrix::standard(profile);

    // When
    let status = matrix.status_line();

    // Then
    assert!(status.starts_with("core:"));
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
    assert_eq!(snapshot.total, matrix.scenarios.len());
}

#[test]
fn given_snapshot_with_equal_counts_when_checking_fully_implemented_then_returns_true() {
    // Given
    let snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 5,
        total: 5,
        profile: ParserFeatureProfile::current(),
    };

    // When
    let result = snapshot.is_fully_implemented();

    // Then
    assert!(result);
}

#[test]
fn given_snapshot_with_partial_counts_when_checking_fully_implemented_then_returns_false() {
    // Given
    let snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 3,
        total: 5,
        profile: ParserFeatureProfile::current(),
    };

    // When
    let result = snapshot.is_fully_implemented();

    // Then
    assert!(!result);
}

#[test]
fn given_snapshot_when_getting_non_conflict_backend_then_returns_valid_backend() {
    // Given
    let snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 5,
        total: 5,
        profile: ParserFeatureProfile::current(),
    };

    // When
    let backend = snapshot.non_conflict_backend();

    // Then
    assert!(!backend.name().is_empty());
}

#[test]
fn given_core_phase_when_calling_bdd_progress_then_returns_valid_counts() {
    // Given / When
    let (implemented, total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);

    // Then
    assert!(total > 0);
    assert!(implemented <= total);
}

#[test]
fn given_runtime_phase_when_calling_bdd_progress_then_returns_valid_counts() {
    // Given / When
    let (implemented, total) = bdd_progress(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID);

    // Then
    assert!(total > 0);
    assert!(implemented <= total);
}

#[test]
fn given_empty_scenarios_when_calling_bdd_progress_then_returns_zero_counts() {
    // Given
    let scenarios: &[adze_governance_matrix_contract::BddScenario] = &[];

    // When
    let (implemented, total) = bdd_progress(BddPhase::Core, scenarios);

    // Then
    assert_eq!(implemented, 0);
    assert_eq!(total, 0);
}

#[test]
fn given_title_when_calling_bdd_progress_report_then_report_contains_title() {
    // Given
    let title = "Matrix Contract Report";

    // When
    let report = bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, title);

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_profile_and_title_when_calling_bdd_progress_report_with_profile_then_report_contains_title()
 {
    // Given
    let profile = ParserFeatureProfile::current();
    let title = "Profile Matrix Report";

    // When
    let report = bdd_progress_report_with_profile(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        title,
        profile,
    );

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_core_phase_when_calling_status_line_then_starts_with_core() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let status = bdd_progress_status_line(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Then
    assert!(status.starts_with("core:"));
}

#[test]
fn given_runtime_phase_when_calling_status_line_then_starts_with_runtime() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let status =
        bdd_progress_status_line(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Then
    assert!(status.starts_with("runtime:"));
}

#[test]
fn given_profile_when_calling_describe_backend_for_conflicts_then_returns_non_empty() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let description = describe_backend_for_conflicts(profile);

    // Then
    assert!(!description.is_empty());
}

#[test]
fn given_glr_conflict_fallback_when_checking_then_is_non_empty() {
    // Given / When
    let fallback = GLR_CONFLICT_FALLBACK;

    // Then
    assert!(!fallback.is_empty());
}

#[test]
fn given_phase_and_grid_when_calling_bdd_governance_snapshot_then_returns_snapshot() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let snapshot = bdd_governance_snapshot(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Then
    assert_eq!(snapshot.phase, BddPhase::Core);
    assert_eq!(snapshot.profile, profile);
    assert!(snapshot.total > 0);
}

#[test]
fn given_snapshot_when_cloning_then_values_match() {
    // Given
    let snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Runtime,
        implemented: 3,
        total: 7,
        profile: ParserFeatureProfile::current(),
    };

    // When
    let cloned = snapshot;

    // Then
    assert_eq!(snapshot, cloned);
}
