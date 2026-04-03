//! BDD-style tests for governance-metadata crate.
//!
//! Tests follow the Given/When/Then pattern to verify public API behavior.

use adze_bdd_grid_core::{BddPhase, BddScenario, BddScenarioStatus};
use adze_feature_policy_core::{ParserBackend, ParserFeatureProfile};
use adze_governance_metadata::{GovernanceMetadata, ParserFeatureProfileSnapshot};

#[test]
fn given_all_flags_true_when_creating_snapshot_then_all_fields_are_true() {
    // Given / When
    let snap = ParserFeatureProfileSnapshot::new(true, true, true, true);

    // Then
    assert!(snap.pure_rust);
    assert!(snap.tree_sitter_standard);
    assert!(snap.tree_sitter_c2rust);
    assert!(snap.glr);
}

#[test]
fn given_all_flags_false_when_creating_snapshot_then_all_fields_are_false() {
    // Given / When
    let snap = ParserFeatureProfileSnapshot::new(false, false, false, false);

    // Then
    assert!(!snap.pure_rust);
    assert!(!snap.tree_sitter_standard);
    assert!(!snap.tree_sitter_c2rust);
    assert!(!snap.glr);
}

#[test]
fn given_snapshot_when_converting_to_profile_then_values_match() {
    // Given
    let snap = ParserFeatureProfileSnapshot::new(true, false, true, false);

    // When
    let profile = snap.as_profile();

    // Then
    assert_eq!(profile.pure_rust, snap.pure_rust);
    assert_eq!(profile.tree_sitter_standard, snap.tree_sitter_standard);
    assert_eq!(profile.tree_sitter_c2rust, snap.tree_sitter_c2rust);
    assert_eq!(profile.glr, snap.glr);
}

#[test]
fn given_profile_when_creating_snapshot_from_profile_then_values_match() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: true,
        glr: false,
    };

    // When
    let snap = ParserFeatureProfileSnapshot::from_profile(profile);

    // Then
    assert_eq!(snap.pure_rust, profile.pure_rust);
    assert_eq!(snap.tree_sitter_standard, profile.tree_sitter_standard);
    assert_eq!(snap.tree_sitter_c2rust, profile.tree_sitter_c2rust);
    assert_eq!(snap.glr, profile.glr);
}

#[test]
fn given_snapshot_when_doing_roundtrip_then_values_are_preserved() {
    // Given
    let original = ParserFeatureProfileSnapshot::new(false, true, false, true);

    // When
    let profile = original.as_profile();
    let roundtrip = ParserFeatureProfileSnapshot::from_profile(profile);

    // Then
    assert_eq!(original, roundtrip);
}

#[test]
fn given_glr_snapshot_when_getting_non_conflict_backend_then_returns_glr() {
    // Given
    let snap = ParserFeatureProfileSnapshot::new(true, false, false, true);

    // When
    let backend = snap.non_conflict_backend();

    // Then
    assert_eq!(backend, ParserBackend::GLR.name());
}

#[test]
fn given_pure_rust_snapshot_when_getting_non_conflict_backend_then_returns_pure_rust() {
    // Given
    let snap = ParserFeatureProfileSnapshot::new(true, false, false, false);

    // When
    let backend = snap.non_conflict_backend();

    // Then
    assert_eq!(backend, ParserBackend::PureRust.name());
}

#[test]
fn given_tree_sitter_snapshot_when_getting_non_conflict_backend_then_returns_tree_sitter() {
    // Given
    let snap = ParserFeatureProfileSnapshot::new(false, true, false, false);

    // When
    let backend = snap.non_conflict_backend();

    // Then
    assert_eq!(backend, ParserBackend::TreeSitter.name());
}

#[test]
fn given_complete_metadata_when_checking_is_complete_then_returns_true() {
    // Given
    let meta = GovernanceMetadata::with_counts("core", 8, 8, "core:8/8");

    // When
    let result = meta.is_complete();

    // Then
    assert!(result);
}

#[test]
fn given_incomplete_metadata_when_checking_is_complete_then_returns_false() {
    // Given
    let meta = GovernanceMetadata::with_counts("core", 5, 8, "core:5/8");

    // When
    let result = meta.is_complete();

    // Then
    assert!(!result);
}

#[test]
fn given_zero_total_metadata_when_checking_is_complete_then_returns_false() {
    // Given
    let meta = GovernanceMetadata::with_counts("core", 0, 0, "core:0/0");

    // When
    let result = meta.is_complete();

    // Then - 0/0 is not complete (total must be > 0)
    assert!(!result);
}

#[test]
fn given_metadata_when_cloning_then_values_match() {
    // Given
    let meta = GovernanceMetadata::with_counts("runtime", 3, 7, "runtime:3/7");

    // When
    let cloned = meta.clone();

    // Then
    assert_eq!(meta, cloned);
}

#[test]
fn given_metadata_with_counts_then_fields_are_set_correctly() {
    // Given / When
    let meta = GovernanceMetadata::with_counts("core", 5, 10, "core:5/10");

    // Then
    assert_eq!(meta.phase, "core");
    assert_eq!(meta.implemented, 5);
    assert_eq!(meta.total, 10);
    assert_eq!(meta.status_line, "core:5/10");
}

#[test]
fn given_default_metadata_then_has_default_values() {
    // Given / When
    let meta = GovernanceMetadata::default();

    // Then
    assert_eq!(meta.phase, "runtime");
    assert_eq!(meta.implemented, 0);
    assert_eq!(meta.total, 0);
    assert!(!meta.is_complete());
}

#[test]
fn given_implemented_scenario_when_creating_metadata_for_grid_then_counts_correctly() {
    // Given
    let scenarios = [BddScenario {
        id: 1,
        title: "test",
        reference: "T-1",
        core_status: BddScenarioStatus::Implemented,
        runtime_status: BddScenarioStatus::Deferred { reason: "wip" },
    }];
    let profile = ParserFeatureProfile::current();

    // When
    let meta = GovernanceMetadata::for_grid(BddPhase::Core, &scenarios, profile);

    // Then
    assert_eq!(meta.phase, "core");
    assert_eq!(meta.implemented, 1);
    assert_eq!(meta.total, 1);
}

#[test]
fn given_deferred_scenario_when_creating_metadata_for_grid_then_counts_correctly() {
    // Given
    let scenarios = [BddScenario {
        id: 1,
        title: "test",
        reference: "T-1",
        core_status: BddScenarioStatus::Deferred { reason: "later" },
        runtime_status: BddScenarioStatus::Implemented,
    }];
    let profile = ParserFeatureProfile::current();

    // When
    let meta = GovernanceMetadata::for_grid(BddPhase::Core, &scenarios, profile);

    // Then
    assert_eq!(meta.phase, "core");
    assert_eq!(meta.implemented, 0);
    assert_eq!(meta.total, 1);
}

#[test]
fn given_empty_scenarios_when_creating_metadata_for_grid_then_counts_are_zero() {
    // Given
    let profile = ParserFeatureProfile::current();

    // When
    let meta = GovernanceMetadata::for_grid(BddPhase::Core, &[], profile);

    // Then
    assert_eq!(meta.implemented, 0);
    assert_eq!(meta.total, 0);
}

#[test]
fn given_runtime_phase_when_creating_metadata_for_grid_then_phase_is_runtime() {
    // Given
    let scenarios = [BddScenario {
        id: 1,
        title: "test",
        reference: "T-1",
        core_status: BddScenarioStatus::Implemented,
        runtime_status: BddScenarioStatus::Implemented,
    }];
    let profile = ParserFeatureProfile::current();

    // When
    let meta = GovernanceMetadata::for_grid(BddPhase::Runtime, &scenarios, profile);

    // Then
    assert_eq!(meta.phase, "runtime");
}

#[test]
fn given_snapshot_when_resolving_non_conflict_backend_then_matches_profile() {
    // Given
    let snap = ParserFeatureProfileSnapshot::new(true, false, false, true);

    // When
    let backend = snap.resolve_non_conflict_backend();
    let expected = snap.as_profile().resolve_backend(false);

    // Then
    assert_eq!(backend, expected);
}

#[test]
fn given_snapshot_when_resolving_conflict_backend_then_matches_profile() {
    // Given
    let snap = ParserFeatureProfileSnapshot::new(true, false, false, true);

    // When
    let backend = snap.resolve_conflict_backend();
    let expected = snap.as_profile().resolve_backend(true);

    // Then
    assert_eq!(backend, expected);
}

#[test]
fn given_snapshot_when_serializing_then_deserializes_correctly() {
    // Given
    let snap = ParserFeatureProfileSnapshot::new(true, false, true, false);

    // When
    let json = serde_json::to_string(&snap).unwrap();
    let deserialized: ParserFeatureProfileSnapshot = serde_json::from_str(&json).unwrap();

    // Then
    assert_eq!(snap, deserialized);
}

#[test]
fn given_metadata_when_serializing_then_deserializes_correctly() {
    // Given
    let meta = GovernanceMetadata::with_counts("core", 3, 5, "core:3/5");

    // When
    let json = serde_json::to_string(&meta).unwrap();
    let deserialized: GovernanceMetadata = serde_json::from_str(&json).unwrap();

    // Then
    assert_eq!(meta, deserialized);
}
