use adze_concurrency_plan_core::ParallelPartitionPlan;

#[test]
fn contract_direct_parallel_threshold_boundary_is_stable() {
    let at_boundary = ParallelPartitionPlan::for_item_count(8, 4);
    let above_boundary = ParallelPartitionPlan::for_item_count(9, 4);

    assert!(at_boundary.use_direct_parallel_iter);
    assert!(!above_boundary.use_direct_parallel_iter);
}

#[test]
fn contract_zero_items_always_use_chunk_size_one() {
    let plan = ParallelPartitionPlan::for_item_count(0, 0);
    assert_eq!(plan.chunk_size, 1);
    assert_eq!(plan.concurrency, 1);
}
