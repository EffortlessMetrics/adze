//! Cross-crate integration tests for the plan chain:
//! `concurrency-normalize-core` → `concurrency-plan-core`
//!
//! These tests validate that the partition planning chain works correctly end-to-end.

use adze_concurrency_normalize_core::{
    MIN_CONCURRENCY, normalized_concurrency as normalize_normalized,
};
use adze_concurrency_plan_core::{
    DIRECT_PARALLEL_THRESHOLD_MULTIPLIER, ParallelPartitionPlan, normalized_concurrency,
};

/// Test that the plan chain correctly re-exports normalized_concurrency.
#[test]
fn test_plan_chain_normalized_concurrency_reexport() {
    for value in [0, 1, 2, 4, 8, 16, 64, 256, usize::MAX] {
        assert_eq!(
            normalized_concurrency(value),
            normalize_normalized(value),
            "Mismatch at value={value}"
        );
    }
}

/// Test that MIN_CONCURRENCY is correctly used.
#[test]
fn test_plan_chain_min_concurrency() {
    assert_eq!(MIN_CONCURRENCY, 1);
    assert_eq!(normalized_concurrency(0), MIN_CONCURRENCY);
}

/// Test that the plan chain produces valid plans for all inputs.
#[test]
fn test_plan_chain_produces_valid_plans() {
    for item_count in 0..=1000 {
        for concurrency in 0..=16 {
            let plan = ParallelPartitionPlan::for_item_count(item_count, concurrency);

            // Concurrency should always be at least MIN_CONCURRENCY
            assert!(plan.concurrency >= MIN_CONCURRENCY);

            // Chunk size should always be at least 1
            assert!(plan.chunk_size >= 1);
        }
    }
}

/// Test that the plan chain correctly decides on direct vs chunked iteration.
#[test]
fn test_plan_chain_direct_iteration_threshold() {
    // At the threshold, should use direct iteration
    let at_threshold = 4 * DIRECT_PARALLEL_THRESHOLD_MULTIPLIER;
    let plan_at = ParallelPartitionPlan::for_item_count(at_threshold, 4);
    assert!(plan_at.use_direct_parallel_iter);

    // Just above the threshold, should use chunking
    let above_threshold = at_threshold + 1;
    let plan_above = ParallelPartitionPlan::for_item_count(above_threshold, 4);
    assert!(!plan_above.use_direct_parallel_iter);
}

/// Test that the plan chain handles edge cases correctly.
#[test]
fn test_plan_chain_edge_cases() {
    // Zero items
    let zero_plan = ParallelPartitionPlan::for_item_count(0, 0);
    assert_eq!(zero_plan.concurrency, MIN_CONCURRENCY);
    assert_eq!(zero_plan.chunk_size, 1);
    assert!(zero_plan.use_direct_parallel_iter);

    // Single item
    let single_plan = ParallelPartitionPlan::for_item_count(1, 1);
    assert_eq!(single_plan.concurrency, 1);
    assert!(single_plan.use_direct_parallel_iter);

    // Large items, zero concurrency (should normalize to 1)
    let large_plan = ParallelPartitionPlan::for_item_count(10000, 0);
    assert_eq!(large_plan.concurrency, MIN_CONCURRENCY);
    assert_eq!(large_plan.chunk_size, 10000);
}

/// Test that the plan chain chunk size calculation is correct.
#[test]
fn test_plan_chain_chunk_size_calculation() {
    // Exact division
    let plan = ParallelPartitionPlan::for_item_count(100, 4);
    assert_eq!(plan.chunk_size, 25);

    // Non-exact division (should round up)
    let plan = ParallelPartitionPlan::for_item_count(101, 4);
    assert_eq!(plan.chunk_size, 26); // 101 / 4 = 25.25, rounded up = 26

    // More items than concurrency
    let plan = ParallelPartitionPlan::for_item_count(1000, 8);
    assert_eq!(plan.chunk_size, 125);
}

/// Test that the plan chain is consistent with the normalize layer.
#[test]
fn test_plan_chain_consistency_with_normalize() {
    // The plan's concurrency should always match normalized_concurrency
    for requested in 0..=32 {
        let plan = ParallelPartitionPlan::for_item_count(100, requested);
        assert_eq!(plan.concurrency, normalized_concurrency(requested));
        assert_eq!(plan.concurrency, normalize_normalized(requested));
    }
}

/// Test that DIRECT_PARALLEL_THRESHOLD_MULTIPLIER has a sensible value.
#[test]
fn test_plan_chain_threshold_multiplier() {
    // The threshold multiplier should be at least 1
    assert!(DIRECT_PARALLEL_THRESHOLD_MULTIPLIER >= 1);

    // A common value is 2 (meaning use direct iteration when items <= 2 * concurrency)
    assert_eq!(DIRECT_PARALLEL_THRESHOLD_MULTIPLIER, 2);
}

/// Test type compatibility across the plan chain.
#[test]
fn test_plan_chain_type_compatibility() {
    fn accepts_normalize_fn(f: fn(usize) -> usize) -> fn(usize) -> usize {
        f
    }

    // normalized_concurrency from plan-core should be compatible
    let f = accepts_normalize_fn(normalized_concurrency);
    assert_eq!(f(0), MIN_CONCURRENCY);
    assert_eq!(f(4), 4);
}
