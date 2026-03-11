use adze_feature_policy_core::{ParserBackend, ParserFeatureProfile};
use adze_feature_profile_metadata::ParserFeatureProfileSnapshot;

#[test]
fn snapshot_from_and_to_profile_roundtrip() {
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: true,
        glr: false,
    };

    let snapshot = ParserFeatureProfileSnapshot::from_profile(profile);
    assert_eq!(snapshot.as_profile(), profile);
}

#[test]
fn snapshot_resolve_backend_helpers_match_profile() {
    let snapshot = ParserFeatureProfileSnapshot::new(true, false, false, true);

    assert_eq!(snapshot.non_conflict_backend(), ParserBackend::GLR.name());
    assert_eq!(snapshot.resolve_non_conflict_backend(), ParserBackend::GLR);
    assert_eq!(snapshot.resolve_conflict_backend(), ParserBackend::GLR);
}

#[test]
fn snapshot_is_serde_roundtrippable() {
    let original = ParserFeatureProfileSnapshot::new(true, true, false, false);
    let json = serde_json::to_string(&original).expect("serialize");
    let decoded: ParserFeatureProfileSnapshot = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(decoded, original);
}
