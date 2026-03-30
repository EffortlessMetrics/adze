//! Property-based tests for bdd-governance-contract.

use proptest::prelude::*;

use adze_bdd_governance_contract::{
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
            let total = total.max(1); // Ensure total >= 1
            let implemented = implemented.min(total); // implemented <= total
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
    fn snapshot_fully_implemented_when_equal(snap in arb_snapshot()) {
        let expected = snap.implemented == snap.total;
        prop_assert_eq!(snap.is_fully_implemented(), expected);
    }

    #[test]
    fn snapshot_implemented_lte_total(snap in arb_snapshot()) {
        prop_assert!(snap.implemented <= snap.total);
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
        prop_assert!(!matrix.scenarios.is_empty());
    }

    #[test]
    fn matrix_debug_non_empty(profile in arb_profile()) {
        let matrix = BddGovernanceMatrix::standard(profile);
        let debug = format!("{:?}", matrix);
        prop_assert!(!debug.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 3 – ParserBackend tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn backend_name_non_empty() {
        prop_assert!(!ParserBackend::TreeSitter.name().is_empty());
        prop_assert!(!ParserBackend::PureRust.name().is_empty());
        prop_assert!(!ParserBackend::GLR.name().is_empty());
    }
}
