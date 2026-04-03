//! Cross-crate integration tests for the governance matrix chain:
//! `governance-contract` → `governance-matrix-contract` → `governance-matrix-core` → `governance-matrix-core-impl`
//!
//! These tests validate that the governance matrix chain works correctly end-to-end.

use adze_bdd_governance_core::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, ParserFeatureProfile,
    bdd_governance_snapshot, bdd_progress, bdd_progress_report_with_profile,
    bdd_progress_status_line,
};
use adze_governance_contract::{
    BddGovernanceMatrix as ContractMatrix, BddPhase as ContractPhase,
    ParserFeatureProfile as ContractProfile,
};
use adze_governance_matrix_core as matrix_core;

/// Test that the governance chain correctly propagates types through all layers.
#[test]
fn test_governance_chain_types_are_compatible() {
    // Given: A profile from the core implementation
    let profile = ParserFeatureProfile::current();

    // When: Creating matrices at different chain levels
    let core_matrix = BddGovernanceMatrix::standard(profile);
    let contract_matrix = ContractMatrix::standard(ContractProfile::current());

    // Then: They should have equivalent phases
    assert_eq!(core_matrix.phase, BddPhase::Core);
    assert_eq!(contract_matrix.phase, ContractPhase::Core);
}

/// Test that snapshots are consistent across the chain.
#[test]
fn test_governance_chain_snapshot_consistency() {
    let profile = ParserFeatureProfile::current();

    // Create snapshot directly from core
    let core_snapshot = bdd_governance_snapshot(
        BddPhase::Core,
        adze_bdd_governance_core::GLR_CONFLICT_PRESERVATION_GRID,
        profile,
    );

    // Create snapshot through matrix
    let matrix = BddGovernanceMatrix::standard(profile);
    let matrix_snapshot = matrix.snapshot();

    // Both should have the same phase and profile
    assert_eq!(core_snapshot.phase, matrix_snapshot.phase);
    assert_eq!(core_snapshot.profile, matrix_snapshot.profile);
    assert_eq!(core_snapshot.total, matrix_snapshot.total);
    assert_eq!(core_snapshot.implemented, matrix_snapshot.implemented);
}

/// Test that progress reporting is consistent through the chain.
#[test]
fn test_governance_chain_progress_reporting() {
    let profile = ParserFeatureProfile::current();

    // Get progress from core function
    let (implemented, total) = bdd_progress(
        BddPhase::Core,
        adze_bdd_governance_core::GLR_CONFLICT_PRESERVATION_GRID,
    );

    // Get progress through matrix snapshot
    let matrix = BddGovernanceMatrix::standard(profile);
    let snapshot = matrix.snapshot();

    // They should match
    assert_eq!(implemented, snapshot.implemented);
    assert_eq!(total, snapshot.total);
    assert!(total >= implemented);
}

/// Test that status line generation works through the chain.
#[test]
fn test_governance_chain_status_line() {
    let profile = ParserFeatureProfile::current();

    // Generate status line directly
    let status = bdd_progress_status_line(
        BddPhase::Core,
        adze_bdd_governance_core::GLR_CONFLICT_PRESERVATION_GRID,
        profile,
    );

    // Generate through matrix
    let matrix = BddGovernanceMatrix::standard(profile);
    let matrix_status = matrix.status_line();

    // Both should be non-empty and start with phase name
    assert!(!status.is_empty());
    assert!(!matrix_status.is_empty());
    assert!(status.starts_with("core:"));
    assert!(matrix_status.starts_with("core:"));
}

/// Test that the matrix contract re-exports match core implementation.
#[test]
fn test_governance_chain_reexports_match() {
    let profile = ParserFeatureProfile::current();

    // matrix_core re-exports from matrix_core_impl which re-exports from bdd_governance_core
    let core_matrix = BddGovernanceMatrix::standard(profile);
    let matrix_core_matrix = matrix_core::BddGovernanceMatrix::standard(profile);

    assert_eq!(core_matrix.phase, matrix_core_matrix.phase);
    assert_eq!(core_matrix.profile, matrix_core_matrix.profile);
}

/// Test that fully implemented check works through the chain.
#[test]
fn test_governance_chain_fully_implemented() {
    // Create a snapshot that is fully implemented
    let full_snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 5,
        total: 5,
        profile: ParserFeatureProfile::current(),
    };
    assert!(full_snapshot.is_fully_implemented());

    // Create a snapshot that is partially implemented
    let partial_snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 2,
        total: 5,
        profile: ParserFeatureProfile::current(),
    };
    assert!(!partial_snapshot.is_fully_implemented());

    // Zero over zero is considered fully implemented
    let empty_snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 0,
        total: 0,
        profile: ParserFeatureProfile::current(),
    };
    assert!(empty_snapshot.is_fully_implemented());
}

/// Test that the chain correctly handles different phases.
#[test]
fn test_governance_chain_phases() {
    let profile = ParserFeatureProfile::current();

    for phase in [BddPhase::Core, BddPhase::Runtime] {
        let matrix = BddGovernanceMatrix::new(
            phase,
            profile,
            adze_bdd_governance_core::GLR_CONFLICT_PRESERVATION_GRID,
        );
        let snapshot = matrix.snapshot();

        assert_eq!(snapshot.phase, phase);
        assert!(snapshot.total >= snapshot.implemented);
    }
}

/// Test that reports contain expected content through the chain.
#[test]
fn test_governance_chain_report_content() {
    let profile = ParserFeatureProfile::current();
    let title = "Integration Test";

    let report = bdd_progress_report_with_profile(
        BddPhase::Core,
        adze_bdd_governance_core::GLR_CONFLICT_PRESERVATION_GRID,
        title,
        profile,
    );

    // Report should contain the title and governance progress
    assert!(report.contains(title));
    assert!(report.contains("Governance progress:"));
}
