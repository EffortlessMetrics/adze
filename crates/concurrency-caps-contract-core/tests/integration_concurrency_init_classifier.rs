use adze_concurrency_caps_contract_core::is_already_initialized_error as caps_classifier;
use adze_concurrency_init_classifier_core::is_already_initialized_error as classifier_core;

#[test]
fn caps_core_reexport_matches_classifier_core_behavior() {
    for message in [
        "The global thread pool has already been initialized",
        "global thread pool initialized",
        "thread pool already initialized",
        "totally unrelated",
        "",
    ] {
        assert_eq!(caps_classifier(message), classifier_core(message));
    }
}

#[test]
fn caps_core_reexport_is_type_compatible_with_classifier_core() {
    fn accepts_core_fn(f: fn(&str) -> bool) -> fn(&str) -> bool {
        f
    }

    let returned = accepts_core_fn(caps_classifier);
    assert_eq!(
        returned("The global thread pool has already been initialized"),
        classifier_core("The global thread pool has already been initialized")
    );
}
