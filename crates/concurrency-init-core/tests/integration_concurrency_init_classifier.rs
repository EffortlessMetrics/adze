use adze_concurrency_init_classifier_core::is_already_initialized_error as classifier_core_fn;
use adze_concurrency_init_core::is_already_initialized_error as init_core_fn;

#[test]
fn init_core_reexport_matches_classifier_core_behavior() {
    for message in [
        "The global thread pool has already been initialized",
        "global thread pool initialized",
        "thread pool already initialized",
        "totally unrelated",
        "",
    ] {
        assert_eq!(init_core_fn(message), classifier_core_fn(message));
    }
}

#[test]
fn init_core_reexport_is_type_compatible_with_classifier_core() {
    fn accepts_core_fn(f: fn(&str) -> bool) -> fn(&str) -> bool {
        f
    }

    let returned = accepts_core_fn(init_core_fn);
    assert_eq!(
        returned("The global thread pool has already been initialized"),
        classifier_core_fn("The global thread pool has already been initialized")
    );
}
