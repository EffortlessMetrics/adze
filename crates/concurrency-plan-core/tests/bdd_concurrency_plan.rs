use adze_concurrency_plan_core::{ParallelPartitionPlan, normalized_concurrency};

#[test]
fn given_zero_requested_concurrency_when_normalizing_then_single_worker_is_used() {
    // Given / When
    let normalized = normalized_concurrency(0);

    // Then
    assert_eq!(normalized, 1);
}

#[test]
fn given_small_workload_when_planning_then_direct_parallel_iteration_is_preferred() {
    // Given / When
    let plan = ParallelPartitionPlan::for_item_count(8, 4);

    // Then
    assert_eq!(plan.concurrency, 4);
    assert_eq!(plan.chunk_size, 2);
    assert!(plan.use_direct_parallel_iter);
}

#[test]
fn given_large_workload_when_planning_then_chunking_strategy_is_applied() {
    // Given / When
    let plan = ParallelPartitionPlan::for_item_count(257, 4);

    // Then
    assert_eq!(plan.concurrency, 4);
    assert_eq!(plan.chunk_size, 65);
    assert!(!plan.use_direct_parallel_iter);
}

#[test]
fn given_empty_workload_when_planning_then_chunk_size_remains_non_zero() {
    // Given / When
    let plan = ParallelPartitionPlan::for_item_count(0, 32);

    // Then
    assert_eq!(plan.chunk_size, 1);
    assert!(plan.use_direct_parallel_iter);
}
