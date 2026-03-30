//! BDD-style tests for parser-contract crate.
//!
//! Tests follow the Given/When/Then pattern to verify public API behavior.

use adze_parser_contract::{
    BddGovernanceMatrix, BddPhase, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID,
    ParserBackend, ParserFeatureProfile, bdd_progress, bdd_progress_report,
};

#[test]
fn given_current_profile_when_creating_matrix_then_matrix_is_valid() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let matrix = BddGovernanceMatrix::standard(profile);

    // Then
    assert_eq!(matrix.phase, BddPhase::Core);
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
fn given_title_when_calling_bdd_progress_report_then_report_contains_title() {
    // Given
    let title = "Parser Contract Test Report";

    // When
    let report = bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, title);

    // Then
    assert!(report.contains(title));
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
    let status = BddScenarioStatus::Deferred { reason: "pending" };

    // When
    let result = status.implemented();

    // Then
    assert!(!result);
}

#[test]
fn given_tree_sitter_backend_when_calling_name_then_returns_non_empty() {
    // Given / When
    let name = ParserBackend::TreeSitter.name();

    // Then
    assert!(!name.is_empty());
}

#[test]
fn given_pure_rust_backend_when_calling_name_then_returns_non_empty() {
    // Given / When
    let name = ParserBackend::PureRust.name();

    // Then
    assert!(!name.is_empty());
}

#[test]
fn given_glr_backend_when_calling_name_then_returns_non_empty() {
    // Given / When
    let name = ParserBackend::GLR.name();

    // Then
    assert!(!name.is_empty());
}

#[test]
fn given_glr_backend_when_checking_is_glr_then_returns_true() {
    // Given / When
    let result = ParserBackend::GLR.is_glr();

    // Then
    assert!(result);
}

#[test]
fn given_tree_sitter_backend_when_checking_is_glr_then_returns_false() {
    // Given / When
    let result = ParserBackend::TreeSitter.is_glr();

    // Then
    assert!(!result);
}

#[test]
fn given_pure_rust_backend_when_checking_is_pure_rust_then_returns_true() {
    // Given / When
    let result = ParserBackend::PureRust.is_pure_rust();

    // Then
    assert!(result);
}

#[test]
fn given_glr_backend_when_checking_is_pure_rust_then_returns_true() {
    // Given / When
    let result = ParserBackend::GLR.is_pure_rust();

    // Then
    assert!(result);
}

#[test]
fn given_tree_sitter_backend_when_checking_is_pure_rust_then_returns_false() {
    // Given / When
    let result = ParserBackend::TreeSitter.is_pure_rust();

    // Then
    assert!(!result);
}

#[test]
fn given_current_profile_when_calling_resolve_backend_then_returns_valid_backend() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let backend = profile.resolve_backend(false);

    // Then
    assert!(!backend.name().is_empty());
}

#[test]
fn given_matrix_when_calling_status_line_then_returns_non_empty() {
    // Given
    let matrix = BddGovernanceMatrix::standard(ParserFeatureProfile::current());

    // When
    let status_line = matrix.status_line();

    // Then
    assert!(!status_line.is_empty());
}

#[test]
fn given_implemented_status_when_calling_icon_then_returns_non_empty() {
    // Given
    let status = BddScenarioStatus::Implemented;

    // When
    let icon = status.icon();

    // Then
    assert!(!icon.is_empty());
}

#[test]
fn given_deferred_status_when_calling_icon_then_returns_different_icon() {
    // Given
    let implemented = BddScenarioStatus::Implemented;
    let deferred = BddScenarioStatus::Deferred { reason: "test" };

    // When
    let icon_implemented = implemented.icon();
    let icon_deferred = deferred.icon();

    // Then
    assert_ne!(icon_implemented, icon_deferred);
}
