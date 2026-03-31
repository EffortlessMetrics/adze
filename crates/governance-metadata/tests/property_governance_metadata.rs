//! Property-based tests for governance-metadata.

use proptest::prelude::*;

use adze_feature_policy_core::ParserFeatureProfile;
use adze_governance_metadata::{GovernanceMetadata, ParserFeatureProfileSnapshot};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate arbitrary ParserFeatureProfileSnapshot values.
fn arb_snapshot() -> impl Strategy<Value = ParserFeatureProfileSnapshot> {
    (any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>()).prop_map(
        |(pure_rust, tree_sitter_standard, tree_sitter_c2rust, glr)| {
            ParserFeatureProfileSnapshot::new(
                pure_rust,
                tree_sitter_standard,
                tree_sitter_c2rust,
                glr,
            )
        },
    )
}

/// Generate metadata with valid counts (implemented <= total).
fn arb_valid_metadata() -> impl Strategy<Value = GovernanceMetadata> {
    (0usize..1000usize, 0usize..1000usize, "[a-z]{1,10}").prop_map(|(implemented, extra, phase)| {
        let total = implemented.saturating_add(extra);
        let status_line = format!("{phase}:{implemented}/{total}");
        GovernanceMetadata::with_counts(phase, implemented, total, status_line)
    })
}

// ---------------------------------------------------------------------------
// 1 – ParserFeatureProfileSnapshot Copy and Clone semantics
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn snapshot_copy_preserves_all_fields(s in arb_snapshot()) {
        let s2 = s;
        prop_assert_eq!(s.pure_rust, s2.pure_rust);
        prop_assert_eq!(s.tree_sitter_standard, s2.tree_sitter_standard);
        prop_assert_eq!(s.tree_sitter_c2rust, s2.tree_sitter_c2rust);
        prop_assert_eq!(s.glr, s2.glr);
    }

    #[test]
    fn snapshot_clone_equals_original(s in arb_snapshot()) {
        let cloned = s;
        prop_assert_eq!(s, cloned);
    }
}

// ---------------------------------------------------------------------------
// 2 – ParserFeatureProfileSnapshot PartialEq / Eq
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn snapshot_eq_reflexive(s in arb_snapshot()) {
        prop_assert_eq!(s, s);
    }

    #[test]
    fn snapshot_eq_symmetric(a in arb_snapshot(), b in arb_snapshot()) {
        prop_assert_eq!(a == b, b == a);
    }

    #[test]
    fn snapshot_eq_transitive(a in arb_snapshot(), b in arb_snapshot(), c in arb_snapshot()) {
        if a == b && b == c {
            prop_assert_eq!(a, c);
        }
    }
}

// ---------------------------------------------------------------------------
// 3 – ParserFeatureProfileSnapshot Hash consistency
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn snapshot_hash_consistent(s in arb_snapshot()) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        s.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        s.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        prop_assert_eq!(hash1, hash2);
    }

    #[test]
    fn snapshot_equal_implies_equal_hash(s in arb_snapshot()) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let s2 = s;

        let mut hasher1 = DefaultHasher::new();
        s.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        s2.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        prop_assert_eq!(hash1, hash2);
    }
}

// ---------------------------------------------------------------------------
// 4 – ParserFeatureProfileSnapshot roundtrip via Profile
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn snapshot_roundtrip_via_profile(s in arb_snapshot()) {
        let profile: ParserFeatureProfile = s.as_profile();
        let s2 = ParserFeatureProfileSnapshot::from_profile(profile);
        prop_assert_eq!(s, s2);
    }
}

// ---------------------------------------------------------------------------
// 5 – ParserFeatureProfileSnapshot serde roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn snapshot_serde_roundtrip(s in arb_snapshot()) {
        let json = serde_json::to_string(&s).unwrap();
        let deserialized: ParserFeatureProfileSnapshot = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s, deserialized);
    }
}

// ---------------------------------------------------------------------------
// 6 – ParserFeatureProfileSnapshot backend resolution properties
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn snapshot_backend_non_empty(s in arb_snapshot()) {
        prop_assert!(!s.non_conflict_backend().is_empty());
    }

    #[test]
    fn snapshot_glr_backend_takes_precedence(pure_rust in any::<bool>(), ts_std in any::<bool>(), ts_c2 in any::<bool>()) {
        // When GLR is enabled, it should always be the backend
        let snap = ParserFeatureProfileSnapshot::new(pure_rust, ts_std, ts_c2, true);
        let backend = snap.non_conflict_backend();
        // GLR backend name is "pure-Rust GLR parser"
        prop_assert!(backend.contains("GLR"), "Expected GLR backend but got: {}", backend);
    }
}

// ---------------------------------------------------------------------------
// 7 – GovernanceMetadata properties
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn metadata_is_complete_iff_implemented_equals_total(implemented in 0usize..100usize, extra in 0usize..100usize) {
        let total = implemented.saturating_add(extra);
        let meta = GovernanceMetadata::with_counts("test", implemented, total, "test");
        let expected_complete = total > 0 && implemented == total;
        prop_assert_eq!(meta.is_complete(), expected_complete);
    }

    #[test]
    fn metadata_is_complete_false_when_total_zero(implemented in 0usize..100usize) {
        let meta = GovernanceMetadata::with_counts("test", implemented, 0, "test:0/0");
        prop_assert!(!meta.is_complete());
    }

    #[test]
    fn metadata_phase_preserved(phase in "[a-z]{1,10}") {
        let meta = GovernanceMetadata::with_counts(&phase, 5, 10, "status");
        prop_assert_eq!(meta.phase, phase);
    }

    #[test]
    fn metadata_counts_preserved(implemented in 0usize..1000, total in 0usize..1000) {
        let meta = GovernanceMetadata::with_counts("test", implemented, total, "status");
        prop_assert_eq!(meta.implemented, implemented);
        prop_assert_eq!(meta.total, total);
    }
}

// ---------------------------------------------------------------------------
// 8 – GovernanceMetadata serde roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn metadata_serde_roundtrip(m in arb_valid_metadata()) {
        let json = serde_json::to_string(&m).unwrap();
        let deserialized: GovernanceMetadata = serde_json::from_str(&json).unwrap();
        let m2 = m.clone();
        prop_assert_eq!(m2, deserialized);
    }
}

// ---------------------------------------------------------------------------
// 9 – GovernanceMetadata equality properties
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn metadata_eq_reflexive(m in arb_valid_metadata()) {
        let m2 = m.clone();
        prop_assert_eq!(m, m2);
    }

    #[test]
    fn metadata_eq_symmetric(a in arb_valid_metadata(), b in arb_valid_metadata()) {
        prop_assert_eq!(a == b, b == a);
    }
}

// ---------------------------------------------------------------------------
// 10 – Cross-type conversion properties
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn profile_to_snapshot_to_profile_roundtrip(pure_rust in any::<bool>(), ts_std in any::<bool>(), ts_c2 in any::<bool>(), glr in any::<bool>()) {
        let profile = ParserFeatureProfile {
            pure_rust,
            tree_sitter_standard: ts_std,
            tree_sitter_c2rust: ts_c2,
            glr,
        };
        let snapshot = ParserFeatureProfileSnapshot::from_profile(profile);
        let profile2 = snapshot.as_profile();
        prop_assert_eq!(profile, profile2);
    }
}
