// Smoke tests for governance-contract (facade crate)
use adze_governance_contract::*;

#[test]
fn re_exports_bdd_governance_matrix() {
    let m = BddGovernanceMatrix::standard(ParserFeatureProfile::current());
    assert!(!m.report("Core").is_empty());
}

#[test]
fn re_exports_bdd_progress() {
    let progress = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
    assert!(progress.1 > 0);
}
