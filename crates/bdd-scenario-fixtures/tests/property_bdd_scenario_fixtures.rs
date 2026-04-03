//! Property-based tests for bdd-scenario-fixtures.

use proptest::prelude::*;

use adze_bdd_scenario_fixtures::{BddPhase, ParserFeatureProfile};

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

// ---------------------------------------------------------------------------
// 1 – Re-exported type tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn phase_copy_preserves_value(phase in arb_phase()) {
        let phase2 = phase;
        prop_assert_eq!(phase, phase2);
    }

    #[test]
    fn phase_display_non_empty(phase in arb_phase()) {
        let display = format!("{}", phase);
        prop_assert!(!display.is_empty());
    }

    #[test]
    fn profile_copy_preserves_value(profile in arb_profile()) {
        let profile2 = profile;
        prop_assert_eq!(profile, profile2);
    }

    #[test]
    fn profile_debug_non_empty(profile in arb_profile()) {
        let debug = format!("{:?}", profile);
        prop_assert!(!debug.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 2 – Grid constant tests
// ---------------------------------------------------------------------------

#[test]
fn grid_constant_has_scenarios() {
    assert!(!adze_bdd_scenario_fixtures::GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}

proptest! {
    #[test]
    fn bdd_progress_returns_valid_counts(phase in arb_phase()) {
        let (implemented, total) = adze_bdd_scenario_fixtures::bdd_progress(
            phase,
            adze_bdd_scenario_fixtures::GLR_CONFLICT_PRESERVATION_GRID,
        );
        prop_assert!(total > 0);
        prop_assert!(implemented <= total);
    }
}
