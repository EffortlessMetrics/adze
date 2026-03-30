//! Property-based tests for runtime-governance-api.

use proptest::prelude::*;

use adze_runtime_governance_api::{BddGovernanceSnapshot, BddPhase, ParserFeatureProfile};

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
    fn snapshot_fully_implemented_consistent(snap in arb_snapshot()) {
        let expected = snap.implemented == snap.total;
        prop_assert_eq!(snap.is_fully_implemented(), expected);
    }
}

// ---------------------------------------------------------------------------
// 2 – ParserFeatureProfile tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn profile_copy_preserves_all_fields(p in arb_profile()) {
        let p2 = p;
        prop_assert_eq!(p, p2);
    }

    #[test]
    fn profile_eq_reflexive(p in arb_profile()) {
        prop_assert_eq!(p, p);
    }
}

// ---------------------------------------------------------------------------
// 3 – Re-exported function tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn bdd_progress_report_for_current_profile_non_empty(phase in arb_phase()) {
        let report = adze_runtime_governance_api::bdd_progress_report_for_current_profile(phase, "Test");
        prop_assert!(!report.is_empty());
    }

    #[test]
    fn bdd_status_line_for_current_profile_non_empty(phase in arb_phase()) {
        let line = adze_runtime_governance_api::bdd_status_line_for_current_profile(phase);
        prop_assert!(!line.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 4 – Grid constant tests
// ---------------------------------------------------------------------------

#[test]
fn grid_constant_has_scenarios() {
    assert!(!adze_runtime_governance_api::GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}
