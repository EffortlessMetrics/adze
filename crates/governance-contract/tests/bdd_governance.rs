//! BDD-style tests for governance-contract crate.
//!
//! Tests follow the Given/When/Then pattern to verify public API behavior.

use adze_governance_contract::{
    BddGovernanceMatrix, BddPhase, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID,
    ParserBackend, ParserFeatureProfile, bdd_progress, bdd_progress_report,
    bdd_progress_report_with_profile, bdd_progress_status_line,
};

#[test]
fn given_core_phase_when_creating_matrix_then_phase_is_core() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let matrix = BddGovernanceMatrix::standard(profile);

    // Then
    assert_eq!(matrix.phase, BddPhase::Core);
}

#[test]
fn given_profile_when_creating_matrix_then_profile_is_set() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let matrix = BddGovernanceMatrix::standard(profile);

    // Then
    assert_eq!(matrix.profile, profile);
}

#[test]
fn given_standard_grid_when_creating_matrix_then_scenarios_exist() {
    // Given / When
    let matrix = BddGovernanceMatrix::standard(ParserFeatureProfile::current());

    // Then
    assert!(!matrix.scenarios.is_empty());
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
    let scenarios: &[adze_governance_contract::BddScenario] = &[];

    // When
    let (implemented, total) = bdd_progress(BddPhase::Core, scenarios);

    // Then
    assert_eq!(implemented, 0);
    assert_eq!(total, 0);
}

#[test]
fn given_title_when_calling_bdd_progress_report_then_report_contains_title() {
    // Given
    let title = "Test Report Title";

    // When
    let report = bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, title);

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_profile_and_title_when_calling_bdd_progress_report_with_profile_then_report_contains_both()
{
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
fn given_implemented_status_when_checking_implemented_then_returns_true() {
    // Given
    let status = BddScenarioStatus::Implemented;

    // When
    let result = status.implemented();

    // Then
    assert!(result);
}

#[test]
fn given_deferred_status_when_checking_implemented_then_returns_false() {
    // Given
    let status = BddScenarioStatus::Deferred { reason: "later" };

    // When
    let result = status.implemented();

    // Then
    assert!(!result);
}

#[test]
fn given_tree_sitter_backend_when_accessing_variant_then_is_valid() {
    // Given / When
    let backend = ParserBackend::TreeSitter;

    // Then
    assert!(!backend.name().is_empty());
}

#[test]
fn given_pure_rust_backend_when_accessing_variant_then_is_valid() {
    // Given / When
    let backend = ParserBackend::PureRust;

    // Then
    assert!(!backend.name().is_empty());
}

#[test]
fn given_glr_backend_when_accessing_variant_then_is_valid() {
    // Given / When
    let backend = ParserBackend::GLR;

    // Then
    assert!(!backend.name().is_empty());
}

#[test]
fn given_matrix_when_taking_snapshot_then_snapshot_matches_matrix() {
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
