use adze_concurrency_normalize_core::{MIN_CONCURRENCY, normalized_concurrency};

#[test]
fn given_zero_requested_concurrency_when_normalizing_then_minimum_is_used() {
    // Given / When
    let normalized = normalized_concurrency(0);

    // Then
    assert_eq!(normalized, MIN_CONCURRENCY);
}

#[test]
fn given_positive_requested_concurrency_when_normalizing_then_value_is_preserved() {
    // Given / When
    let normalized = normalized_concurrency(17);

    // Then
    assert_eq!(normalized, 17);
}
