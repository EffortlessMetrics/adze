use adze_concurrency_plan_core::{
    DIRECT_PARALLEL_THRESHOLD_MULTIPLIER, ParallelPartitionPlan, normalized_concurrency,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn normalized_concurrency_never_returns_zero(value in any::<usize>()) {
        prop_assert!(normalized_concurrency(value) >= 1);
    }

    #[test]
    fn plan_concurrency_matches_normalization(
        item_count in 0usize..4096,
        requested_concurrency in any::<usize>(),
    ) {
        let plan = ParallelPartitionPlan::for_item_count(item_count, requested_concurrency);
        prop_assert_eq!(plan.concurrency, normalized_concurrency(requested_concurrency));
    }

    #[test]
    fn plan_chunk_size_is_valid(
        item_count in 0usize..4096,
        requested_concurrency in any::<usize>(),
    ) {
        let plan = ParallelPartitionPlan::for_item_count(item_count, requested_concurrency);
        prop_assert!(plan.chunk_size >= 1);

        if item_count == 0 {
            prop_assert_eq!(plan.chunk_size, 1);
        } else {
            prop_assert_eq!(plan.chunk_size, item_count.div_ceil(plan.concurrency));
            prop_assert!(plan.chunk_size <= item_count);
        }
    }

    #[test]
    fn direct_parallel_decision_matches_contract(
        item_count in 0usize..4096,
        requested_concurrency in any::<usize>(),
    ) {
        let plan = ParallelPartitionPlan::for_item_count(item_count, requested_concurrency);
        let cutoff = plan
            .concurrency
            .saturating_mul(DIRECT_PARALLEL_THRESHOLD_MULTIPLIER);
        prop_assert_eq!(plan.use_direct_parallel_iter, item_count <= cutoff);
    }
}
