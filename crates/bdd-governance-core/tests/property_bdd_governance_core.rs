//! Property-based tests for bdd-governance-core.

use proptest::prelude::*;

use adze_bdd_governance_core::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, BddScenarioStatus, ParserBackend,
    ParserFeatureProfile,
};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate arbitrary BddPhase values.
fn arb_phase() -> impl Strategy<Value = BddPhase> {
    prop_oneof![Just(BddPhase::Core), Just(BddPhase::Runtime),]
}

/// Generate arbitrary ParserFeatureProfile values.
fn arb_profile() -> impl Strategy<Value = ParserFeatureProfile> {
    (any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>()).prop_map(
        |(pure_rust, tree_sitter_standard, tree_sitter_c2rust, glr)| ParserFeatureProfile {
            pure_rust,
            tree_sitter_standard,
            tree_sitter_c2rust,
            glr,
        },
    )
}

/// Generate arbitrary BddGovernanceSnapshot values.
fn arb_snapshot() -> impl Strategy<Value = BddGovernanceSnapshot> {
    (
        arb_phase(),
        0usize..100usize,
        0usize..100usize,
        arb_profile(),
    )
        .prop_map(|(phase, implemented, total, profile)| {
            let total = total.max(1);
            let implemented = implemented.min(total);
            BddGovernanceSnapshot {
                phase,
                implemented,
                total,
                profile,
            }
        })
}

// ---------------------------------------------------------------------------
// 1 – BddGovernanceSnapshot tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn snapshot_copy_preserves_value(snap in arb_snapshot()) {
        let snap2 = snap;
        prop_assert_eq!(snap.phase, snap2.phase);
        prop_assert_eq!(snap.implemented, snap2.implemented);
        prop_assert_eq!(snap.total, snap2.total);
        prop_assert_eq!(snap.profile, snap2.profile);
    }

    #[test]
    fn snapshot_eq_reflexive(snap in arb_snapshot()) {
        prop_assert_eq!(snap, snap);
    }

    #[test]
    fn snapshot_debug_non_empty(snap in arb_snapshot()) {
        let debug = format!("{:?}", snap);
        prop_assert!(!debug.is_empty());
    }

    #[test]
    fn snapshot_fully_implemented_consistent(snap in arb_snapshot()) {
        let expected = snap.implemented == snap.total;
        prop_assert_eq!(snap.is_fully_implemented(), expected);
    }

    #[test]
    fn snapshot_non_conflict_backend_consistent(snap in arb_snapshot()) {
        let backend = snap.non_conflict_backend();
        // Should be a valid backend
        prop_assert!(!backend.name().is_empty());
    }
}

// ---------------------------------------------------------------------------
// 2 – BddGovernanceMatrix tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn matrix_standard_uses_profile(profile in arb_profile()) {
        let matrix = BddGovernanceMatrix::standard(profile);
        prop_assert_eq!(matrix.profile, profile);
        prop_assert_eq!(matrix.phase, BddPhase::Core);
    }

    #[test]
    fn matrix_new_explicit_params(phase in arb_phase(), profile in arb_profile()) {
        let matrix = BddGovernanceMatrix::new(phase, profile, adze_bdd_governance_core::GLR_CONFLICT_PRESERVATION_GRID);
        prop_assert_eq!(matrix.phase, phase);
        prop_assert_eq!(matrix.profile, profile);
    }

    #[test]
    fn matrix_snapshot_returns_valid_snapshot(profile in arb_profile()) {
        let matrix = BddGovernanceMatrix::standard(profile);
        let snap = matrix.snapshot();
        prop_assert_eq!(snap.profile, profile);
        prop_assert!(snap.total > 0);
    }

    #[test]
    fn matrix_is_fully_implemented_matches_snapshot(profile in arb_profile()) {
        let matrix = BddGovernanceMatrix::standard(profile);
        let snap = matrix.snapshot();
        prop_assert_eq!(matrix.is_fully_implemented(), snap.is_fully_implemented());
    }
}

// ---------------------------------------------------------------------------
// 3 – describe_backend_for_conflicts tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn describe_backend_for_conflicts_non_empty(profile in arb_profile()) {
        let desc = adze_bdd_governance_core::describe_backend_for_conflicts(profile);
        prop_assert!(!desc.is_empty());
    }

    #[test]
    fn describe_backend_glr_when_enabled(profile in arb_profile_without_glr()) {
        // When GLR is enabled, should return GLR backend name
        let mut glr_profile = profile;
        glr_profile.glr = true;
        let desc = adze_bdd_governance_core::describe_backend_for_conflicts(glr_profile);
        prop_assert_eq!(desc, ParserBackend::GLR.name());
    }
}

/// Generate profile without GLR for testing.
fn arb_profile_without_glr() -> impl Strategy<Value = ParserFeatureProfile> {
    (any::<bool>(), any::<bool>(), any::<bool>()).prop_map(
        |(pure_rust, tree_sitter_standard, tree_sitter_c2rust)| ParserFeatureProfile {
            pure_rust,
            tree_sitter_standard,
            tree_sitter_c2rust,
            glr: false,
        },
    )
}
