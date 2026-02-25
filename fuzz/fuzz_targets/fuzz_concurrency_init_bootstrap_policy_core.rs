#![no_main]

use adze_concurrency_env_core::ConcurrencyCaps;
use adze_concurrency_init_bootstrap_policy_core::bootstrap_caps;
use libfuzzer_sys::fuzz_target;

fn model_bootstrap_caps(caps: ConcurrencyCaps) -> ConcurrencyCaps {
    ConcurrencyCaps {
        rayon_threads: caps.rayon_threads.max(1),
        tokio_worker_threads: caps.tokio_worker_threads,
    }
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 16 {
        return;
    }

    let mut rayon_bytes = [0u8; 8];
    let mut tokio_bytes = [0u8; 8];
    rayon_bytes.copy_from_slice(&data[..8]);
    tokio_bytes.copy_from_slice(&data[8..16]);

    let input = ConcurrencyCaps {
        rayon_threads: u64::from_le_bytes(rayon_bytes) as usize,
        tokio_worker_threads: u64::from_le_bytes(tokio_bytes) as usize,
    };

    let expected = model_bootstrap_caps(input);
    let got = bootstrap_caps(input);

    assert_eq!(got, expected);
});
