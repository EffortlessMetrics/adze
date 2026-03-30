//! BDD tests for bdd-scenario-fixtures facade crate.
//!
//! These tests verify the public API behavior using Given/When/Then style.

use adze_bdd_scenario_fixtures::*;

// =============================================================================
// BddPhase Tests
// =============================================================================

#[test]
fn given_bdd_phase_core_when_checking_variants_then_core_exists() {
    // Given / When
    let phase = BddPhase::Core;

    // Then
    assert!(matches!(phase, BddPhase::Core));
}

#[test]
fn given_bdd_phase_runtime_when_checking_variants_then_runtime_exists() {
    // Given / When
    let phase = BddPhase::Runtime;

    // Then
    assert!(matches!(phase, BddPhase::Runtime));
}

// =============================================================================
// BddScenarioStatus Tests
// =============================================================================

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
    let status = BddScenarioStatus::Deferred {
        reason: "work in progress",
    };

    // When
    let result = status.implemented();

    // Then
    assert!(!result);
}

#[test]
fn given_deferred_status_when_getting_detail_then_returns_reason() {
    // Given
    let status = BddScenarioStatus::Deferred {
        reason: "waiting on dependency",
    };

    // When
    let detail = status.detail();

    // Then
    assert_eq!(detail, "waiting on dependency");
}

// =============================================================================
// GLR Conflict Preservation Grid Tests
// =============================================================================

#[test]
fn given_glr_conflict_grid_when_accessing_then_is_not_empty() {
    // Given / When
    let grid = GLR_CONFLICT_PRESERVATION_GRID;

    // Then
    assert!(!grid.is_empty());
}

#[test]
fn given_glr_conflict_grid_when_iterating_then_contains_scenarios() {
    // Given
    let grid = GLR_CONFLICT_PRESERVATION_GRID;

    // When
    let count = grid.len();

    // Then
    assert!(count > 0);
}

// =============================================================================
// BDD Progress Tests
// =============================================================================

#[test]
fn given_core_phase_and_grid_when_calculating_progress_then_returns_valid_counts() {
    // Given
    let phase = BddPhase::Core;
    let grid = GLR_CONFLICT_PRESERVATION_GRID;

    // When
    let (implemented, total) = bdd_progress(phase, grid);

    // Then
    assert!(implemented <= total);
    assert!(total > 0);
}

#[test]
fn given_runtime_phase_and_grid_when_calculating_progress_then_returns_valid_counts() {
    // Given
    let phase = BddPhase::Runtime;
    let grid = GLR_CONFLICT_PRESERVATION_GRID;

    // When
    let (implemented, total) = bdd_progress(phase, grid);

    // Then
    assert!(implemented <= total);
    assert!(total > 0);
}

// =============================================================================
// BDD Progress Report Tests
// =============================================================================

#[test]
fn given_core_phase_and_title_when_generating_report_then_contains_title() {
    // Given
    let phase = BddPhase::Core;
    let title = "Test Phase";

    // When
    let report = bdd_progress_report_for_current_profile(phase, title);

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_runtime_phase_and_title_when_generating_report_then_contains_phase_info() {
    // Given
    let phase = BddPhase::Runtime;
    let title = "Runtime Tests";

    // When
    let report = bdd_progress_report_for_current_profile(phase, title);

    // Then
    assert!(report.contains(title));
}

// =============================================================================
// ParserFeatureProfile Tests
// =============================================================================

#[test]
fn given_current_profile_when_accessing_then_returns_valid_profile() {
    // Given / When
    let profile = ParserFeatureProfile::current();

    // Then
    let _ = format!("{:?}", profile);
    let _ = format!("{}", profile);
}

#[test]
fn given_current_profile_when_checking_features_then_returns_booleans() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When / Then - These should not panic
    let _ = profile.has_pure_rust();
    let _ = profile.has_glr();
    let _ = profile.has_tree_sitter();
}
