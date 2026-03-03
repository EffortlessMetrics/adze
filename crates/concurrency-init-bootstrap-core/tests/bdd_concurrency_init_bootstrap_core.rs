use adze_concurrency_env_core::ConcurrencyCaps;
use adze_concurrency_init_bootstrap_core::init_concurrency_caps;
use adze_concurrency_init_bootstrap_core::init_concurrency_caps_with_caps;

#[test]
fn given_zero_rayon_threads_when_bootstrapping_with_explicit_caps_then_initialization_succeeds() {
    // Given / When
    let zero_caps = ConcurrencyCaps {
        rayon_threads: 0,
        tokio_worker_threads: 2,
    };

    // Then
    init_concurrency_caps_with_caps(zero_caps);
}

#[test]
fn given_multiple_explicit_calls_when_bootstrapping_then_initialization_is_idempotent() {
    // Given / When
    let caps = ConcurrencyCaps {
        rayon_threads: 4,
        tokio_worker_threads: 3,
    };

    // Then
    init_concurrency_caps_with_caps(caps);
    init_concurrency_caps_with_caps(caps);
}

#[test]
fn given_default_environment_when_bootstrapping_then_initialization_is_idempotent() {
    init_concurrency_caps();
    init_concurrency_caps();
}
