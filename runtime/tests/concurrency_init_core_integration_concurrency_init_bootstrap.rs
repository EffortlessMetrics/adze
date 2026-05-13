use adze::concurrency_caps::init::init_concurrency_caps_with_caps;
use adze::concurrency_caps::init::rayon::init_rayon_global_once;
use adze_concurrency_env_contract_core::ConcurrencyCaps;

#[test]
fn bootstrap_initialization_aligns_with_low_level_rayon_init() {
    let caps = ConcurrencyCaps {
        rayon_threads: 12,
        tokio_worker_threads: 7,
    };

    init_concurrency_caps_with_caps(caps);
    assert!(init_rayon_global_once(caps.rayon_threads).is_ok());
}
