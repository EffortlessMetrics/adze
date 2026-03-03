#![no_main]

use adze_concurrency_env_core::ConcurrencyCaps;
use adze_concurrency_init_bootstrap_core::init_concurrency_caps_with_caps;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let rayon_threads = usize::from(data[0]);
    let mut tokio_worker_threads = 0usize;

    if let Some(byte) = data.get(1) {
        tokio_worker_threads = usize::from(*byte);
    }

    init_concurrency_caps_with_caps(ConcurrencyCaps {
        rayon_threads,
        tokio_worker_threads,
    });
});
