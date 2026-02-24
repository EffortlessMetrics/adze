use adze::concurrency_caps::ConcurrencyCaps as RuntimeConcurrencyCaps;
use adze::concurrency_caps::{bounded_parallel_map, init_concurrency_caps, normalized_concurrency};
use adze_concurrency_caps_core::ConcurrencyCaps as CoreConcurrencyCaps;

#[test]
fn runtime_reexports_bounded_parallel_map_with_expected_behavior() {
    let input: Vec<i32> = (0..256).collect();
    let mut output = bounded_parallel_map(input.clone(), 4, |value| value * 2 + 1);
    output.sort_unstable();

    let expected: Vec<i32> = input.into_iter().map(|value| value * 2 + 1).collect();
    assert_eq!(output, expected);
}

#[test]
fn runtime_reexport_normalizes_zero_concurrency() {
    assert_eq!(normalized_concurrency(0), 1);
    assert_eq!(normalized_concurrency(5), 5);
}

#[test]
fn runtime_reexport_init_is_idempotent() {
    init_concurrency_caps();
    init_concurrency_caps();
}

#[test]
fn runtime_reexport_stays_type_compatible() {
    fn accepts_core_type(value: CoreConcurrencyCaps) -> CoreConcurrencyCaps {
        value
    }

    let runtime_value = RuntimeConcurrencyCaps::default();
    let returned = accepts_core_type(runtime_value);

    assert_eq!(
        returned.rayon_threads,
        RuntimeConcurrencyCaps::default().rayon_threads
    );
    assert_eq!(
        returned.tokio_worker_threads,
        RuntimeConcurrencyCaps::default().tokio_worker_threads
    );
}
