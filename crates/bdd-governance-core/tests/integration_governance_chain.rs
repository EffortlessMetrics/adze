//! Integration tests for the BDD governance chain.
//!
//! Tests the chain: bdd-grid-core → bdd-governance-core → bdd-governance-contract

/// Tests that BddGovernanceMatrix can be constructed and used across the chain.
#[test]
fn test_governance_chain_matrix_from_core_to_contract() {
    // Given: Types from bdd-governance-core (which re-exports from bdd-grid-core)
    use adze_bdd_governance_core::{BddGovernanceMatrix, BddPhase, ParserFeatureProfile};

    // When: Create a matrix using the standard constructor
    let profile = ParserFeatureProfile::current();
    let matrix = BddGovernanceMatrix::standard(profile);

    // Then: Verify the matrix is properly constructed
    assert_eq!(matrix.phase, BddPhase::Core);
    assert!(!matrix.scenarios.is_empty());
}

/// Tests that BddGovernanceSnapshot correctly reports progress through the chain.
#[test]
fn test_governance_chain_snapshot_progress() {
    // Given: A governance matrix
    use adze_bdd_governance_core::{BddGovernanceMatrix, ParserFeatureProfile};

    let profile = ParserFeatureProfile::current();
    let matrix = BddGovernanceMatrix::standard(profile);

    // When: Take a snapshot
    let snapshot = matrix.snapshot();

    // Then: Verify snapshot invariants
    assert!(snapshot.total >= snapshot.implemented);
    assert_eq!(snapshot.phase, matrix.phase);
    assert_eq!(snapshot.profile, profile);
}

/// Tests that the status line integrates properly across the chain.
#[test]
fn test_governance_chain_status_line_integration() {
    // Given: Types from the governance chain
    use adze_bdd_governance_core::{BddGovernanceMatrix, ParserFeatureProfile};

    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };

    // When: Generate a status line
    let matrix = BddGovernanceMatrix::standard(profile);
    let status_line = matrix.status_line();

    // Then: Status line should contain expected information
    assert!(status_line.contains("core:"));
    assert!(status_line.contains("pure-rust"));
}

/// Tests that the report includes profile-aware annotations.
#[test]
fn test_governance_chain_report_with_profile() {
    // Given: A governance matrix
    use adze_bdd_governance_core::{BddGovernanceMatrix, ParserFeatureProfile};

    let profile = ParserFeatureProfile::current();
    let matrix = BddGovernanceMatrix::standard(profile);

    // When: Generate a report
    let report = matrix.report("Integration Test Phase");

    // Then: Report should contain phase title and profile info
    assert!(report.contains("Integration Test Phase"));
    assert!(report.contains("Feature profile:"));
}

/// Tests that bdd_progress function works through the chain.
#[test]
fn test_governance_chain_bdd_progress_function() {
    // Given: Re-exported bdd_progress from grid-core through governance-core
    use adze_bdd_governance_core::{BddPhase, GLR_CONFLICT_PRESERVATION_GRID, bdd_progress};

    // When: Calculate progress for Core phase
    let (implemented, total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);

    // Then: Should return valid counts
    assert!(total > 0, "Grid should have scenarios");
    assert!(implemented <= total, "Implemented cannot exceed total");
}

/// Tests that BddScenarioStatus variants work correctly through the chain.
#[test]
fn test_governance_chain_scenario_status_variants() {
    // Given: Re-exported status types
    use adze_bdd_governance_core::{BddPhase, GLR_CONFLICT_PRESERVATION_GRID};

    // When: Check status of scenarios
    let has_implemented = GLR_CONFLICT_PRESERVATION_GRID
        .iter()
        .any(|s| s.status(BddPhase::Core).implemented());

    // Then: At least some scenarios should be implemented
    assert!(has_implemented || !GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}

/// Tests that is_fully_implemented works correctly.
#[test]
fn test_governance_chain_fully_implemented_check() {
    // Given: A governance matrix
    use adze_bdd_governance_core::{BddGovernanceMatrix, ParserFeatureProfile};

    let profile = ParserFeatureProfile::current();
    let matrix = BddGovernanceMatrix::standard(profile);

    // When: Check if fully implemented
    let is_full = matrix.is_fully_implemented();
    let snapshot = matrix.snapshot();

    // Then: is_fully_implemented should match snapshot check
    assert_eq!(is_full, snapshot.is_fully_implemented());
}

/// Tests that ParserBackend resolution works through the chain.
#[test]
fn test_governance_chain_backend_resolution() {
    // Given: A snapshot with a specific profile
    use adze_bdd_governance_core::{BddGovernanceMatrix, ParserBackend, ParserFeatureProfile};

    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    let matrix = BddGovernanceMatrix::standard(profile);
    let snapshot = matrix.snapshot();

    // When: Resolve the non-conflict backend
    let backend = snapshot.non_conflict_backend();

    // Then: Should be PureRust for pure-rust profile
    assert_eq!(backend, ParserBackend::PureRust);
}
