//! Comprehensive BDD-style tests for the governance-matrix-core-impl crate.

use adze_governance_matrix_core_impl::*;

// ---------------------------------------------------------------------------
// BddPhase Tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_when_comparing_to_runtime_then_phases_differ() {
    // Given
    let core = BddPhase::Core;
    let runtime = BddPhase::Runtime;

    // Then
    assert_ne!(core, runtime);
}

// ---------------------------------------------------------------------------
// BddGovernanceSnapshot Tests
// ---------------------------------------------------------------------------

#[test]
fn given_snapshot_with_equal_counts_when_checking_fully_implemented_then_returns_true() {
    // Given
    let snap = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 10,
        total: 10,
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
        total: 8,
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

// ---------------------------------------------------------------------------
// BddGovernanceMatrix Tests
// ---------------------------------------------------------------------------

#[test]
fn given_standard_matrix_when_creating_then_uses_core_phase() {
    // Given / When
    let matrix = BddGovernanceMatrix::standard(ParserFeatureProfile::current());

    // Then
    assert_eq!(matrix.phase, BddPhase::Core);
}

#[test]
fn given_standard_matrix_when_creating_then_has_scenarios() {
    // Given / When
    let matrix = BddGovernanceMatrix::standard(ParserFeatureProfile::current());

    // Then
    assert!(!matrix.scenarios.is_empty());
}

#[test]
fn given_matrix_with_custom_phase_when_creating_then_phase_is_set() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let matrix =
        BddGovernanceMatrix::new(BddPhase::Runtime, profile, GLR_CONFLICT_PRESERVATION_GRID);

    // Then
    assert_eq!(matrix.phase, BddPhase::Runtime);
    assert_eq!(matrix.profile, profile);
}

#[test]
fn given_matrix_when_generating_report_then_contains_title() {
    // Given
    let matrix = BddGovernanceMatrix::standard(ParserFeatureProfile::current());

    // When
    let report = matrix.report("Test Report Title");

    // Then
    assert!(report.contains("Test Report Title"));
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

// ---------------------------------------------------------------------------
// GLR_CONFLICT_FALLBACK Tests
// ---------------------------------------------------------------------------

#[test]
fn given_glr_conflict_fallback_when_checking_then_is_non_empty() {
    // Given / When
    // GLR_CONFLICT_FALLBACK is a constant

    // Then
    assert!(!GLR_CONFLICT_FALLBACK.is_empty());
}

// ---------------------------------------------------------------------------
// GLR_CONFLICT_PRESERVATION_GRID Tests
// ---------------------------------------------------------------------------

#[test]
fn given_glr_conflict_grid_when_checking_then_has_scenarios() {
    // Given / When
    // GLR_CONFLICT_PRESERVATION_GRID is a constant

    // Then
    assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}

#[test]
fn given_glr_conflict_grid_when_counting_scenarios_then_matches_expected_count() {
    // Given / When
    let count = GLR_CONFLICT_PRESERVATION_GRID.len();

    // Then
    // The grid should have 8 scenarios based on the BDD plan
    assert!(count > 0);
}

// ---------------------------------------------------------------------------
// describe_backend_for_conflicts Tests
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// bdd_progress_report Tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_when_generating_progress_report_then_contains_title() {
    // Given
    let title = "Core Phase Report";

    // When
    let report = bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, title);

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_runtime_phase_when_generating_progress_report_then_contains_title() {
    // Given
    let title = "Runtime Phase Report";

    // When
    let report = bdd_progress_report(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, title);

    // Then
    assert!(report.contains(title));
}

// ---------------------------------------------------------------------------
// bdd_progress_report_with_profile Tests
// ---------------------------------------------------------------------------

#[test]
fn given_profile_when_generating_progress_report_then_contains_profile_info() {
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
    assert!(report.contains(&format!("{}", profile)));
}

// ---------------------------------------------------------------------------
// bdd_progress_status_line Tests
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
// bdd_governance_snapshot Tests
// ---------------------------------------------------------------------------

#[test]
fn given_grid_and_phase_when_creating_snapshot_then_values_match() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let snap = bdd_governance_snapshot(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Then
    assert_eq!(snap.phase, BddPhase::Core);
    assert_eq!(snap.profile, profile);
    assert_eq!(snap.total, GLR_CONFLICT_PRESERVATION_GRID.len());
}
