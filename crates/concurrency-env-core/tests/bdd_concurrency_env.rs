use adze_concurrency_env_core::{
    ConcurrencyCaps, DEFAULT_RAYON_NUM_THREADS, DEFAULT_TOKIO_WORKER_THREADS,
    RAYON_NUM_THREADS_ENV, TOKIO_WORKER_THREADS_ENV,
};

#[test]
fn given_no_environment_values_when_building_caps_then_defaults_are_applied() {
    // Given / When
    let caps = ConcurrencyCaps::from_lookup(|_| None);

    // Then
    assert_eq!(caps.rayon_threads, DEFAULT_RAYON_NUM_THREADS);
    assert_eq!(caps.tokio_worker_threads, DEFAULT_TOKIO_WORKER_THREADS);
}

#[test]
fn given_valid_environment_values_when_building_caps_then_overrides_are_used() {
    // Given / When
    let caps = ConcurrencyCaps::from_lookup(|name| match name {
        RAYON_NUM_THREADS_ENV => Some(String::from("12")),
        TOKIO_WORKER_THREADS_ENV => Some(String::from("7")),
        _ => None,
    });

    // Then
    assert_eq!(caps.rayon_threads, 12);
    assert_eq!(caps.tokio_worker_threads, 7);
}

#[test]
fn given_invalid_environment_values_when_building_caps_then_defaults_are_applied() {
    // Given / When
    let caps = ConcurrencyCaps::from_lookup(|name| match name {
        RAYON_NUM_THREADS_ENV => Some(String::from("0")),
        TOKIO_WORKER_THREADS_ENV => Some(String::from("not-a-number")),
        _ => None,
    });

    // Then
    assert_eq!(caps.rayon_threads, DEFAULT_RAYON_NUM_THREADS);
    assert_eq!(caps.tokio_worker_threads, DEFAULT_TOKIO_WORKER_THREADS);
}
