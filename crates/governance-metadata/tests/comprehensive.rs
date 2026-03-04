// Comprehensive tests for governance metadata types
use adze_governance_metadata::{GovernanceMetadata, ParserFeatureProfileSnapshot};

// ---------------------------------------------------------------------------
// ParserFeatureProfileSnapshot
// ---------------------------------------------------------------------------

#[test]
fn snapshot_new_all_false() {
    let s = ParserFeatureProfileSnapshot::new(false, false, false, false);
    assert!(!s.pure_rust);
    assert!(!s.tree_sitter_standard);
    assert!(!s.tree_sitter_c2rust);
    assert!(!s.glr);
}

#[test]
fn snapshot_new_all_true() {
    let s = ParserFeatureProfileSnapshot::new(true, true, true, true);
    assert!(s.pure_rust);
    assert!(s.tree_sitter_standard);
    assert!(s.tree_sitter_c2rust);
    assert!(s.glr);
}

#[test]
fn snapshot_pure_rust_only() {
    let s = ParserFeatureProfileSnapshot::new(true, false, false, false);
    assert!(s.pure_rust);
    assert!(!s.tree_sitter_standard);
}

#[test]
fn snapshot_glr_only() {
    let s = ParserFeatureProfileSnapshot::new(false, false, false, true);
    assert!(s.glr);
    assert!(!s.pure_rust);
}

#[test]
fn snapshot_debug() {
    let s = ParserFeatureProfileSnapshot::new(true, false, true, false);
    let debug = format!("{:?}", s);
    assert!(debug.contains("ParserFeatureProfileSnapshot"));
}

#[test]
fn snapshot_clone() {
    let s = ParserFeatureProfileSnapshot::new(true, false, true, false);
    let s2 = s;
    assert_eq!(s, s2);
}

#[test]
fn snapshot_eq() {
    let s1 = ParserFeatureProfileSnapshot::new(true, false, true, false);
    let s2 = ParserFeatureProfileSnapshot::new(true, false, true, false);
    assert_eq!(s1, s2);
}

#[test]
fn snapshot_ne() {
    let s1 = ParserFeatureProfileSnapshot::new(true, false, true, false);
    let s2 = ParserFeatureProfileSnapshot::new(false, false, true, false);
    assert_ne!(s1, s2);
}

#[test]
fn snapshot_from_env() {
    let s = ParserFeatureProfileSnapshot::from_env();
    let _ = format!("{:?}", s);
}

#[test]
fn snapshot_serde_roundtrip() {
    let s = ParserFeatureProfileSnapshot::new(true, false, true, true);
    let json = serde_json::to_string(&s).unwrap();
    let s2: ParserFeatureProfileSnapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(s, s2);
}

#[test]
fn snapshot_serde_json_fields() {
    let s = ParserFeatureProfileSnapshot::new(true, false, true, false);
    let json = serde_json::to_string(&s).unwrap();
    assert!(json.contains("pure_rust"));
    assert!(json.contains("tree_sitter_c2rust"));
}

// ---------------------------------------------------------------------------
// Profile conversion
// ---------------------------------------------------------------------------

#[test]
fn snapshot_from_profile() {
    use adze_feature_policy_core::ParserFeatureProfile;
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: true,
        glr: false,
    };
    let s = ParserFeatureProfileSnapshot::from_profile(profile);
    assert!(s.pure_rust);
    assert!(!s.tree_sitter_standard);
    assert!(s.tree_sitter_c2rust);
    assert!(!s.glr);
}

#[test]
fn snapshot_as_profile_roundtrip() {
    use adze_feature_policy_core::ParserFeatureProfile;
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: true,
        tree_sitter_c2rust: false,
        glr: true,
    };
    let s = ParserFeatureProfileSnapshot::from_profile(profile);
    let p2 = s.as_profile();
    assert_eq!(p2.pure_rust, profile.pure_rust);
    assert_eq!(p2.tree_sitter_standard, profile.tree_sitter_standard);
    assert_eq!(p2.tree_sitter_c2rust, profile.tree_sitter_c2rust);
    assert_eq!(p2.glr, profile.glr);
}

// ---------------------------------------------------------------------------
// GovernanceMetadata
// ---------------------------------------------------------------------------

#[test]
fn metadata_complete() {
    let m = GovernanceMetadata::with_counts("test", 10, 10, "all done");
    assert!(m.is_complete());
}

#[test]
fn metadata_incomplete_zero() {
    let m = GovernanceMetadata::with_counts("test", 0, 0, "empty");
    assert!(!m.is_complete());
}

#[test]
fn metadata_incomplete_partial() {
    let m = GovernanceMetadata::with_counts("test", 5, 10, "half");
    assert!(!m.is_complete());
}

#[test]
fn metadata_debug() {
    let m = GovernanceMetadata::with_counts("phase", 3, 5, "in progress");
    let debug = format!("{:?}", m);
    assert!(debug.contains("GovernanceMetadata"));
}
