use adze_concurrency_caps_core::{
    ConcurrencyCaps, DEFAULT_RAYON_NUM_THREADS, DEFAULT_TOKIO_WORKER_THREADS, bounded_parallel_map,
    normalized_concurrency, parse_positive_usize_or_default,
};
use proptest::prelude::*;

fn model_transform(value: i32) -> i32 {
    value.wrapping_mul(17).wrapping_add(3)
}

proptest! {
    #[test]
    fn bounded_parallel_map_matches_sequential_multiset(
        input in prop::collection::vec(any::<i32>(), 0..256),
        concurrency in 0usize..64,
    ) {
        let mut got = bounded_parallel_map(input.clone(), concurrency, model_transform);
        let mut expected: Vec<i32> = input.into_iter().map(model_transform).collect();

        got.sort_unstable();
        expected.sort_unstable();
        prop_assert_eq!(got, expected);
    }

    #[test]
    fn bounded_parallel_map_preserves_output_length(
        input in prop::collection::vec(any::<i32>(), 0..256),
        concurrency in 0usize..64,
    ) {
        let got = bounded_parallel_map(input.clone(), concurrency, model_transform);
        prop_assert_eq!(got.len(), input.len());
    }

    #[test]
    fn normalized_concurrency_never_returns_zero(value in any::<usize>()) {
        prop_assert!(normalized_concurrency(value) >= 1);
    }

    #[test]
    fn result_multiset_is_independent_of_concurrency(
        input in prop::collection::vec(any::<i32>(), 0..256),
        c1 in 0usize..64,
        c2 in 0usize..64,
    ) {
        let mut r1 = bounded_parallel_map(input.clone(), c1, model_transform);
        let mut r2 = bounded_parallel_map(input, c2, model_transform);

        r1.sort_unstable();
        r2.sort_unstable();
        prop_assert_eq!(r1, r2);
    }

    #[test]
    fn normalized_concurrency_is_idempotent(value in any::<usize>()) {
        let once = normalized_concurrency(value);
        let twice = normalized_concurrency(once);
        prop_assert_eq!(once, twice);
    }

    #[test]
    fn caps_from_lookup_fields_always_positive(
        rayon_str in prop::option::of("[0-9]{0,6}"),
        tokio_str in prop::option::of("[0-9]{0,6}"),
    ) {
        let caps = ConcurrencyCaps::from_lookup(|name| {
            match name {
                "RAYON_NUM_THREADS" => rayon_str.clone(),
                "TOKIO_WORKER_THREADS" => tokio_str.clone(),
                _ => None,
            }
        });
        prop_assert!(caps.rayon_threads >= 1,
            "rayon_threads was 0 for input {:?}", rayon_str);
        prop_assert!(caps.tokio_worker_threads >= 1,
            "tokio_worker_threads was 0 for input {:?}", tokio_str);
    }

    #[test]
    fn caps_from_lookup_none_returns_defaults(_dummy in 0u8..1) {
        let caps = ConcurrencyCaps::from_lookup(|_| None);
        prop_assert_eq!(caps.rayon_threads, DEFAULT_RAYON_NUM_THREADS);
        prop_assert_eq!(caps.tokio_worker_threads, DEFAULT_TOKIO_WORKER_THREADS);
    }

    #[test]
    fn parse_positive_usize_always_positive(
        raw in prop::option::of(".*"),
        default in 1usize..1000,
    ) {
        let result = parse_positive_usize_or_default(raw.as_deref(), default);
        prop_assert!(result >= 1, "parse returned 0 for input {:?} default {}", raw, default);
    }

    #[test]
    fn parse_positive_usize_valid_roundtrip(value in 1usize..100_000) {
        let s = value.to_string();
        let result = parse_positive_usize_or_default(Some(&s), 999);
        prop_assert_eq!(result, value);
    }

    #[test]
    fn empty_input_always_empty(concurrency in 0usize..64) {
        let result: Vec<i32> = bounded_parallel_map(Vec::new(), concurrency, model_transform);
        prop_assert!(result.is_empty());
    }
}
