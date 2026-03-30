//! Integration tests for the BDD scenario fixtures chain.
//!
//! Tests the chain: bdd-governance-fixtures + bdd-grammar-fixtures → bdd-scenario-fixtures

/// Tests that scenario fixtures properly re-export governance types.
#[test]
fn test_scenario_chain_reexports_governance_types() {
    // Given: Types re-exported through scenario-fixtures
    use adze_bdd_scenario_fixtures::{
        BddPhase, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID, ParserFeatureProfile,
    };

    // When: Use the re-exported types
    let phase = BddPhase::Core;
    let status = BddScenarioStatus::Implemented;
    let profile = ParserFeatureProfile::current();

    // Then: All types should work correctly
    assert_eq!(phase, BddPhase::Core);
    assert!(status.implemented());
    assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}

/// Tests that bdd_progress works through the scenario fixtures chain.
#[test]
fn test_scenario_chain_bdd_progress() {
    // Given: Re-exported bdd_progress function
    use adze_bdd_scenario_fixtures::{BddPhase, GLR_CONFLICT_PRESERVATION_GRID, bdd_progress};

    // When: Calculate progress for both phases
    let (core_impl, core_total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
    let (rt_impl, rt_total) = bdd_progress(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID);

    // Then: Should return valid counts
    assert!(core_impl <= core_total);
    assert!(rt_impl <= rt_total);
}

/// Tests that bdd_progress_report_for_current_profile works through the chain.
#[test]
fn test_scenario_chain_current_profile_report() {
    // Given: Re-exported report function for current profile
    use adze_bdd_scenario_fixtures::{BddPhase, bdd_progress_report_for_current_profile};

    // When: Generate reports for both phases
    let core_report = bdd_progress_report_for_current_profile(BddPhase::Core, "Core Scenarios");
    let rt_report = bdd_progress_report_for_current_profile(BddPhase::Runtime, "Runtime Scenarios");

    // Then: Reports should contain phase titles
    assert!(core_report.contains("Core Scenarios"));
    assert!(rt_report.contains("Runtime Scenarios"));
}

/// Tests that bdd_progress_status_line_for_current_profile works through the chain.
#[test]
fn test_scenario_chain_current_profile_status_line() {
    // Given: Re-exported status line function
    use adze_bdd_scenario_fixtures::{BddPhase, bdd_progress_status_line_for_current_profile};

    // When: Generate status lines
    let core_line = bdd_progress_status_line_for_current_profile(BddPhase::Core);
    let rt_line = bdd_progress_status_line_for_current_profile(BddPhase::Runtime);

    // Then: Lines should be non-empty
    assert!(!core_line.is_empty());
    assert!(!rt_line.is_empty());
}

/// Tests that ParserBackend is accessible and works correctly.
#[test]
fn test_scenario_chain_parser_backend() {
    // Given: Re-exported ParserBackend and ParserFeatureProfile
    use adze_bdd_scenario_fixtures::{ParserBackend, ParserFeatureProfile};

    // When: Create profile and resolve backend
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    let backend = profile.resolve_backend(false);

    // Then: Backend should be PureRust
    assert_eq!(backend, ParserBackend::PureRust);
}

/// Tests that all BddPhase variants are accessible.
#[test]
fn test_scenario_chain_bdd_phase_variants() {
    // Given: Re-exported BddPhase
    use adze_bdd_scenario_fixtures::BddPhase;

    // When: Use both variants
    let core = BddPhase::Core;
    let runtime = BddPhase::Runtime;

    // Then: They should be distinct and display correctly
    assert_ne!(core, runtime);
    assert_eq!(format!("{}", core), "Core");
    assert_eq!(format!("{}", runtime), "Runtime");
}

/// Tests that BddScenarioStatus variants work correctly.
#[test]
fn test_scenario_chain_scenario_status() {
    // Given: Re-exported BddScenarioStatus
    use adze_bdd_scenario_fixtures::BddScenarioStatus;

    // When: Create both status variants
    let implemented = BddScenarioStatus::Implemented;
    let deferred = BddScenarioStatus::Deferred {
        reason: "pending work",
    };

    // Then: implemented() should return correct values
    assert!(implemented.implemented());
    assert!(!deferred.implemented());
    assert_eq!(implemented.icon(), "✅");
    assert_eq!(deferred.icon(), "⏳");
}

/// Tests that bdd_progress_report_with_profile integrates properly.
#[test]
fn test_scenario_chain_report_with_profile() {
    // Given: Re-exported report function with profile
    use adze_bdd_scenario_fixtures::{
        BddPhase, GLR_CONFLICT_PRESERVATION_GRID, ParserFeatureProfile,
        bdd_progress_report_with_profile,
    };

    let profile = ParserFeatureProfile::current();

    // When: Generate a report with explicit profile
    let report = bdd_progress_report_with_profile(
        BddPhase::Core,
        GLR_CONFLICT_PRESERVATION_GRID,
        "Integration Test",
        profile,
    );

    // Then: Report should contain expected sections
    assert!(report.contains("Integration Test"));
    assert!(report.contains("Feature profile:"));
}

/// Tests that the grid constant contains valid scenarios.
#[test]
fn test_scenario_chain_grid_constant_valid() {
    // Given: The grid constant
    use adze_bdd_scenario_fixtures::{BddPhase, GLR_CONFLICT_PRESERVATION_GRID};

    // When: Inspect the scenarios
    for scenario in GLR_CONFLICT_PRESERVATION_GRID {
        // Then: Each scenario should have non-empty title and reference
        assert!(!scenario.title.is_empty());
        assert!(!scenario.reference.is_empty());
    }
}
