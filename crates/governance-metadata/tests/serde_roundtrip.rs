//! Serde JSON roundtrip tests for all serializable types in governance-metadata.

use adze_governance_metadata::{GovernanceMetadata, ParserFeatureProfileSnapshot};

// --- ParserFeatureProfileSnapshot ---

#[test]
fn roundtrip_parser_feature_profile_snapshot() {
    let original = ParserFeatureProfileSnapshot::new(true, false, true, true);
    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: ParserFeatureProfileSnapshot =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn roundtrip_parser_feature_profile_snapshot_pretty() {
    let original = ParserFeatureProfileSnapshot::new(true, false, true, true);
    let json = serde_json::to_string_pretty(&original).expect("serialize pretty");
    let deserialized: ParserFeatureProfileSnapshot =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn roundtrip_parser_feature_profile_snapshot_all_false() {
    let original = ParserFeatureProfileSnapshot::new(false, false, false, false);
    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: ParserFeatureProfileSnapshot =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn roundtrip_parser_feature_profile_snapshot_all_true() {
    let original = ParserFeatureProfileSnapshot::new(true, true, true, true);
    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: ParserFeatureProfileSnapshot =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

// --- GovernanceMetadata ---

#[test]
fn roundtrip_governance_metadata() {
    let original = GovernanceMetadata::with_counts("runtime", 5, 10, "runtime:5/10");
    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: GovernanceMetadata = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn roundtrip_governance_metadata_pretty() {
    let original = GovernanceMetadata::with_counts("runtime", 5, 10, "runtime:5/10");
    let json = serde_json::to_string_pretty(&original).expect("serialize pretty");
    let deserialized: GovernanceMetadata = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn roundtrip_governance_metadata_default() {
    let original = GovernanceMetadata::default();
    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: GovernanceMetadata = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn roundtrip_governance_metadata_complete() {
    let original = GovernanceMetadata::with_counts("core", 42, 42, "core:42/42:glr:pure_rust");
    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: GovernanceMetadata = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
    assert!(original.is_complete());
}

#[test]
fn roundtrip_governance_metadata_zero_counts() {
    let original = GovernanceMetadata::with_counts("runtime", 0, 0, "runtime:0/0");
    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: GovernanceMetadata = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn roundtrip_governance_metadata_unicode() {
    let original = GovernanceMetadata::with_counts("运行时", 3, 5, "phase:3/5 — ✓");
    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: GovernanceMetadata = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}
