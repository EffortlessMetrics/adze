use adze_concurrency_env_contract_core::{
    ConcurrencyCaps, RAYON_NUM_THREADS_ENV, TOKIO_WORKER_THREADS_ENV,
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
            if name == RAYON_NUM_THREADS_ENV {
                rayon.clone()
            } else if name == TOKIO_WORKER_THREADS_ENV {
                tokio.clone()
            } else {
                None
            }
        });

        prop_assert!(caps.rayon_threads >= 1);
        prop_assert!(caps.tokio_worker_threads >= 1);
    }
}
