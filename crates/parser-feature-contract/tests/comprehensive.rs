// Smoke tests for parser-feature-contract (facade crate)
use adze_parser_feature_contract::*;

#[test]
fn re_exports_parser_backend() {
    let b = ParserBackend::GLR;
    assert!(b.is_glr());
}

#[test]
fn re_exports_parser_feature_profile() {
    let p = ParserFeatureProfile::current();
    let _ = format!("{:?}", p);
}

#[test]
fn backend_name_non_empty() {
    for b in [
        ParserBackend::TreeSitter,
        ParserBackend::PureRust,
        ParserBackend::GLR,
    ] {
        assert!(!b.name().is_empty());
    }
}
