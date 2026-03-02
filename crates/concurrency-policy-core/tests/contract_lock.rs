use adze_concurrency_policy_core::{
    DEFAULT_RAYON_NUM_THREADS, DEFAULT_TOKIO_WORKER_THREADS, RAYON_NUM_THREADS_ENV,
    TOKIO_WORKER_THREADS_ENV, resolve_caps_from_lookup,
};

#[test]
fn constants_remain_stable() {
    assert_eq!(RAYON_NUM_THREADS_ENV, "RAYON_NUM_THREADS");
    assert_eq!(TOKIO_WORKER_THREADS_ENV, "TOKIO_WORKER_THREADS");
    assert_eq!(DEFAULT_RAYON_NUM_THREADS, 4);
    assert_eq!(DEFAULT_TOKIO_WORKER_THREADS, 2);
}

#[test]
fn resolve_caps_from_lookup_parses_and_falls_back() {
    let (rayon, tokio) = resolve_caps_from_lookup(|name| match name {
        RAYON_NUM_THREADS_ENV => Some("16".to_string()),
        TOKIO_WORKER_THREADS_ENV => Some("0".to_string()),
        _ => None,
    });

    assert_eq!(rayon, 16);
    assert_eq!(tokio, DEFAULT_TOKIO_WORKER_THREADS);
}
