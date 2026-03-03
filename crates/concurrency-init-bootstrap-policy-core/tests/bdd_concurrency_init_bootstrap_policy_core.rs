use adze_concurrency_env_core::ConcurrencyCaps;
use adze_concurrency_init_bootstrap_policy_core::bootstrap_caps;

#[test]
fn given_zero_rayon_threads_when_bootstrapping_then_it_normalizes_to_one_worker() {
    // Given / When
    let caps = bootstrap_caps(ConcurrencyCaps {
        rayon_threads: 0,
        tokio_worker_threads: 4,
    });

    // Then
    assert_eq!(caps.rayon_threads, 1);
}

#[test]
fn given_repeated_bootstrap_calls_when_caps_are_normalized_then_result_is_stable() {
    // Given
    let source = ConcurrencyCaps {
        rayon_threads: 0,
        tokio_worker_threads: 3,
    };

    // When
    let once = bootstrap_caps(source);
    let twice = bootstrap_caps(once);

    // Then
    assert_eq!(once, twice);
}
