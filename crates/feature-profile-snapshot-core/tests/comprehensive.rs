use adze_feature_policy_core::ParserBackend;
use adze_feature_profile_snapshot_core::ParserFeatureProfileSnapshot;

#[test]
fn non_conflict_backend_glr_profile() {
    let snap = ParserFeatureProfileSnapshot::new(true, false, false, true);
    assert_eq!(snap.non_conflict_backend(), ParserBackend::GLR.name());
}

#[test]
fn non_conflict_backend_pure_rust_profile() {
    let snap = ParserFeatureProfileSnapshot::new(true, false, false, false);
    assert_eq!(snap.non_conflict_backend(), ParserBackend::PureRust.name());
}

#[test]
fn non_conflict_backend_tree_sitter_fallback() {
    let snap = ParserFeatureProfileSnapshot::new(false, true, false, false);
    assert_eq!(
        snap.non_conflict_backend(),
        ParserBackend::TreeSitter.name()
    );
}

#[test]
fn resolve_non_conflict_and_conflict_backend() {
    let snap = ParserFeatureProfileSnapshot::new(true, false, false, true);
    let non_conflict = snap.resolve_non_conflict_backend();
    let conflict = snap.resolve_conflict_backend();
    assert_eq!(non_conflict, snap.as_profile().resolve_backend(false));
    assert_eq!(conflict, snap.as_profile().resolve_backend(true));
}

#[test]
fn from_env_with_no_vars_does_not_panic() {
    let snap = ParserFeatureProfileSnapshot::from_env();
    let _ = snap.non_conflict_backend();
}
