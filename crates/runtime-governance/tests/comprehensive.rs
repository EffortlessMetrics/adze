// Smoke tests for runtime-governance (facade crate)
use adze_runtime_governance::*;

#[test]
fn re_exports_parser_feature_profile() {
    let p = parser_feature_profile_for_runtime();
    let _ = format!("{:?}", p);
}

#[test]
fn re_exports_resolve_backend() {
    let p = parser_feature_profile_for_runtime();
    let b = resolve_backend_for_profile(p, false);
    assert!(!b.name().is_empty());
}

#[test]
fn re_exports_governance_matrix_for_runtime() {
    let m = bdd_governance_matrix_for_runtime();
    let _ = format!("{:?}", m);
}

#[test]
fn re_exports_glr_conflict_fallback() {
    assert!(!GLR_CONFLICT_FALLBACK.is_empty());
}
