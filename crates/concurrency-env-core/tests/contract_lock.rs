use adze_concurrency_env_core::{
    ConcurrencyCaps, DEFAULT_RAYON_NUM_THREADS, DEFAULT_TOKIO_WORKER_THREADS,
    RAYON_NUM_THREADS_ENV, TOKIO_WORKER_THREADS_ENV, parse_positive_usize_or_default,
};

#[test]
fn contract_environment_variable_names_are_stable() {
    assert_eq!(RAYON_NUM_THREADS_ENV, "RAYON_NUM_THREADS");
    assert_eq!(TOKIO_WORKER_THREADS_ENV, "TOKIO_WORKER_THREADS");
}

#[test]
fn contract_default_caps_are_stable() {
    let caps = ConcurrencyCaps::default();
    assert_eq!(caps.rayon_threads, DEFAULT_RAYON_NUM_THREADS);
    assert_eq!(caps.tokio_worker_threads, DEFAULT_TOKIO_WORKER_THREADS);
}

#[test]
fn contract_zero_or_invalid_values_use_default() {
    assert_eq!(parse_positive_usize_or_default(Some("0"), 5), 5);
    assert_eq!(parse_positive_usize_or_default(Some("invalid"), 5), 5);
}
