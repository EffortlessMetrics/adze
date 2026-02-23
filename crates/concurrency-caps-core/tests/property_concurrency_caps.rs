use adze_concurrency_caps_core::{bounded_parallel_map, normalized_concurrency};
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
}
