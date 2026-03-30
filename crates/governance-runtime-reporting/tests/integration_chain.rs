//! Cross-crate integration tests for the governance runtime chain:
//! `governance-metadata` → `governance-runtime-core` → `governance-runtime-reporting`
//!
//! These tests validate that the governance runtime chain works correctly end-to-end.

use adze_bdd_grid_core::BddPhase;
use adze_feature_policy_core::ParserFeatureProfile;
use adze_governance_metadata::{GovernanceMetadata, ParserFeatureProfileSnapshot};
use adze_governance_runtime_core::{
    bdd_governance_matrix_for_profile, bdd_governance_snapshot, bdd_progress_report_with_profile,
    bdd_progress_status_line, describe_backend_for_conflicts,
};

/// Test that the runtime chain correctly propagates profile types.
#[test]
fn test_runtime_chain_profile_types_are_compatible() {
    // Given: A profile from feature-policy-core
    let profile = ParserFeatureProfile::current();

    // When: Creating a snapshot from governance-metadata
    let metadata_snapshot = ParserFeatureProfileSnapshot::from_profile(profile);

    // Then: The snapshot should match the profile
    assert_eq!(metadata_snapshot.pure_rust, profile.pure_rust);
    assert_eq!(
        metadata_snapshot.tree_sitter_standard,
        profile.tree_sitter_standard
    );
    assert_eq!(
        metadata_snapshot.tree_sitter_c2rust,
        profile.tree_sitter_c2rust
    );
    assert_eq!(metadata_snapshot.glr, profile.glr);
}

/// Test that the runtime chain correctly creates governance matrices.
#[test]
fn test_runtime_chain_matrix_creation() {
    let profile = ParserFeatureProfile::current();

    // Create matrix through runtime-core
    let matrix = bdd_governance_matrix_for_profile(BddPhase::Core, profile);

    // Verify matrix properties
    assert_eq!(matrix.phase, BddPhase::Core);
    assert_eq!(matrix.profile, profile);
}

/// Test that snapshots are consistent through the runtime chain.
#[test]
fn test_runtime_chain_snapshot_consistency() {
    let profile = ParserFeatureProfile::current();

    // Create snapshot through runtime-core
    let snapshot = bdd_governance_snapshot(
        BddPhase::Core,
        adze_bdd_grid_core::GLR_CONFLICT_PRESERVATION_GRID,
        profile,
    );

    // Verify snapshot properties
    assert_eq!(snapshot.phase, BddPhase::Core);
    assert_eq!(snapshot.profile, profile);
    assert!(snapshot.total >= snapshot.implemented);
}

/// Test that progress reports are generated correctly through the chain.
#[test]
fn test_runtime_chain_progress_report() {
    let profile = ParserFeatureProfile::current();
    let title = "Runtime Chain Test";

    let report = bdd_progress_report_with_profile(
        BddPhase::Core,
        adze_bdd_grid_core::GLR_CONFLICT_PRESERVATION_GRID,
        title,
        profile,
    );

    // Report should contain title and status information
    assert!(report.contains(title));
    assert!(report.contains("Governance progress:"));
}

/// Test that status lines are generated correctly through the chain.
#[test]
fn test_runtime_chain_status_line() {
    let profile = ParserFeatureProfile::current();

    let status = bdd_progress_status_line(
        BddPhase::Core,
        adze_bdd_grid_core::GLR_CONFLICT_PRESERVATION_GRID,
        profile,
    );

    // Status line should start with phase name
    assert!(!status.is_empty());
    assert!(status.starts_with("core:"));
}

/// Test that backend description works through the chain.
#[test]
fn test_runtime_chain_backend_description() {
    let profile = ParserFeatureProfile::current();

    let description = describe_backend_for_conflicts(profile);

    // Description should be non-empty
    assert!(!description.is_empty());
}

/// Test that ParserFeatureProfileSnapshot from metadata is serializable.
#[test]
fn test_runtime_chain_metadata_snapshot_serialization() {
    let profile = ParserFeatureProfile::current();
    let snapshot = ParserFeatureProfileSnapshot::from_profile(profile);

    // Should be serializable to JSON
    let json = serde_json::to_string(&snapshot).expect("Should serialize to JSON");
    assert!(json.contains("pure_rust"));

    // Should be deserializable
    let deserialized: ParserFeatureProfileSnapshot =
        serde_json::from_str(&json).expect("Should deserialize from JSON");
    assert_eq!(deserialized, snapshot);
}

/// Test that the chain handles different phases correctly.
#[test]
fn test_runtime_chain_different_phases() {
    let profile = ParserFeatureProfile::current();

    for phase in [BddPhase::Core, BddPhase::Runtime] {
        let matrix = bdd_governance_matrix_for_profile(phase, profile);
        let snapshot = matrix.snapshot();

        assert_eq!(snapshot.phase, phase);
        assert!(snapshot.total >= snapshot.implemented);
    }
}

/// Test that GovernanceMetadata from metadata crate works correctly.
#[test]
fn test_runtime_chain_governance_metadata() {
    let profile = ParserFeatureProfile::current();

    // Create governance metadata
    let metadata = GovernanceMetadata::for_grid(
        BddPhase::Core,
        adze_bdd_grid_core::GLR_CONFLICT_PRESERVATION_GRID,
        profile,
    );

    // Verify metadata
    assert_eq!(metadata.phase, "core");
    assert!(metadata.total >= metadata.implemented);
    assert!(!metadata.status_line.is_empty());
}

/// Test that GovernanceMetadata is complete when all scenarios are implemented.
#[test]
fn test_runtime_chain_governance_metadata_complete() {
    let complete = GovernanceMetadata::with_counts("core", 10, 10, "core:10/10");
    assert!(complete.is_complete());

    let incomplete = GovernanceMetadata::with_counts("core", 5, 10, "core:5/10");
    assert!(!incomplete.is_complete());
}
