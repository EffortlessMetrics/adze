//! Property-based tests for runtime-governance-matrix.

use proptest::prelude::*;

use adze_runtime_governance_matrix::{BddGovernanceSnapshot, BddPhase, ParserFeatureProfile};

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

    #[test]
    fn profile_hash_consistent(p in arb_profile()) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        p.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        p.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        prop_assert_eq!(hash1, hash2);
    }
}

// ---------------------------------------------------------------------------
// 3 – Matrix function tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn bdd_governance_matrix_for_profile_returns_valid(phase in arb_phase(), profile in arb_profile()) {
        let matrix = adze_runtime_governance_matrix::bdd_governance_matrix_for_profile(phase, profile);
        prop_assert_eq!(matrix.phase, phase);
        prop_assert_eq!(matrix.profile, profile);
    }

    #[test]
    fn bdd_governance_matrix_for_current_profile_returns_valid(phase in arb_phase()) {
        let matrix = adze_runtime_governance_matrix::bdd_governance_matrix_for_current_profile(phase);
        prop_assert_eq!(matrix.phase, phase);
    }

    #[test]
    fn bdd_progress_report_for_current_profile_non_empty(phase in arb_phase()) {
        let report = adze_runtime_governance_matrix::bdd_progress_report_for_current_profile(phase, "Test");
        prop_assert!(!report.is_empty());
    }

    #[test]
    fn bdd_status_line_for_current_profile_non_empty(phase in arb_phase()) {
        let line = adze_runtime_governance_matrix::bdd_status_line_for_current_profile(phase);
        prop_assert!(!line.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 4 – Backend resolution tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn resolve_backend_for_profile_consistent(profile in arb_profile()) {
        let backend = adze_runtime_governance_matrix::resolve_backend_for_profile(profile, false);
        let expected = profile.resolve_backend(false);
        prop_assert_eq!(backend, expected);
    }
}

#[test]
fn current_backend_for_returns_valid_backend() {
    let backend = adze_runtime_governance_matrix::current_backend_for(false);
    assert!(!backend.name().is_empty());
}

// ---------------------------------------------------------------------------
// 5 – Grid constant tests
// ---------------------------------------------------------------------------

#[test]
fn grid_constant_has_scenarios() {
    assert!(!adze_runtime_governance_matrix::GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}
