// Smoke tests for bdd-governance-fixtures (facade crate)
use adze_bdd_governance_fixtures::*;

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
fn re_exports_bdd_progress() {
    let progress = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
    assert!(progress.1 > 0);
}
