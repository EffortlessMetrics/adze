// Smoke tests for governance-matrix-core (facade crate)
use adze_governance_matrix_core::*;

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
fn re_exports_bdd_governance_matrix() {
    let m = BddGovernanceMatrix::standard(ParserFeatureProfile::current());
    assert!(!m.status_line().is_empty());
}

#[test]
fn re_exports_parser_backend() {
    let b = ParserBackend::select(false);
    assert!(!b.name().is_empty());
}
