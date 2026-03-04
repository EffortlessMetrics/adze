// Smoke tests for bdd-contract (facade crate)
use adze_bdd_contract::*;

#[test]
fn re_exports_bdd_phase() {
    let _ = BddPhase::Core;
    let _ = BddPhase::Runtime;
}

#[test]
fn re_exports_bdd_scenario_status() {
    let _ = BddScenarioStatus::Implemented;
    let _ = BddScenarioStatus::Deferred { reason: "test" };
}

#[test]
fn re_exports_bdd_scenario() {
    let s = BddScenario {
        id: 99,
        title: "test",
        reference: "ref",
        core_status: BddScenarioStatus::Implemented,
        runtime_status: BddScenarioStatus::Deferred { reason: "wip" },
    };
    assert_eq!(s.title, "test");
}

#[test]
fn re_exports_glr_grid() {
    assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}
