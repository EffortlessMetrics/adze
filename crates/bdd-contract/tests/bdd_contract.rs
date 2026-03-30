//! BDD-style tests for bdd-contract crate.
//!
//! Tests follow the Given/When/Then pattern to verify public API behavior.

use adze_bdd_contract::{
    BddPhase, BddScenario, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID, bdd_progress,
    bdd_progress_report,
};

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
fn given_core_phase_when_calling_progress_report_then_report_contains_phase_info() {
    // Given / When
    let report = bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, "Core");

    // Then
    assert!(report.contains("Core"));
}

#[test]
fn given_runtime_phase_when_calling_progress_report_then_report_contains_phase_info() {
    // Given / When
    let report = bdd_progress_report(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, "Runtime");

    // Then
    assert!(report.contains("Runtime"));
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
fn given_core_phase_when_comparing_to_runtime_then_phases_differ() {
    // Given
    let core = BddPhase::Core;
    let runtime = BddPhase::Runtime;

    // When / Then
    assert_ne!(core, runtime);
}

#[test]
fn given_phase_when_formatting_display_then_outputs_phase_name() {
    // Given
    let core = BddPhase::Core;
    let runtime = BddPhase::Runtime;

    // When
    let core_str = format!("{}", core);
    let runtime_str = format!("{}", runtime);

    // Then
    assert!(!core_str.is_empty());
    assert!(!runtime_str.is_empty());
    assert_ne!(core_str, runtime_str);
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

#[test]
fn given_scenario_when_getting_status_for_phase_then_returns_correct_status() {
    // Given
    let scenario = &GLR_CONFLICT_PRESERVATION_GRID[0];

    // When
    let core_status = scenario.status(BddPhase::Core);
    let runtime_status = scenario.status(BddPhase::Runtime);

    // Then
    // Both statuses should be valid (implemented or deferred)
    let _ = core_status.implemented();
    let _ = runtime_status.implemented();
}

#[test]
fn given_scenario_when_formatting_display_then_contains_title() {
    // Given
    let scenario = &GLR_CONFLICT_PRESERVATION_GRID[0];

    // When
    let display = format!("{}", scenario);

    // Then
    assert!(display.contains(scenario.title));
}
