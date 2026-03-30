//! Integration tests for the BDD contract chain.
//!
//! Tests the chain: bdd-grid-contract → bdd-contract

/// Tests that bdd-contract re-exports all necessary types from bdd-grid-contract.
#[test]
fn test_contract_chain_reexports_from_grid_contract() {
    // Given: Types re-exported through bdd-contract
    use adze_bdd_contract::{
        BddPhase, BddScenario, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID,
    };

    // When: Use the re-exported types
    let phase = BddPhase::Core;
    let status = BddScenarioStatus::Implemented;

    // Then: Types should work correctly
    assert_eq!(phase, BddPhase::Core);
    assert!(status.implemented());
    assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}

/// Tests that bdd_progress works through the contract chain.
#[test]
fn test_contract_chain_bdd_progress() {
    // Given: Re-exported bdd_progress function
    use adze_bdd_contract::{BddPhase, GLR_CONFLICT_PRESERVATION_GRID, bdd_progress};

    // When: Calculate progress for both phases
    let (core_impl, core_total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
    let (rt_impl, rt_total) = bdd_progress(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID);

    // Then: Should return valid counts
    assert!(core_impl <= core_total);
    assert!(rt_impl <= rt_total);
}

/// Tests that bdd_progress_report works through the contract chain.
#[test]
fn test_contract_chain_bdd_progress_report() {
    // Given: Re-exported bdd_progress_report function
    use adze_bdd_contract::{BddPhase, GLR_CONFLICT_PRESERVATION_GRID, bdd_progress_report};

    // When: Generate reports for both phases
    let core_report =
        bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, "Core Phase");
    let rt_report = bdd_progress_report(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        "Runtime Phase",
    );

    // Then: Reports should contain phase titles
    assert!(core_report.contains("Core Phase"));
    assert!(rt_report.contains("Runtime Phase"));
}

/// Tests that BddPhase variants work correctly.
#[test]
fn test_contract_chain_bdd_phase_variants() {
    // Given: Re-exported BddPhase
    use adze_bdd_contract::BddPhase;

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
fn test_contract_chain_scenario_status() {
    // Given: Re-exported BddScenarioStatus
    use adze_bdd_contract::BddScenarioStatus;

    // When: Create both status variants
    let implemented = BddScenarioStatus::Implemented;
    let deferred = BddScenarioStatus::Deferred {
        reason: "pending implementation",
    };

    // Then: Properties should be correct
    assert!(implemented.implemented());
    assert!(!deferred.implemented());
    assert_eq!(implemented.icon(), "✅");
    assert_eq!(deferred.icon(), "⏳");
    assert_eq!(implemented.label(), "IMPLEMENTED");
    assert_eq!(deferred.label(), "DEFERRED");
}

/// Tests that GLR_CONFLICT_PRESERVATION_GRID contains valid scenarios.
#[test]
fn test_contract_chain_grid_constant_valid() {
    // Given: The grid constant
    use adze_bdd_contract::{BddScenario, GLR_CONFLICT_PRESERVATION_GRID};

    // When: Inspect the scenarios
    assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());

    for scenario in GLR_CONFLICT_PRESERVATION_GRID {
        // Then: Each scenario should have non-empty title and reference
        assert!(!scenario.title.is_empty());
        assert!(!scenario.reference.is_empty());
    }
}

/// Tests that BddScenario struct is accessible and usable.
#[test]
fn test_contract_chain_bdd_scenario_struct() {
    // Given: Re-exported BddScenario
    use adze_bdd_contract::{
        BddPhase, BddScenario, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID,
    };

    // When: Access scenarios from the grid
    let first_scenario = &GLR_CONFLICT_PRESERVATION_GRID[0];

    // Then: Scenario fields should be accessible
    assert!(!first_scenario.title.is_empty());
    assert!(!first_scenario.reference.is_empty());
    // Status should be one of the valid variants
    let status = first_scenario.status(BddPhase::Core);
    match status {
        BddScenarioStatus::Implemented => {
            assert!(status.implemented());
        }
        BddScenarioStatus::Deferred { reason: _ } => {
            assert!(!status.implemented());
        }
    }
}

/// Tests that progress counts are consistent between calls.
#[test]
fn test_contract_chain_progress_consistency() {
    // Given: Re-exported bdd_progress function
    use adze_bdd_contract::{BddPhase, GLR_CONFLICT_PRESERVATION_GRID, bdd_progress};

    // When: Calculate progress multiple times
    let (impl1, total1) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
    let (impl2, total2) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);

    // Then: Results should be consistent
    assert_eq!(impl1, impl2);
    assert_eq!(total1, total2);
}
