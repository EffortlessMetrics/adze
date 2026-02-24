use adze_concurrency_map_core::{ParallelPartitionPlan, bounded_parallel_map};

#[test]
fn contract_zero_concurrency_matches_single_worker_behavior() {
    let input: Vec<i64> = (0..128).collect();

    let mut zero = bounded_parallel_map(input.clone(), 0, |value| value * 2);
    let mut one = bounded_parallel_map(input, 1, |value| value * 2);

    zero.sort_unstable();
    one.sort_unstable();
    assert_eq!(zero, one);
}

#[test]
fn contract_threshold_boundary_stays_stable() {
    let at_boundary = ParallelPartitionPlan::for_item_count(8, 4);
    let above_boundary = ParallelPartitionPlan::for_item_count(9, 4);

    assert!(at_boundary.use_direct_parallel_iter);
    assert!(!above_boundary.use_direct_parallel_iter);
}
