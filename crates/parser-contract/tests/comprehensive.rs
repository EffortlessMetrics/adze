// Smoke tests for parser-contract (facade crate)
use adze_parser_contract::*;

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
