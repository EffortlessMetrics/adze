//! Contract lock test - verifies that public API remains stable.

use adze_feature_policy_core::ParserFeatureProfile;
use adze_governance_metadata::{GovernanceMetadata, ParserFeatureProfileSnapshot};

type ParserFeatureProfileSnapshotNewFn = fn(bool, bool, bool, bool) -> ParserFeatureProfileSnapshot;
type GovernanceMetadataWithCountsFn = fn(String, usize, usize, String) -> GovernanceMetadata;

/// Verify all public types exist and have expected structure.
#[test]
fn test_contract_lock_types() {
    // Verify ParserFeatureProfileSnapshot struct exists with expected fields
    let snap = ParserFeatureProfileSnapshot {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: true,
        glr: false,
    };
    assert!(snap.pure_rust);
    assert!(!snap.tree_sitter_standard);
    assert!(snap.tree_sitter_c2rust);
    assert!(!snap.glr);

    // Verify GovernanceMetadata struct exists with expected fields
    let meta = GovernanceMetadata {
        phase: "core".to_string(),
        implemented: 5,
        total: 10,
        status_line: "core:5/10".to_string(),
    };
    assert_eq!(meta.phase, "core");
    assert_eq!(meta.implemented, 5);
    assert_eq!(meta.total, 10);
    assert_eq!(meta.status_line, "core:5/10");
}

/// Verify all public functions exist with expected signatures.
#[test]
fn test_contract_lock_functions() {
    // Verify ParserFeatureProfileSnapshot::new exists
    let _fn_ptr: Option<ParserFeatureProfileSnapshotNewFn> =
        Some(ParserFeatureProfileSnapshot::new);

    // Verify ParserFeatureProfileSnapshot::from_profile exists
    let _fn_ptr: Option<fn(ParserFeatureProfile) -> ParserFeatureProfileSnapshot> =
        Some(ParserFeatureProfileSnapshot::from_profile);

    // Verify ParserFeatureProfileSnapshot::as_profile exists
    let snap = ParserFeatureProfileSnapshot::new(true, false, true, false);
    let _profile = snap.as_profile();

    // Verify ParserFeatureProfileSnapshot::non_conflict_backend exists
    let snap = ParserFeatureProfileSnapshot::new(true, false, false, true);
    let _backend = snap.non_conflict_backend();

    // Verify ParserFeatureProfileSnapshot::resolve_non_conflict_backend exists
    let snap = ParserFeatureProfileSnapshot::new(true, false, false, true);
    let _backend = snap.resolve_non_conflict_backend();

    // Verify ParserFeatureProfileSnapshot::resolve_conflict_backend exists
    let snap = ParserFeatureProfileSnapshot::new(true, false, false, true);
    let _backend = snap.resolve_conflict_backend();

    // Verify GovernanceMetadata::with_counts exists
    let _fn_ptr: Option<GovernanceMetadataWithCountsFn> = Some(GovernanceMetadata::with_counts);

    // Verify GovernanceMetadata::is_complete exists
    let meta = GovernanceMetadata::with_counts("core", 5, 10, "core:5/10");
    let _result = meta.is_complete();
}

/// Verify ParserFeatureProfileSnapshot methods.
#[test]
fn test_contract_lock_snapshot_methods() {
    let snap = ParserFeatureProfileSnapshot::new(true, false, true, false);

    // Verify as_profile method
    let profile = snap.as_profile();
    assert_eq!(profile.pure_rust, snap.pure_rust);

    // Verify non_conflict_backend method
    let backend = snap.non_conflict_backend();
    assert!(!backend.is_empty());

    // Verify resolve_non_conflict_backend method
    let _backend = snap.resolve_non_conflict_backend();

    // Verify from_env method exists
    let _env_snap = ParserFeatureProfileSnapshot::from_env();
}

/// Verify GovernanceMetadata methods.
#[test]
fn test_contract_lock_metadata_methods() {
    let meta = GovernanceMetadata::with_counts("core", 5, 10, "core:5/10");

    // Verify is_complete method
    let _result = meta.is_complete();

    // Verify Default trait
    let _default = GovernanceMetadata::default();

    // Verify Clone trait
    let _cloned = meta.clone();

    // Verify Debug trait is implemented (just ensure it compiles)
    let _debug = format!("{:?}", meta);
}
