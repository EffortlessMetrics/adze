// Smoke tests for governance-matrix-contract (facade crate)
use adze_governance_matrix_contract::*;

#[test]
fn re_exports_bdd_phase() {
    let _ = BddPhase::Core;
}

#[test]
fn re_exports_parser_feature_profile() {
    let p = ParserFeatureProfile::current();
    let _ = format!("{:?}", p);
}

#[test]
fn re_exports_bdd_governance_matrix() {
    let m = BddGovernanceMatrix::standard(ParserFeatureProfile::current());
    assert_eq!(m.phase, BddPhase::Core);
}
