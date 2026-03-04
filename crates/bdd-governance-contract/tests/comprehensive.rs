// Smoke tests for bdd-governance-contract (facade crate)
use adze_bdd_governance_contract::*;

#[test]
fn re_exports_bdd_governance_snapshot() {
    let snap = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 3,
        total: 6,
        profile: ParserFeatureProfile::current(),
    };
    assert!(!snap.is_fully_implemented());
}

#[test]
fn re_exports_bdd_governance_matrix() {
    let m = BddGovernanceMatrix::standard(ParserFeatureProfile::current());
    assert!(!m.status_line().is_empty());
}

#[test]
fn re_exports_glr_conflict_fallback() {
    assert!(!GLR_CONFLICT_FALLBACK.is_empty());
}
