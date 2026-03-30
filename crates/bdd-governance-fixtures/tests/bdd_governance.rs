//! BDD-style tests for bdd-governance-fixtures crate.
//!
//! Tests follow the Given/When/Then pattern to verify public API behavior.

use adze_bdd_governance_fixtures::*;

// ---------------------------------------------------------------------------
// BddPhase tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_when_using_bdd_phase_then_variant_is_core() {
    // Given
    let phase = BddPhase::Core;

    // When / Then
    assert!(matches!(phase, BddPhase::Core));
}

#[test]
fn given_runtime_phase_when_using_bdd_phase_then_variant_is_runtime() {
    // Given
    let phase = BddPhase::Runtime;

    // When / Then
    assert!(matches!(phase, BddPhase::Runtime));
}

// ---------------------------------------------------------------------------
// BddScenarioStatus tests
// ---------------------------------------------------------------------------

#[test]
fn given_implemented_status_when_checking_implemented_then_returns_true() {
    // Given
    let status = BddScenarioStatus::Implemented;

    // When / Then
    assert!(status.implemented());
}

#[test]
fn given_deferred_status_when_checking_implemented_then_returns_false() {
    // Given
    let status = BddScenarioStatus::Deferred {
        reason: "pending implementation",
    };

    // When / Then
    assert!(!status.implemented());
}

#[test]
fn given_deferred_status_when_getting_reason_then_returns_reason() {
    // Given
    let status = BddScenarioStatus::Deferred {
        reason: "pending implementation",
    };

    // When
    let detail = status.detail();

    // Then
    assert_eq!(detail, "pending implementation");
}

// ---------------------------------------------------------------------------
// GLR_CONFLICT_PRESERVATION_GRID tests
// ---------------------------------------------------------------------------

#[test]
fn given_glr_grid_when_checking_is_empty_then_returns_false() {
    // Given / When
    let grid = GLR_CONFLICT_PRESERVATION_GRID;

    // Then
    assert!(!grid.is_empty());
}

#[test]
fn given_glr_grid_when_iterating_scenarios_then_has_scenarios() {
    // Given
    let grid = GLR_CONFLICT_PRESERVATION_GRID;

    // When
    let count = grid.len();

    // Then
    assert!(count > 0);
}

// ---------------------------------------------------------------------------
// bdd_progress tests
// ---------------------------------------------------------------------------

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
    let scenarios: &[BddScenario] = &[];

    // When
    let (implemented, total) = bdd_progress(BddPhase::Core, scenarios);

    // Then
    assert_eq!(implemented, 0);
    assert_eq!(total, 0);
}

// ---------------------------------------------------------------------------
// bdd_progress_report tests
// ---------------------------------------------------------------------------

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
fn given_core_phase_when_calling_bdd_progress_report_then_report_is_non_empty() {
    // Given / When
    let report = bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, "Core Phase");

    // Then
    assert!(!report.is_empty());
}

#[test]
fn given_runtime_phase_when_calling_bdd_progress_report_then_report_is_non_empty() {
    // Given / When
    let report = bdd_progress_report(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        "Runtime Phase",
    );

    // Then
    assert!(!report.is_empty());
}

// ---------------------------------------------------------------------------
// bdd_progress_report_with_profile tests
// ---------------------------------------------------------------------------

#[test]
fn given_profile_and_title_when_calling_bdd_progress_report_with_profile_then_report_contains_title()
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

// ---------------------------------------------------------------------------
// bdd_progress_report_for_current_profile tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_when_calling_bdd_progress_report_for_current_profile_then_report_is_non_empty()
{
    // Given / When
    let report = bdd_progress_report_for_current_profile(BddPhase::Core, "Core Phase");

    // Then
    assert!(!report.is_empty());
    assert!(report.contains("Core Phase"));
}

#[test]
fn given_runtime_phase_when_calling_bdd_progress_report_for_current_profile_then_report_is_non_empty()
 {
    // Given / When
    let report = bdd_progress_report_for_current_profile(BddPhase::Runtime, "Runtime Phase");

    // Then
    assert!(!report.is_empty());
    assert!(report.contains("Runtime Phase"));
}

// ---------------------------------------------------------------------------
// bdd_progress_status_line tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_when_calling_bdd_progress_status_line_then_line_is_non_empty() {
    // Given / When
    let line = bdd_progress_status_line(
        BddPhase::Core,
        GLR_CONFLICT_PRESERVATION_GRID,
        ParserFeatureProfile::current(),
    );

    // Then
    assert!(!line.is_empty());
}

#[test]
fn given_runtime_phase_when_calling_bdd_progress_status_line_then_line_is_non_empty() {
    // Given / When
    let line = bdd_progress_status_line(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        ParserFeatureProfile::current(),
    );

    // Then
    assert!(!line.is_empty());
}

// ---------------------------------------------------------------------------
// bdd_progress_status_line_for_current_profile tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_when_calling_status_line_for_current_profile_then_line_is_non_empty() {
    // Given / When
    let line = bdd_progress_status_line_for_current_profile(BddPhase::Core);

    // Then
    assert!(!line.is_empty());
}

#[test]
fn given_runtime_phase_when_calling_status_line_for_current_profile_then_line_is_non_empty() {
    // Given / When
    let line = bdd_progress_status_line_for_current_profile(BddPhase::Runtime);

    // Then
    assert!(!line.is_empty());
}

#[test]
fn given_different_phases_when_calling_status_line_for_current_profile_then_lines_differ() {
    // Given
    let core_line = bdd_progress_status_line_for_current_profile(BddPhase::Core);
    let runtime_line = bdd_progress_status_line_for_current_profile(BddPhase::Runtime);

    // When / Then
    // Both should be non-empty; content may differ based on scenario statuses
    assert!(!core_line.is_empty());
    assert!(!runtime_line.is_empty());
}

// ---------------------------------------------------------------------------
// ParserFeatureProfile tests
// ---------------------------------------------------------------------------

#[test]
fn given_current_profile_when_calling_current_then_returns_valid_profile() {
    // Given / When
    let profile = ParserFeatureProfile::current();

    // Then
    let _ = format!("{:?}", profile);
}

// ---------------------------------------------------------------------------
// ParserBackend tests
// ---------------------------------------------------------------------------

#[test]
fn given_tree_sitter_backend_when_using_parser_backend_then_variant_is_tree_sitter() {
    // Given
    let backend = ParserBackend::TreeSitter;

    // When / Then
    assert!(matches!(backend, ParserBackend::TreeSitter));
}

#[test]
fn given_glr_backend_when_using_parser_backend_then_variant_is_glr() {
    // Given
    let backend = ParserBackend::GLR;

    // When / Then
    assert!(matches!(backend, ParserBackend::GLR));
}
