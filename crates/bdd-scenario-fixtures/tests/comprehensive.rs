// Smoke tests for bdd-scenario-fixtures (facade crate)
use adze_bdd_scenario_fixtures::*;

#[test]
fn re_exports_bdd_phase() {
    let _ = BddPhase::Core;
    let _ = BddPhase::Runtime;
}

#[test]
fn re_exports_parser_feature_profile() {
    let p = ParserFeatureProfile::current();
    let _ = format!("{:?}", p);
}

#[test]
fn re_exports_glr_grid() {
    assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}
