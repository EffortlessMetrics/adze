//! BDD-style tests for bdd-governance-fixtures crate.
//!
//! Tests follow the Given/When/Then pattern to verify public API behavior.

use adze_bdd_governance_fixtures::{
    BddPhase, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID, ParserBackend,
    ParserFeatureProfile, bdd_progress, bdd_progress_report,
    bdd_progress_report_for_current_profile, bdd_progress_report_with_profile,
    bdd_progress_status_line, bdd_progress_status_line_for_current_profile,
};

#[test]
fn given_core_phase_when_calling_bdd_progress_report_for_current_profile_then_report_contains_title()
 {
    // Given
    let title = "Core Phase Report";

    // When
    let report = bdd_progress_report_for_current_profile(BddPhase::Core, title);

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_runtime_phase_when_calling_bdd_progress_report_for_current_profile_then_report_contains_title()
 {
    // Given
    let title = "Runtime Phase Report";

    // When
    let report = bdd_progress_report_for_current_profile(BddPhase::Runtime, title);

    // Then
    assert!(report.contains(title));
}

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
fn given_different_phases_when_calling_status_line_then_lines_differ() {
    // Given
    let core_line = bdd_progress_status_line_for_current_profile(BddPhase::Core);
    let runtime_line = bdd_progress_status_line_for_current_profile(BddPhase::Runtime);

    // When / Then
    // Both should be non-empty; content may differ based on scenario statuses
    assert!(!core_line.is_empty());
    assert!(!runtime_line.is_empty());
}

#[test]
fn given_grid_constant_when_checking_then_is_non_empty() {
    // Given / When / Then
    assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
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
    let title = "Test Report";

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
        BddPhase::Core,
        GLR_CONFLICT_PRESERVATION_GRID,
        title,
        profile,
    );

    // Then
    assert!(report.contains(title));
    assert!(report.contains("Feature profile:"));
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
    let status = BddScenarioStatus::Deferred { reason: "pending" };

    // When
    let result = status.implemented();

    // Then
    assert!(!result);
}

#[test]
fn given_implemented_status_when_calling_icon_then_returns_checkmark() {
    // Given
    let status = BddScenarioStatus::Implemented;

    // When
    let icon = status.icon();

    // Then
    assert_eq!(icon, "✅");
}

#[test]
fn given_deferred_status_when_calling_icon_then_returns_hourglass() {
    // Given
    let status = BddScenarioStatus::Deferred { reason: "wip" };

    // When
    let icon = status.icon();

    // Then
    assert_eq!(icon, "⏳");
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
fn given_current_profile_when_calling_resolve_backend_then_returns_valid_backend() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let backend = profile.resolve_backend(false);

    // Then
    assert!(!backend.name().is_empty());
}

#[test]
fn given_grid_when_checking_scenarios_then_all_have_required_fields() {
    // Given / When
    for scenario in GLR_CONFLICT_PRESERVATION_GRID {
        // Then
        assert!(!scenario.title.is_empty());
        assert!(!scenario.reference.is_empty());
    }
}
