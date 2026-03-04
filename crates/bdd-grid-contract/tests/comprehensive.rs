// Smoke tests for bdd-grid-contract (facade crate)
use adze_bdd_grid_contract::*;

#[test]
fn re_exports_bdd_phase() {
    let _ = BddPhase::Core;
    let _ = BddPhase::Runtime;
}

#[test]
fn re_exports_bdd_scenario() {
    let s = &GLR_CONFLICT_PRESERVATION_GRID[0];
    assert!(!s.title.is_empty());
}

#[test]
fn re_exports_glr_grid() {
    assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}

#[test]
fn re_exports_bdd_progress() {
    let progress = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
    assert!(progress.1 > 0);
}
