use adze_concurrency_normalize_core::{MIN_CONCURRENCY, normalized_concurrency};
use proptest::prelude::*;

proptest! {
    #[test]
    fn normalized_concurrency_never_returns_zero(value in any::<usize>()) {
        prop_assert!(normalized_concurrency(value) >= MIN_CONCURRENCY);
    }

    #[test]
    fn normalized_concurrency_matches_model(value in any::<usize>()) {
        let expected = value.max(MIN_CONCURRENCY);
        prop_assert_eq!(normalized_concurrency(value), expected);
    }
}
