use adze_concurrency_map_core::{
    ParallelPartitionPlan, bounded_parallel_map, normalized_concurrency,
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
    fn normalized_concurrency_preserves_positive(value in 1usize..10_000) {
        prop_assert_eq!(normalized_concurrency(value), value);
    }

    #[test]
    fn identity_map_preserves_all_values(
        input in prop::collection::vec(any::<i64>(), 0..128),
        concurrency in 1usize..16,
    ) {
        let mut result = bounded_parallel_map(input.clone(), concurrency, |x| x);
        result.sort_unstable();
        let mut expected = input;
        expected.sort_unstable();
        prop_assert_eq!(result, expected);
    }

    #[test]
    fn double_application_is_composable(
        input in prop::collection::vec(any::<i32>(), 0..64),
        concurrency in 1usize..16,
    ) {
        let once = bounded_parallel_map(input.clone(), concurrency, model_transform);
        let mut twice = bounded_parallel_map(once.clone(), concurrency, model_transform);
        let mut expected: Vec<i32> = input.into_iter()
            .map(|x| model_transform(model_transform(x)))
            .collect();

        twice.sort_unstable();
        expected.sort_unstable();
        prop_assert_eq!(twice, expected);
    }

    #[test]
    fn plan_invariants_hold(
        item_count in 0usize..1024,
        concurrency in 0usize..128,
    ) {
        let plan = ParallelPartitionPlan::for_item_count(item_count, concurrency);
        prop_assert!(plan.concurrency >= 1);
        prop_assert!(plan.chunk_size >= 1);
        if item_count > 0 {
            prop_assert!(plan.chunk_size * plan.concurrency >= item_count);
        }
    }

    #[test]
    fn empty_input_always_empty(concurrency in 0usize..64) {
        let result: Vec<i32> = bounded_parallel_map(Vec::new(), concurrency, model_transform);
        prop_assert!(result.is_empty());
    }
}
