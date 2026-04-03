//! Property-based tests for bdd-grid-contract (re-exports from bdd-grid-core).

use proptest::prelude::*;

use adze_bdd_grid_contract::{BddPhase, BddScenarioStatus};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate arbitrary BddPhase values.
fn arb_phase() -> impl Strategy<Value = BddPhase> {
    prop_oneof![Just(BddPhase::Core), Just(BddPhase::Runtime),]
}

/// Generate arbitrary BddScenarioStatus values.
fn arb_status() -> impl Strategy<Value = BddScenarioStatus> {
    prop_oneof![
        Just(BddScenarioStatus::Implemented),
        Just(BddScenarioStatus::Deferred { reason: "test" }),
    ]
}

// ---------------------------------------------------------------------------
// 1 – Re-exported BddPhase tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn phase_copy_preserves_value(phase in arb_phase()) {
        let phase2 = phase;
        prop_assert_eq!(phase, phase2);
    }

    #[test]
    fn phase_eq_reflexive(phase in arb_phase()) {
        prop_assert_eq!(phase, phase);
    }

    #[test]
    fn phase_display_non_empty(phase in arb_phase()) {
        let display = format!("{}", phase);
        prop_assert!(!display.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 2 – Re-exported BddScenarioStatus tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn status_copy_preserves_value(status in arb_status()) {
        let status2 = status;
        prop_assert_eq!(status, status2);
    }

    #[test]
    fn status_eq_reflexive(status in arb_status()) {
        prop_assert_eq!(status, status);
    }

    #[test]
    fn status_icon_non_empty(status in arb_status()) {
        let icon = status.icon();
        prop_assert!(!icon.is_empty());
    }

    #[test]
    fn status_label_non_empty(status in arb_status()) {
        let label = status.label();
        prop_assert!(!label.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 3 – Grid constant tests
// ---------------------------------------------------------------------------

#[test]
fn grid_constant_has_scenarios() {
    assert!(!adze_bdd_grid_contract::GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}
