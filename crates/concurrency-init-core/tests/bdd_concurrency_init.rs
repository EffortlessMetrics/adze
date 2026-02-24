use adze_concurrency_init_core::{
    init_concurrency_caps, init_rayon_global_once, is_already_initialized_error,
};

#[test]
fn given_global_already_message_when_classifying_then_it_is_detected() {
    // Given
    let message = "The global thread pool has already been initialized";

    // When
    let detected = is_already_initialized_error(message);

    // Then
    assert!(detected);
}

#[test]
fn given_message_missing_required_tokens_when_classifying_then_it_is_not_detected() {
    // Given / When
    let only_global = is_already_initialized_error("global thread pool initialized");
    let only_already = is_already_initialized_error("thread pool already initialized");

    // Then
    assert!(!only_global);
    assert!(!only_already);
}

#[test]
fn given_multiple_init_calls_when_initializing_then_it_remains_idempotent() {
    // Given / When
    init_concurrency_caps();
    init_concurrency_caps();
    init_concurrency_caps();
}

#[test]
fn given_multiple_low_level_init_calls_when_initializing_then_it_remains_idempotent() {
    // Given / When
    let first = init_rayon_global_once(2);
    let second = init_rayon_global_once(8);
    let third = init_rayon_global_once(16);

    // Then
    assert!(first.is_ok());
    assert!(second.is_ok());
    assert!(third.is_ok());
}
