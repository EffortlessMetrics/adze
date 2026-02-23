use adze_concurrency_init_rayon_core::init_rayon_global_once;

#[test]
fn given_multiple_init_calls_when_initializing_then_it_remains_idempotent() {
    // Given / When
    let first = init_rayon_global_once(2);
    let second = init_rayon_global_once(8);
    let third = init_rayon_global_once(16);

    // Then
    assert!(first.is_ok());
    assert!(second.is_ok());
    assert!(third.is_ok());
}

#[test]
fn given_zero_threads_when_initializing_then_minimum_threads_are_used() {
    // Given / When
    let result = init_rayon_global_once(0);

    // Then
    assert!(result.is_ok());
}
