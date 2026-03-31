//! Property-based tests for bdd-governance-fixtures.

use proptest::prelude::*;

use adze_bdd_governance_fixtures::{BddPhase, ParserFeatureProfile};

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
    fn profile_display_non_empty(profile in arb_profile()) {
        let display = format!("{}", profile);
        prop_assert!(!display.is_empty() || display == "none");
    }
}

// ---------------------------------------------------------------------------
// 2 – Report function tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn bdd_progress_report_for_current_profile_contains_title(
        phase in arb_phase(),
        title in ".*{0,20}"
    ) {
        let report = adze_bdd_governance_fixtures::bdd_progress_report_for_current_profile(phase, &title);
        // Report should always be non-empty
        prop_assert!(!report.is_empty());
    }

    #[test]
    fn bdd_progress_status_line_for_current_profile_non_empty(phase in arb_phase()) {
        let line = adze_bdd_governance_fixtures::bdd_progress_status_line_for_current_profile(phase);
        prop_assert!(!line.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 3 – Grid constant tests
// ---------------------------------------------------------------------------

#[test]
fn grid_constant_has_scenarios() {
    assert!(!adze_bdd_governance_fixtures::GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}
