use adze_concurrency_caps_contract_core::{
    ConcurrencyCaps, DEFAULT_RAYON_NUM_THREADS, DEFAULT_TOKIO_WORKER_THREADS, bounded_parallel_map,
};

#[test]
fn contract_default_caps_are_stable() {
    let caps = ConcurrencyCaps::default();
    assert_eq!(caps.rayon_threads, DEFAULT_RAYON_NUM_THREADS);
    assert_eq!(caps.tokio_worker_threads, DEFAULT_TOKIO_WORKER_THREADS);
}

#[test]
fn contract_zero_concurrency_matches_single_worker_behavior() {
    let input: Vec<i64> = (0..128).collect();

    let mut zero = bounded_parallel_map(input.clone(), 0, |value| value * 2);
    let mut one = bounded_parallel_map(input, 1, |value| value * 2);

    zero.sort_unstable();
    one.sort_unstable();
    assert_eq!(zero, one);
}
