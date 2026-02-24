use adze_concurrency_caps_core::{
    ParallelPartitionPlan as CapsParallelPartitionPlan,
    normalized_concurrency as caps_normalized_concurrency,
};
use adze_concurrency_plan_core::{
    ParallelPartitionPlan as CoreParallelPartitionPlan,
    normalized_concurrency as core_normalized_concurrency,
};

#[test]
fn caps_core_reexport_matches_plan_core() {
    for item_count in 0..=256 {
        for concurrency in 0..=32 {
            let caps_plan = CapsParallelPartitionPlan::for_item_count(item_count, concurrency);
            let core_plan = CoreParallelPartitionPlan::for_item_count(item_count, concurrency);
            assert_eq!(
                caps_plan, core_plan,
                "item_count={item_count}, concurrency={concurrency}"
            );
        }
    }
}

#[test]
fn normalized_concurrency_reexport_matches_plan_core() {
    for value in [0, 1, 2, 8, 64, usize::MAX] {
        assert_eq!(
            caps_normalized_concurrency(value),
            core_normalized_concurrency(value)
        );
    }
}
