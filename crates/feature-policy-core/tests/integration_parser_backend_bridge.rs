use std::panic::catch_unwind;

use adze_feature_policy_core::ParserFeatureProfile;
use adze_parser_backend_core::ParserBackend as Backend;

#[test]
fn parser_backend_reexport_and_profile_resolve_backend_stay_in_sync() {
    let profile = ParserFeatureProfile::current();

    for has_conflicts in [false, true] {
        let from_profile = catch_unwind(|| profile.resolve_backend(has_conflicts));
        let from_reexport = catch_unwind(|| Backend::select(has_conflicts));

        assert_eq!(
            from_profile.is_ok(),
            from_reexport.is_ok(),
            "panic behavior differs for has_conflicts={has_conflicts}"
        );

        if let (Ok(from_profile), Ok(from_reexport)) = (from_profile, from_reexport) {
            assert_eq!(
                from_profile, from_reexport,
                "backend selection changed for has_conflicts={has_conflicts}"
            );
        }
    }
}
