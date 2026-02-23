use adze_concurrency_init_core::{
    init_concurrency_caps, init_rayon_global_once, is_already_initialized_error,
};

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

#[test]
fn contract_init_remains_idempotent() {
    init_concurrency_caps();
    init_concurrency_caps();
}

#[test]
fn contract_low_level_init_remains_idempotent() {
    assert!(init_rayon_global_once(1).is_ok());
    assert!(init_rayon_global_once(8).is_ok());
}
