// Smoke tests for feature-policy-contract (facade crate)
use adze_feature_policy_contract::*;

#[test]
fn re_exports_parser_backend() {
    let b = ParserBackend::select(false);
    assert!(!b.name().is_empty());
}

#[test]
fn re_exports_parser_feature_profile() {
    let p = ParserFeatureProfile::current();
    let _ = format!("{:?}", p);
}

#[test]
fn backend_variants_accessible() {
    let _ = ParserBackend::TreeSitter;
    let _ = ParserBackend::PureRust;
    let _ = ParserBackend::GLR;
}
