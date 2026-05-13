use adze::concurrency_caps::is_already_initialized_error as runtime_classifier;
use adze_concurrency_init_core::rayon::is_already_initialized_error as rayon_classifier;

#[test]
fn runtime_reexport_matches_microcrate_behavior() {
    for message in [
        "The global thread pool has already been initialized",
        "global thread pool initialized",
        "thread pool already initialized",
        "totally unrelated",
        "",
    ] {
        assert_eq!(runtime_classifier(message), rayon_classifier(message));
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
        rayon_classifier("The global thread pool has already been initialized")
    );
}
