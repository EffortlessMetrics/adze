use adze_concurrency_env_core::ConcurrencyCaps;
use adze_concurrency_init_bootstrap_core::init_concurrency_caps_with_caps;
use adze_concurrency_init_rayon_core::init_rayon_global_once;

#[test]
fn bootstrap_initialization_aligns_with_low_level_rayon_init() {
    let caps = ConcurrencyCaps {
        rayon_threads: 12,
        tokio_worker_threads: 7,
    };

    init_concurrency_caps_with_caps(caps);
    assert!(init_rayon_global_once(caps.rayon_threads).is_ok());
}
