//! BDD-style tests for bdd-governance-core crate.
//!
//! Tests follow the Given/When/Then pattern to verify public API behavior.

use adze_bdd_governance_core::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, BddScenarioStatus, GLR_CONFLICT_FALLBACK,
    GLR_CONFLICT_PRESERVATION_GRID, ParserBackend, ParserFeatureProfile, bdd_governance_snapshot,
    bdd_progress, bdd_progress_report, bdd_progress_report_with_profile, bdd_progress_status_line,
    describe_backend_for_conflicts,
};

#[test]
fn given_current_profile_when_creating_standard_matrix_then_matrix_is_valid() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let matrix = BddGovernanceMatrix::standard(profile);

    // Then
    assert_eq!(matrix.phase, BddPhase::Core);
    assert!(!matrix.scenarios.is_empty());
}

#[test]
fn given_custom_phase_when_creating_matrix_then_phase_is_set() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let matrix =
        BddGovernanceMatrix::new(BddPhase::Runtime, profile, GLR_CONFLICT_PRESERVATION_GRID);

    // Then
    assert_eq!(matrix.phase, BddPhase::Runtime);
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
    let scenarios: &[adze_bdd_governance_core::BddScenario] = &[];

    // When
    let (implemented, total) = bdd_progress(BddPhase::Core, scenarios);

    // Then
    assert_eq!(implemented, 0);
    assert_eq!(total, 0);
}

#[test]
fn given_title_when_calling_bdd_progress_report_then_report_contains_title() {
    // Given
    let title = "Governance Core Report";

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
    let title = "Profile Aware Report";

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
    assert!(report.contains("Non-conflict backend:"));
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
    let status = BddScenarioStatus::Deferred { reason: "deferred" };

    // When
    let result = status.implemented();

    // Then
    assert!(!result);
}

#[test]
fn given_snapshot_with_equal_counts_when_checking_fully_implemented_then_returns_true() {
    // Given
    let snap = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 6,
        total: 6,
        profile: ParserFeatureProfile::current(),
    };

    // When
    let result = snap.is_fully_implemented();

    // Then
    assert!(result);
}

#[test]
fn given_snapshot_with_zero_total_when_checking_fully_implemented_then_returns_true() {
    // Given - 0/0 is considered fully implemented (vacuously true)
    let snap = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 0,
        total: 0,
        profile: ParserFeatureProfile::current(),
    };

    // When
    let result = snap.is_fully_implemented();

    // Then
    assert!(result);
}

#[test]
fn given_snapshot_with_partial_implementation_when_checking_fully_implemented_then_returns_false() {
    // Given
    let snap = BddGovernanceSnapshot {
        phase: BddPhase::Runtime,
        implemented: 3,
        total: 6,
        profile: ParserFeatureProfile::current(),
    };

    // When
    let result = snap.is_fully_implemented();

    // Then
    assert!(!result);
}

#[test]
fn given_snapshot_when_resolving_backend_then_returns_expected_backend() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };
    let snap = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 5,
        total: 5,
        profile,
    };

    // When
    let backend = snap.non_conflict_backend();

    // Then
    assert_eq!(backend, profile.resolve_backend(false));
}

#[test]
fn given_glr_conflict_fallback_when_checking_then_is_non_empty() {
    // Given / When / Then
    assert!(!GLR_CONFLICT_FALLBACK.is_empty());
}

#[test]
fn given_glr_conflict_grid_when_checking_then_has_scenarios() {
    // Given / When / Then
    assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}

#[test]
fn given_profile_with_glr_when_describing_backend_then_returns_non_empty() {
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
fn given_profile_without_glr_when_describing_backend_then_returns_non_empty() {
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

#[test]
fn given_grid_and_phase_when_creating_snapshot_then_values_match() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let snap = bdd_governance_snapshot(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Then
    assert_eq!(snap.phase, BddPhase::Core);
    assert_eq!(snap.profile, profile);
    assert!(snap.total > 0);
    assert!(snap.implemented <= snap.total);
}

#[test]
fn given_matrix_when_generating_report_then_contains_title() {
    // Given
    let matrix = BddGovernanceMatrix::standard(ParserFeatureProfile::current());
    let title = "Matrix Report";

    // When
    let report = matrix.report(title);

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_matrix_when_generating_status_line_then_starts_with_phase() {
    // Given
    let profile = ParserFeatureProfile::current();
    let matrix =
        BddGovernanceMatrix::new(BddPhase::Runtime, profile, GLR_CONFLICT_PRESERVATION_GRID);

    // When
    let status = matrix.status_line();

    // Then
    assert!(status.starts_with("runtime:"));
}

#[test]
fn given_matrix_when_taking_snapshot_then_snapshot_matches_matrix_state() {
    // Given
    let profile = ParserFeatureProfile::current();
    let matrix = BddGovernanceMatrix::standard(profile);

    // When
    let snap = matrix.snapshot();

    // Then
    assert_eq!(snap.phase, matrix.phase);
    assert_eq!(snap.profile, matrix.profile);
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
fn given_matrix_when_checking_implementation_then_returns_boolean() {
    // Given
    let matrix = BddGovernanceMatrix::standard(ParserFeatureProfile::current());

    // When
    let _ = matrix.is_fully_implemented();

    // Then - just verify it doesn't panic
}
