use adze::concurrency_caps::is_already_initialized_error as runtime_classifier;
use adze_concurrency_init_classifier_core::is_already_initialized_error as core_classifier;

#[test]
fn runtime_reexport_matches_microcrate_behavior() {
    for message in [
        "The global thread pool has already been initialized",
        "global thread pool initialized",
        "thread pool already initialized",
        "totally unrelated",
        "",
    ] {
        assert_eq!(runtime_classifier(message), core_classifier(message));
    }
}

#[test]
fn runtime_reexport_stays_type_compatible() {
    fn accepts_core_fn(f: fn(&str) -> bool) -> fn(&str) -> bool {
        f
    }

    let returned = accepts_core_fn(runtime_classifier);
    assert_eq!(
        returned("The global thread pool has already been initialized"),
        core_classifier("The global thread pool has already been initialized")
    );
}
