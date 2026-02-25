use adze_concurrency_env_core::ConcurrencyCaps;
use adze_concurrency_init_bootstrap_policy_core::bootstrap_caps;
use adze_concurrency_normalize_core::normalized_concurrency;
use proptest::prelude::*;

proptest! {
    #[test]
    fn bootstrap_caps_never_returns_zero_rayon_threads(
        rayon_threads in any::<usize>(),
        tokio_threads in any::<usize>(),
    ) {
        let input = ConcurrencyCaps {
            rayon_threads,
            tokio_worker_threads: tokio_threads,
        };
        let output = bootstrap_caps(input);

        prop_assert!(output.rayon_threads >= 1);
        prop_assert_eq!(output.tokio_worker_threads, tokio_threads);
        prop_assert_eq!(output.rayon_threads, normalized_concurrency(input.rayon_threads));
    }

    #[test]
    fn bootstrap_caps_matches_model(
        rayon_threads in 0usize..=4096,
        tokio_threads in 0usize..=1024,
    ) {
        let input = ConcurrencyCaps {
            rayon_threads,
            tokio_worker_threads: tokio_threads,
        };
        let output = bootstrap_caps(input);

        let expected = ConcurrencyCaps {
            rayon_threads: normalized_concurrency(rayon_threads),
            tokio_worker_threads: tokio_threads,
        };

        prop_assert_eq!(output, expected);
    }
}
