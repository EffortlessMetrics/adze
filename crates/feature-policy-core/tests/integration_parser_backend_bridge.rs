use std::panic::catch_unwind;

use adze_feature_policy_core::{ParserBackend, ParserFeatureProfile};

#[test]
fn parser_backend_reexport_and_profile_resolve_backend_stay_in_sync() {
    let profile = ParserFeatureProfile::current();

    for has_conflicts in [false, true] {
        let from_profile = catch_unwind(|| profile.resolve_backend(has_conflicts));
        let from_backend = catch_unwind(|| ParserBackend::select(has_conflicts));

        assert_eq!(
            from_profile.is_ok(),
            from_backend.is_ok(),
            "panic behavior differs for has_conflicts={has_conflicts}"
        );

        if let (Ok(from_profile), Ok(from_backend)) = (from_profile, from_backend) {
            assert_eq!(
                from_profile, from_backend,
                "backend selection changed for has_conflicts={has_conflicts}"
            );
        }
    }
}
