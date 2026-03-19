// Smoke tests for parser-governance-contract (facade crate)
use adze_parser_governance_contract::*;

#[test]
fn re_exports_bdd_governance_matrix() {
    let m = BddGovernanceMatrix::standard(ParserFeatureProfile::current());
    assert!(!m.status_line().is_empty());
}

#[test]
fn re_exports_glr_conflict_fallback() {
    assert!(!GLR_CONFLICT_FALLBACK.is_empty());
}

#[test]
fn re_exports_parser_backend() {
    let profile = ParserFeatureProfile::current();
    let backend = ParserBackend::select(false);
    assert!(!backend.name().is_empty());

    if profile.has_glr() {
        let conflict_backend = ParserBackend::select(true);
        assert!(!conflict_backend.name().is_empty());
    }
}
