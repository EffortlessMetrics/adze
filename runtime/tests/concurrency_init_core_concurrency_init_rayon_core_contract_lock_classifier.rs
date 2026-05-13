use adze::concurrency_caps::init::rayon::is_already_initialized_error;

#[test]
fn contract_classifier_requires_global_and_already_tokens() {
    assert!(is_already_initialized_error(
        "The global thread pool has already been initialized"
    ));
    assert!(!is_already_initialized_error(
        "global thread pool initialized"
    ));
    assert!(!is_already_initialized_error(
        "thread pool already initialized"
    ));
}
