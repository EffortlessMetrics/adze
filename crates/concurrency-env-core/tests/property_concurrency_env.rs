use adze_concurrency_env_core::{
    ConcurrencyCaps, DEFAULT_RAYON_NUM_THREADS, DEFAULT_TOKIO_WORKER_THREADS,
    parse_positive_usize_or_default,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn parse_positive_values_round_trip(
        default in 1usize..1024,
        value in 1usize..1_000_000usize,
    ) {
        let raw = format!(" {value} ");
        prop_assert_eq!(parse_positive_usize_or_default(Some(&raw), default), value);
    }

    #[test]
    fn parse_zero_or_negative_like_inputs_fall_back_to_default(
        default in 1usize..1024,
        value in i64::MIN..=0,
    ) {
        let raw = value.to_string();
        prop_assert_eq!(parse_positive_usize_or_default(Some(&raw), default), default);
    }

    #[test]
    fn parse_unparseable_inputs_fall_back_to_default(
        default in 1usize..1024,
        bytes in prop::collection::vec(any::<u8>(), 1..64),
    ) {
        let raw = String::from_utf8_lossy(&bytes).to_string();
        prop_assume!(raw.trim().parse::<usize>().ok().filter(|value| *value > 0).is_none());
        prop_assert_eq!(parse_positive_usize_or_default(Some(&raw), default), default);
    }

    #[test]
    fn from_lookup_never_returns_zero(
        rayon in prop::option::of(any::<String>()),
        tokio in prop::option::of(any::<String>()),
    ) {
        let caps = ConcurrencyCaps::from_lookup(|name| {
            if name == adze_concurrency_env_core::RAYON_NUM_THREADS_ENV {
                rayon.clone()
            } else if name == adze_concurrency_env_core::TOKIO_WORKER_THREADS_ENV {
                tokio.clone()
            } else {
                None
            }
        });

        prop_assert!(caps.rayon_threads >= 1);
        prop_assert!(caps.tokio_worker_threads >= 1);
    }
}

#[test]
fn from_lookup_matches_defaults_when_missing() {
    let caps = ConcurrencyCaps::from_lookup(|_| None);
    assert_eq!(caps.rayon_threads, DEFAULT_RAYON_NUM_THREADS);
    assert_eq!(caps.tokio_worker_threads, DEFAULT_TOKIO_WORKER_THREADS);
}
