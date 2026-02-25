use adze_concurrency_env_core::ConcurrencyCaps;
use adze_concurrency_init_bootstrap_policy_core::bootstrap_caps;
use adze_concurrency_normalize_core::normalized_concurrency;

#[test]
fn bootstrap_caps_matches_normalize_core_contract() {
    let raw = ConcurrencyCaps {
        rayon_threads: 0,
        tokio_worker_threads: 7,
    };

    let normalized = bootstrap_caps(raw);
    let expected = ConcurrencyCaps {
        rayon_threads: normalized_concurrency(raw.rayon_threads),
        tokio_worker_threads: raw.tokio_worker_threads,
    };

    assert_eq!(normalized, expected);
}

#[test]
fn bootstrap_caps_preserves_tokio_worker_threads_exactly() {
    let raw = ConcurrencyCaps {
        rayon_threads: 11,
        tokio_worker_threads: 13,
    };

    let normalized = bootstrap_caps(raw);
    assert_eq!(normalized.tokio_worker_threads, raw.tokio_worker_threads);
}
