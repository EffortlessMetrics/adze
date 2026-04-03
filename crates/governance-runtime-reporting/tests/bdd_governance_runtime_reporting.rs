//! BDD-style tests for governance-runtime-reporting crate.
//!
//! Tests follow the Given/When/Then pattern to verify runtime report
//! formatting and re-exported governance matrix types.

use adze_governance_runtime_reporting::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, GLR_CONFLICT_FALLBACK,
    GLR_CONFLICT_PRESERVATION_GRID, ParserBackend, ParserFeatureProfile, bdd_governance_snapshot,
    bdd_progress, bdd_progress_report, bdd_progress_report_with_profile,
    bdd_progress_report_with_profile_runtime, bdd_progress_status_line,
    describe_backend_for_conflicts,
};

// ---------------------------------------------------------------------------
// Re-exported Type Tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_and_runtime_phases_when_comparing_then_phases_differ() {
    // Given
    let core = BddPhase::Core;
    let runtime = BddPhase::Runtime;

    // When/Then
    assert_ne!(core, runtime);
}

#[test]
fn given_re_exported_bdd_governance_snapshot_when_created_then_has_expected_fields() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 3,
        total: 5,
        profile,
    };

    // Then
    assert_eq!(snapshot.phase, BddPhase::Core);
    assert_eq!(snapshot.implemented, 3);
    assert_eq!(snapshot.total, 5);
}

#[test]
fn given_re_exported_matrix_when_created_standard_then_has_scenarios() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let matrix = BddGovernanceMatrix::standard(profile);

    // Then
    assert!(!matrix.scenarios.is_empty());
}

// ---------------------------------------------------------------------------
// bdd_progress Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_with_grid_scenarios_when_calling_bdd_progress_then_returns_valid_counts() {
    // Given/When
    let (implemented, total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);

    // Then
    assert!(total > 0);
    assert!(implemented <= total);
}

#[test]
fn given_runtime_phase_with_grid_scenarios_when_calling_bdd_progress_then_returns_valid_counts() {
    // Given/When
    let (implemented, total) = bdd_progress(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID);

    // Then
    assert!(total > 0);
    assert!(implemented <= total);
}

#[test]
fn given_empty_scenarios_when_calling_bdd_progress_then_returns_zero_counts() {
    // Given
    let scenarios: &[adze_governance_runtime_reporting::BddScenario] = &[];

    // When
    let (implemented, total) = bdd_progress(BddPhase::Core, scenarios);

    // Then
    assert_eq!(implemented, 0);
    assert_eq!(total, 0);
}

// ---------------------------------------------------------------------------
// bdd_governance_snapshot Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_phase_and_scenarios_when_creating_snapshot_then_snapshot_matches_inputs() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let snapshot = bdd_governance_snapshot(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Then
    assert_eq!(snapshot.phase, BddPhase::Core);
    assert_eq!(snapshot.profile, profile);
}

// ---------------------------------------------------------------------------
// bdd_progress_report Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_title_when_generating_progress_report_then_report_contains_title() {
    // Given
    let title = "Test Progress Report";

    // When
    let report = bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, title);

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_runtime_phase_when_generating_progress_report_then_report_is_not_empty() {
    // Given
    let title = "Runtime";

    // When
    let report = bdd_progress_report(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, title);

    // Then
    assert!(!report.is_empty());
}

// ---------------------------------------------------------------------------
// bdd_progress_report_with_profile Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_profile_and_title_when_generating_report_then_contains_both() {
    // Given
    let profile = ParserFeatureProfile::current();
    let title = "Profile-Aware Report";

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

// ---------------------------------------------------------------------------
// bdd_progress_report_with_profile_runtime Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_runtime_report_params_when_generating_report_then_contains_all_sections() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };
    let title = "Runtime Governance";

    // When
    let report = bdd_progress_report_with_profile_runtime(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        title,
        profile,
    );

    // Then
    assert!(report.contains(title));
    assert!(report.contains("Governance status:"));
    assert!(report.contains("Feature profile:"));
    assert!(report.contains("Non-conflict backend:"));
    assert!(report.contains("Conflict profiles:"));
}

#[test]
fn given_empty_scenarios_for_runtime_report_when_generating_then_shows_zero_counts() {
    // Given
    let profile = ParserFeatureProfile::current();
    let scenarios: &[adze_governance_runtime_reporting::BddScenario] = &[];

    // When
    let report =
        bdd_progress_report_with_profile_runtime(BddPhase::Core, scenarios, "Empty Test", profile);

    // Then
    assert!(report.contains("Empty Test"));
    assert!(report.contains("Governance status: 0/0"));
}

#[test]
fn given_glr_profile_for_runtime_report_when_generating_then_shows_glr_backend() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When
    let report = bdd_progress_report_with_profile_runtime(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        "GLR Test",
        profile,
    );

    // Then
    assert!(report.contains("Non-conflict backend:"));
}

// ---------------------------------------------------------------------------
// bdd_progress_status_line Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_when_generating_status_line_then_starts_with_core_prefix() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let status = bdd_progress_status_line(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Then
    assert!(status.starts_with("core:"));
}

#[test]
fn given_runtime_phase_when_generating_status_line_then_starts_with_runtime_prefix() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let status =
        bdd_progress_status_line(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Then
    assert!(status.starts_with("runtime:"));
}

#[test]
fn given_profile_when_generating_status_line_then_contains_profile_string() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };

    // When
    let status = bdd_progress_status_line(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Then
    assert!(status.contains(&format!("{}", profile)));
}

// ---------------------------------------------------------------------------
// describe_backend_for_conflicts Function Tests
// ---------------------------------------------------------------------------

#[test]
fn given_any_profile_when_describing_backend_for_conflicts_then_returns_non_empty_string() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let description = describe_backend_for_conflicts(profile);

    // Then
    assert!(!description.is_empty());
}

#[test]
fn given_glr_enabled_profile_when_describing_backend_then_description_is_valid() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When
    let description = describe_backend_for_conflicts(profile);

    // Then
    assert!(!description.is_empty());
}

// ---------------------------------------------------------------------------
// GLR_CONFLICT_FALLBACK Constant Tests
// ---------------------------------------------------------------------------

#[test]
fn given_glr_conflict_fallback_constant_when_checking_then_not_empty() {
    // Given/When
    let fallback = GLR_CONFLICT_FALLBACK;

    // Then
    assert!(!fallback.is_empty());
}

#[test]
fn given_glr_conflict_fallback_when_checking_content_then_contains_expected_keywords() {
    // Given/When
    let fallback = GLR_CONFLICT_FALLBACK;

    // Then - It's a description string about GLR fallback behavior
    assert!(fallback.contains("GLR") || fallback.contains("glr") || fallback.contains("conflict"));
}

// ---------------------------------------------------------------------------
// GLR_CONFLICT_PRESERVATION_GRID Constant Tests
// ---------------------------------------------------------------------------

#[test]
fn given_conflict_preservation_grid_when_checking_then_not_empty() {
    // Given/When
    let grid = GLR_CONFLICT_PRESERVATION_GRID;

    // Then
    assert!(!grid.is_empty());
}

// ---------------------------------------------------------------------------
// ParserBackend Integration Tests
// ---------------------------------------------------------------------------

#[test]
fn given_glr_backend_when_getting_name_then_returns_expected_string() {
    // Given
    let backend = ParserBackend::GLR;

    // When
    let name = backend.name();

    // Then
    assert!(!name.is_empty());
}

#[test]
fn given_tree_sitter_backend_when_getting_name_then_returns_expected_string() {
    // Given
    let backend = ParserBackend::TreeSitter;

    // When
    let name = backend.name();

    // Then
    assert!(!name.is_empty());
}

// ---------------------------------------------------------------------------
// ParserFeatureProfile Integration Tests
// ---------------------------------------------------------------------------

#[test]
fn given_pure_rust_profile_when_resolving_non_conflict_backend_then_returns_expected() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When
    let backend = profile.resolve_backend(false);

    // Then
    assert_eq!(backend, ParserBackend::GLR);
}

#[test]
fn given_tree_sitter_profile_when_resolving_non_conflict_backend_then_returns_expected() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: true,
        tree_sitter_c2rust: false,
        glr: false,
    };

    // When
    let backend = profile.resolve_backend(false);

    // Then
    assert_eq!(backend, ParserBackend::TreeSitter);
}

#[test]
fn given_current_profile_when_creating_then_all_fields_are_accessible() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When/Then - Just verify we can access all fields (compile-time check)
    let _ = profile.pure_rust;
    let _ = profile.tree_sitter_standard;
    let _ = profile.tree_sitter_c2rust;
    let _ = profile.glr;
}
