use adze_concurrency_normalize_core::{MIN_CONCURRENCY, normalized_concurrency};

#[test]
fn contract_minimum_concurrency_constant_is_stable() {
    assert_eq!(MIN_CONCURRENCY, 1);
}

#[test]
fn contract_zero_requested_concurrency_normalizes_to_minimum() {
    assert_eq!(normalized_concurrency(0), MIN_CONCURRENCY);
}
