use adze::concurrency_caps::{
    ParallelPartitionPlan as RuntimeParallelPartitionPlan,
    normalized_concurrency as runtime_normalized_concurrency,
};
use adze_concurrency_plan_core::{
    ParallelPartitionPlan as CoreParallelPartitionPlan,
    normalized_concurrency as core_normalized_concurrency,
};

#[test]
fn runtime_reexport_matches_microcrate_partition_plans() {
    for item_count in 0..=256 {
        for concurrency in 0..=32 {
            let runtime_plan =
                RuntimeParallelPartitionPlan::for_item_count(item_count, concurrency);
            let core_plan = CoreParallelPartitionPlan::for_item_count(item_count, concurrency);
            assert_eq!(
                runtime_plan, core_plan,
                "item_count={item_count}, concurrency={concurrency}"
            );
        }
    }
}

#[test]
fn runtime_reexport_stays_type_compatible() {
    fn accepts_core_type(value: CoreParallelPartitionPlan) -> CoreParallelPartitionPlan {
        value
    }

    let runtime_value = RuntimeParallelPartitionPlan::for_item_count(257, 4);
    let returned = accepts_core_type(runtime_value);

    assert_eq!(returned.concurrency, 4);
    assert_eq!(returned.chunk_size, 65);
    assert!(!returned.use_direct_parallel_iter);
}

#[test]
fn runtime_reexport_uses_same_normalization_contract() {
    for value in [0, 1, 2, 8, 64, usize::MAX] {
        assert_eq!(
            runtime_normalized_concurrency(value),
            core_normalized_concurrency(value)
        );
    }
}
