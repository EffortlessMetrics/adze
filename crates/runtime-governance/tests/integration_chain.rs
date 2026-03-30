//! Cross-crate integration tests for the runtime governance chain:
//! `runtime-governance-api` → `runtime-governance-matrix` → `runtime-governance`
//!
//! These tests validate that the runtime governance chain works correctly end-to-end.

use adze_bdd_grid_core::BddPhase;
use adze_runtime_governance::{
    BddGovernanceSnapshot, bdd_governance_matrix_for_current_profile,
    bdd_governance_matrix_for_profile, bdd_governance_matrix_for_runtime,
    bdd_governance_matrix_for_runtime2, bdd_progress_report_for_current_profile,
    bdd_progress_status_line, bdd_status_line_for_current_profile, current_backend_for,
    parser_feature_profile_for_runtime, resolve_backend_for_profile, runtime_governance_snapshot,
};
use adze_runtime_governance_api as api;
use adze_runtime_governance_matrix as matrix;

/// Test that the runtime governance chain correctly propagates profile types.
#[test]
fn test_runtime_governance_chain_profile_consistency() {
    // Given: Profile from runtime-governance
    let profile = parser_feature_profile_for_runtime();

    // When: Getting profile through matrix
    let matrix_profile = matrix::parser_feature_profile_for_runtime();

    // Then: They should match
    assert_eq!(profile, matrix_profile);
}

/// Test that backend resolution is consistent through the chain.
#[test]
fn test_runtime_governance_chain_backend_resolution() {
    let profile = parser_feature_profile_for_runtime();

    // Resolve backend through different chain levels
    let governance_backend = current_backend_for(false);
    let profile_backend = resolve_backend_for_profile(profile, false);

    // They should match
    assert_eq!(governance_backend, profile_backend);
    assert_eq!(governance_backend, profile.resolve_backend(false));
}

/// Test that matrices are created consistently through the chain.
#[test]
fn test_runtime_governance_chain_matrix_creation() {
    let profile = parser_feature_profile_for_runtime();

    // Create matrices at different chain levels
    let governance_matrix = bdd_governance_matrix_for_profile(BddPhase::Core, profile);
    let matrix_matrix = matrix::bdd_governance_matrix_for_profile(BddPhase::Core, profile);

    // They should have the same properties
    assert_eq!(governance_matrix.phase, matrix_matrix.phase);
    assert_eq!(governance_matrix.profile, matrix_matrix.profile);
}

/// Test that runtime-specific matrix helpers work correctly.
#[test]
fn test_runtime_governance_chain_runtime_matrix() {
    let profile = parser_feature_profile_for_runtime();

    // Runtime matrix
    let runtime_matrix = bdd_governance_matrix_for_runtime();
    assert_eq!(runtime_matrix.phase, BddPhase::Runtime);
    assert_eq!(runtime_matrix.profile, profile);

    // Runtime2 matrix
    let runtime2_matrix = bdd_governance_matrix_for_runtime2(BddPhase::Core, profile.glr);
    assert_eq!(runtime2_matrix.phase, BddPhase::Core);
}

/// Test that current profile matrix helper works correctly.
#[test]
fn test_runtime_governance_chain_current_profile_matrix() {
    let matrix = bdd_governance_matrix_for_current_profile(BddPhase::Core);
    let profile = parser_feature_profile_for_runtime();

    assert_eq!(matrix.phase, BddPhase::Core);
    assert_eq!(matrix.profile, profile);
}

/// Test that snapshots are consistent through the chain.
#[test]
fn test_runtime_governance_chain_snapshot_consistency() {
    let profile = parser_feature_profile_for_runtime();

    // Create snapshot through runtime-governance
    let snapshot = runtime_governance_snapshot(BddPhase::Core);

    // Verify snapshot properties
    assert_eq!(snapshot.phase, BddPhase::Core);
    assert_eq!(snapshot.profile, profile);
    assert!(snapshot.total >= snapshot.implemented);
}

/// Test that progress reports are generated correctly through the chain.
#[test]
fn test_runtime_governance_chain_progress_report() {
    let title = "Runtime Governance Test";

    let report = bdd_progress_report_for_current_profile(BddPhase::Core, title);

    // Report should contain title and status information
    assert!(report.contains(title));
    assert!(report.contains("Feature profile:") || report.contains("Governance progress:"));
}

/// Test that status lines are generated correctly through the chain.
#[test]
fn test_runtime_governance_chain_status_line() {
    let profile = parser_feature_profile_for_runtime();

    // Status line through governance
    let status = bdd_progress_status_line(
        BddPhase::Core,
        adze_bdd_grid_core::GLR_CONFLICT_PRESERVATION_GRID,
        profile,
    );

    // Status line through current profile helper
    let current_status = bdd_status_line_for_current_profile(BddPhase::Core);

    // Both should be non-empty and start with phase name
    assert!(!status.is_empty());
    assert!(!current_status.is_empty());
    assert!(status.starts_with("core:"));
    assert!(current_status.starts_with("core:"));
}

/// Test that the chain handles different phases correctly.
#[test]
fn test_runtime_governance_chain_different_phases() {
    for phase in [BddPhase::Core, BddPhase::Runtime] {
        let matrix = bdd_governance_matrix_for_current_profile(phase);
        let snapshot = matrix.snapshot();

        assert_eq!(snapshot.phase, phase);
        assert!(snapshot.total >= snapshot.implemented);
    }
}

/// Test that the API facade correctly re-exports from runtime-governance.
#[test]
fn test_runtime_governance_chain_api_facade() {
    // The API facade should re-export all the same types
    let profile = parser_feature_profile_for_runtime();
    let api_profile = api::parser_feature_profile_for_runtime();

    assert_eq!(profile, api_profile);
}

/// Test that backend selection works correctly for conflict scenarios.
#[test]
fn test_runtime_governance_chain_conflict_backend() {
    let profile = parser_feature_profile_for_runtime();

    // Without conflicts
    let no_conflict_backend = current_backend_for(false);
    let expected = profile.resolve_backend(false);
    assert_eq!(no_conflict_backend, expected);

    // With conflicts - behavior depends on feature flags
    let conflict_backend = current_backend_for(true);
    let expected_conflict = profile.resolve_backend(true);
    assert_eq!(conflict_backend, expected_conflict);
}

/// Test that fully implemented check works through the chain.
#[test]
fn test_runtime_governance_chain_fully_implemented() {
    // Create a snapshot that is fully implemented
    let full_snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 5,
        total: 5,
        profile: parser_feature_profile_for_runtime(),
    };
    assert!(full_snapshot.is_fully_implemented());

    // Create a snapshot that is partially implemented
    let partial_snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 2,
        total: 5,
        profile: parser_feature_profile_for_runtime(),
    };
    assert!(!partial_snapshot.is_fully_implemented());
}

/// Test that matrix snapshot method matches standalone snapshot function.
#[test]
fn test_runtime_governance_chain_matrix_snapshot_method() {
    let profile = parser_feature_profile_for_runtime();
    let matrix = bdd_governance_matrix_for_profile(BddPhase::Core, profile);

    // Snapshot through matrix method
    let method_snapshot = matrix.snapshot();

    // Snapshot through standalone function
    let func_snapshot = runtime_governance_snapshot(BddPhase::Core);

    // They should have the same phase
    assert_eq!(method_snapshot.phase, func_snapshot.phase);
}
