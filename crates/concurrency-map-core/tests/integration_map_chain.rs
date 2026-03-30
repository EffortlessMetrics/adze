//! Cross-crate integration tests for the map chain:
//! `concurrency-normalize-core` → `concurrency-plan-core` → `concurrency-bounded-map-core` → `concurrency-map-core`
//!
//! These tests validate that the parallel map chain works correctly end-to-end.

use adze_concurrency_bounded_map_core::{
    ParallelPartitionPlan as BoundedPlan, bounded_parallel_map as bounded_map,
    normalized_concurrency as bounded_normalized,
};
use adze_concurrency_map_core::{
    ParallelPartitionPlan, bounded_parallel_map, normalized_concurrency,
};
use adze_concurrency_normalize_core::{
    MIN_CONCURRENCY, normalized_concurrency as normalize_normalized,
};
use adze_concurrency_plan_core::{
    ParallelPartitionPlan as PlanPlan, normalized_concurrency as plan_normalized,
};

/// Test that the map chain correctly normalizes concurrency values.
#[test]
fn test_map_chain_normalizes_concurrency() {
    // All levels should normalize zero to one
    assert_eq!(normalized_concurrency(0), MIN_CONCURRENCY);
    assert_eq!(bounded_normalized(0), MIN_CONCURRENCY);
    assert_eq!(plan_normalized(0), MIN_CONCURRENCY);
    assert_eq!(normalize_normalized(0), MIN_CONCURRENCY);

    // All levels should preserve positive values
    for value in [1, 2, 4, 8, 16, 64, 256] {
        assert_eq!(normalized_concurrency(value), value);
        assert_eq!(bounded_normalized(value), value);
        assert_eq!(plan_normalized(value), value);
        assert_eq!(normalize_normalized(value), value);
    }
}

/// Test that bounded_parallel_map works correctly across the chain.
#[test]
fn test_map_chain_bounded_parallel_map_produces_correct_results() {
    let input: Vec<i32> = (0..100).collect();

    // Test at various concurrency levels
    for concurrency in [0, 1, 2, 4, 8, 16] {
        let mut result = bounded_parallel_map(input.clone(), concurrency, |x| x * 2);
        result.sort_unstable();

        let expected: Vec<i32> = input.iter().map(|x| x * 2).collect();
        assert_eq!(result, expected, "Failed at concurrency={concurrency}");
    }
}

/// Test that ParallelPartitionPlan is consistent across the chain.
#[test]
fn test_map_chain_partition_plan_consistency() {
    for item_count in [0, 1, 10, 100, 1000] {
        for concurrency in [0, 1, 2, 4, 8] {
            let map_plan = ParallelPartitionPlan::for_item_count(item_count, concurrency);
            let bounded_plan = BoundedPlan::for_item_count(item_count, concurrency);
            let core_plan = PlanPlan::for_item_count(item_count, concurrency);

            assert_eq!(
                map_plan, bounded_plan,
                "map vs bounded at items={item_count}, concurrency={concurrency}"
            );
            assert_eq!(
                map_plan, core_plan,
                "map vs plan at items={item_count}, concurrency={concurrency}"
            );
        }
    }
}

/// Test that the chain handles empty input correctly.
#[test]
fn test_map_chain_handles_empty_input() {
    let empty: Vec<i32> = vec![];

    let result = bounded_parallel_map(empty.clone(), 4, |x: i32| x * 2);
    assert!(result.is_empty());

    let result2 = bounded_map(empty, 0, |x: i32| x * 2);
    assert!(result2.is_empty());
}

/// Test that the chain handles single-element input.
#[test]
fn test_map_chain_handles_single_element() {
    let single = vec![42];

    let result = bounded_parallel_map(single.clone(), 4, |x| x + 1);
    assert_eq!(result, vec![43]);

    let result2 = bounded_map(single, 0, |x| x + 1);
    assert_eq!(result2, vec![43]);
}

/// Test that the chain preserves output order characteristics.
#[test]
fn test_map_chain_preserves_output_count() {
    for size in [1_usize, 10, 100, 1000] {
        let input: Vec<i32> = (0..size as i32).collect();

        for concurrency in [0, 1, 2, 4] {
            let result = bounded_parallel_map(input.clone(), concurrency, |x| x);
            assert_eq!(
                result.len(),
                size,
                "Size mismatch at size={size}, concurrency={concurrency}"
            );
        }
    }
}

/// Test that the partition plan correctly decides on direct vs chunked iteration.
#[test]
fn test_map_chain_partition_plan_chunking_decision() {
    // Small workloads should use direct iteration
    let small_plan = ParallelPartitionPlan::for_item_count(4, 4);
    assert!(
        small_plan.use_direct_parallel_iter,
        "Small workload should use direct iteration"
    );

    // Large workloads should use chunking
    let large_plan = ParallelPartitionPlan::for_item_count(1000, 4);
    assert!(
        !large_plan.use_direct_parallel_iter,
        "Large workload should use chunking"
    );
}

/// Test type compatibility across the chain.
#[test]
fn test_map_chain_type_compatibility() {
    fn accepts_plan_fn(f: fn(usize, usize) -> ParallelPartitionPlan) -> ParallelPartitionPlan {
        f(10, 4)
    }

    let plan = accepts_plan_fn(ParallelPartitionPlan::for_item_count);
    assert!(plan.concurrency >= 1);
    assert!(plan.chunk_size >= 1);
}

/// Test that the chain works with complex transformations.
#[test]
fn test_map_chain_complex_transformation() {
    let input: Vec<String> = vec!["hello".to_string(), "world".to_string()];

    let result = bounded_parallel_map(input, 2, |s| s.to_uppercase());
    assert!(result.contains(&"HELLO".to_string()));
    assert!(result.contains(&"WORLD".to_string()));
}
