//! BDD-style tests for bdd-grid-core crate.
//!
//! Tests follow the Given/When/Then pattern to verify public API behavior.

use adze_bdd_grid_core::{
    BddPhase, BddScenario, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID, bdd_progress,
    bdd_progress_report,
};

// ---------------------------------------------------------------------------
// BddPhase tests
// ---------------------------------------------------------------------------

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
fn given_phase_when_formatting_debug_then_contains_phase_name() {
    // Given
    let core = BddPhase::Core;

    // When
    let debug_str = format!("{:?}", core);

    // Then
    assert!(debug_str.contains("Core"));
}

#[test]
fn given_phase_when_cloning_then_equals_original() {
    // Given
    let core = BddPhase::Core;

    // When
    let cloned = core;

    // Then
    assert_eq!(core, cloned);
}

// ---------------------------------------------------------------------------
// BddScenarioStatus tests
// ---------------------------------------------------------------------------

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
fn given_implemented_status_when_calling_label_then_returns_uppercase() {
    // Given
    let status = BddScenarioStatus::Implemented;

    // When
    let label = status.label();

    // Then
    assert_eq!(label, "IMPLEMENTED");
}

#[test]
fn given_deferred_status_when_calling_label_then_returns_uppercase() {
    // Given
    let status = BddScenarioStatus::Deferred { reason: "wip" };

    // When
    let label = status.label();

    // Then
    assert_eq!(label, "DEFERRED");
}

#[test]
fn given_deferred_status_when_calling_detail_then_returns_reason() {
    // Given
    let status = BddScenarioStatus::Deferred {
        reason: "not ready",
    };

    // When
    let detail = status.detail();

    // Then
    assert_eq!(detail, "not ready");
}

#[test]
fn given_implemented_status_when_calling_detail_then_returns_empty() {
    // Given
    let status = BddScenarioStatus::Implemented;

    // When
    let detail = status.detail();

    // Then
    assert_eq!(detail, "");
}

#[test]
fn given_implemented_status_when_formatting_display_then_contains_implemented() {
    // Given
    let status = BddScenarioStatus::Implemented;

    // When
    let display = format!("{}", status);

    // Then
    assert!(display.contains("Implemented"));
}

#[test]
fn given_deferred_status_when_formatting_display_then_contains_deferred() {
    // Given
    let status = BddScenarioStatus::Deferred { reason: "later" };

    // When
    let display = format!("{}", status);

    // Then
    assert!(display.contains("Deferred"));
}

// ---------------------------------------------------------------------------
// BddScenario tests
// ---------------------------------------------------------------------------

#[test]
fn given_scenario_when_getting_status_for_core_phase_then_returns_core_status() {
    // Given
    let scenario = &GLR_CONFLICT_PRESERVATION_GRID[0];

    // When
    let status = scenario.status(BddPhase::Core);

    // Then
    assert_eq!(status, scenario.core_status);
}

#[test]
fn given_scenario_when_getting_status_for_runtime_phase_then_returns_runtime_status() {
    // Given
    let scenario = &GLR_CONFLICT_PRESERVATION_GRID[0];

    // When
    let status = scenario.status(BddPhase::Runtime);

    // Then
    assert_eq!(status, scenario.runtime_status);
}

#[test]
fn given_scenario_when_formatting_display_then_contains_id_and_title() {
    // Given
    let scenario = &GLR_CONFLICT_PRESERVATION_GRID[0];

    // When
    let display = format!("{}", scenario);

    // Then
    assert!(display.contains(&scenario.id.to_string()));
    assert!(display.contains(scenario.title));
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

#[test]
fn given_both_phases_when_counting_progress_then_totals_match() {
    // Given / When
    let (_, core_total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
    let (_, runtime_total) = bdd_progress(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID);

    // Then
    assert_eq!(core_total, runtime_total);
    assert_eq!(core_total, GLR_CONFLICT_PRESERVATION_GRID.len());
}

#[test]
fn given_all_implemented_scenarios_when_counting_progress_then_all_implemented() {
    // Given
    let scenarios = [BddScenario {
        id: 1,
        title: "test",
        reference: "ref",
        core_status: BddScenarioStatus::Implemented,
        runtime_status: BddScenarioStatus::Implemented,
    }];

    // When
    let (implemented, total) = bdd_progress(BddPhase::Core, &scenarios);

    // Then
    assert_eq!(implemented, 1);
    assert_eq!(total, 1);
}

#[test]
fn given_all_deferred_scenarios_when_counting_progress_then_none_implemented() {
    // Given
    let scenarios = [BddScenario {
        id: 1,
        title: "test",
        reference: "ref",
        core_status: BddScenarioStatus::Deferred { reason: "wip" },
        runtime_status: BddScenarioStatus::Deferred { reason: "wip" },
    }];

    // When
    let (implemented, total) = bdd_progress(BddPhase::Core, &scenarios);

    // Then
    assert_eq!(implemented, 0);
    assert_eq!(total, 1);
}

// ---------------------------------------------------------------------------
// bdd_progress_report tests
// ---------------------------------------------------------------------------

#[test]
fn given_title_when_calling_bdd_progress_report_then_report_contains_title() {
    // Given
    let title = "Grid Core Test Report";

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
fn given_empty_scenarios_when_calling_progress_report_then_shows_zero_counts() {
    // Given
    let scenarios: &[BddScenario] = &[];

    // When
    let report = bdd_progress_report(BddPhase::Core, scenarios, "Empty");

    // Then
    assert!(report.contains("Empty"));
    assert!(report.contains("0/0"));
}

#[test]
fn given_grid_when_calling_progress_report_then_contains_scenario_info() {
    // Given / When
    let report = bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, "Report");

    // Then
    assert!(report.contains("Scenario"));
}

// ---------------------------------------------------------------------------
// GLR_CONFLICT_PRESERVATION_GRID tests
// ---------------------------------------------------------------------------

#[test]
fn given_grid_constant_when_checking_then_is_non_empty() {
    // Given / When / Then
    assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}

#[test]
fn given_grid_constant_when_counting_then_has_expected_count() {
    // Given / When
    let count = GLR_CONFLICT_PRESERVATION_GRID.len();

    // Then
    // The grid should have 8 scenarios based on the BDD plan
    assert_eq!(count, 8);
}

#[test]
fn given_grid_scenarios_when_checking_ids_then_are_unique() {
    // Given
    let ids: std::collections::HashSet<u8> = GLR_CONFLICT_PRESERVATION_GRID
        .iter()
        .map(|s| s.id)
        .collect();

    // When / Then
    assert_eq!(ids.len(), GLR_CONFLICT_PRESERVATION_GRID.len());
}
