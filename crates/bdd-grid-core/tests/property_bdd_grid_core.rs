//! Property-based tests for bdd-grid-core.

use proptest::prelude::*;

use adze_bdd_grid_core::{BddPhase, BddScenarioStatus};

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
        Just(BddScenarioStatus::Deferred {
            reason: "test reason"
        }),
    ]
}

// ---------------------------------------------------------------------------
// 1 – BddPhase tests
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

    #[test]
    fn phase_debug_non_empty(phase in arb_phase()) {
        let debug = format!("{:?}", phase);
        prop_assert!(!debug.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 2 – BddScenarioStatus tests
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
    fn status_display_non_empty(status in arb_status()) {
        let display = format!("{}", status);
        prop_assert!(!display.is_empty());
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

    #[test]
    fn status_implemented_consistent(status in arb_status()) {
        let implemented = status.implemented();
        let label = status.label();
        if implemented {
            prop_assert_eq!(label, "IMPLEMENTED");
        } else {
            prop_assert_eq!(label, "DEFERRED");
        }
    }

    #[test]
    fn status_detail_for_implemented_is_empty(status in arb_status()) {
        if status.implemented() {
            prop_assert_eq!(status.detail(), "");
        }
    }
}

// ---------------------------------------------------------------------------
// 3 – Grid constant tests
// ---------------------------------------------------------------------------

#[test]
fn grid_scenarios_have_valid_ids() {
    for (i, scenario) in adze_bdd_grid_core::GLR_CONFLICT_PRESERVATION_GRID
        .iter()
        .enumerate()
    {
        assert!(scenario.id > 0, "Scenario {} should have positive ID", i);
        assert!(
            !scenario.title.is_empty(),
            "Scenario {} should have title",
            i
        );
        assert!(
            !scenario.reference.is_empty(),
            "Scenario {} should have reference",
            i
        );
    }
}

#[test]
fn grid_scenarios_have_unique_ids() {
    use std::collections::HashSet;
    let ids: HashSet<u8> = adze_bdd_grid_core::GLR_CONFLICT_PRESERVATION_GRID
        .iter()
        .map(|s| s.id)
        .collect();
    assert_eq!(
        ids.len(),
        adze_bdd_grid_core::GLR_CONFLICT_PRESERVATION_GRID.len()
    );
}

#[test]
fn bdd_progress_empty_returns_zero() {
    let (implemented, total) = adze_bdd_grid_core::bdd_progress(BddPhase::Core, &[]);
    assert_eq!(implemented, 0);
    assert_eq!(total, 0);
}

#[test]
fn bdd_progress_grid_returns_valid_counts() {
    let (core_impl, core_total) = adze_bdd_grid_core::bdd_progress(
        BddPhase::Core,
        adze_bdd_grid_core::GLR_CONFLICT_PRESERVATION_GRID,
    );
    let (rt_impl, rt_total) = adze_bdd_grid_core::bdd_progress(
        BddPhase::Runtime,
        adze_bdd_grid_core::GLR_CONFLICT_PRESERVATION_GRID,
    );

    assert!(core_total > 0);
    assert_eq!(core_total, rt_total);
    assert!(core_impl <= core_total);
    assert!(rt_impl <= rt_total);
}
