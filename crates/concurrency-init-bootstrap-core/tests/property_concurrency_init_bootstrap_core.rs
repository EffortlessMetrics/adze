use adze_concurrency_env_core::ConcurrencyCaps;
use adze_concurrency_init_bootstrap_core::init_concurrency_caps_with_caps;
use proptest::prelude::*;

proptest! {
    #[test]
    fn repeated_bootstrap_calls_never_panic(
        rayon_threads in 0usize..4096,
        tokio_threads in 0usize..2048,
        call_count in 0usize..128,
    ) {
        let caps = ConcurrencyCaps {
            rayon_threads,
            tokio_worker_threads: tokio_threads,
        };

        for _ in 0..=call_count {
            init_concurrency_caps_with_caps(caps);
        }
    }
}
