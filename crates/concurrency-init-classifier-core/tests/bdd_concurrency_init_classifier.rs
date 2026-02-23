use adze_concurrency_init_classifier_core::is_already_initialized_error;

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
