//! Core bounded parallel map utilities built on deterministic partition planning.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

/// Re-exported bounded parallel map and partition planning utilities.
pub use adze_concurrency_bounded_map_core::{
    ParallelPartitionPlan, bounded_parallel_map, normalized_concurrency,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_concurrency_clamps_zero_to_one() {
        assert_eq!(normalized_concurrency(0), 1);
    }

    #[test]
    fn normalized_concurrency_preserves_positive() {
        assert_eq!(normalized_concurrency(1), 1);
        assert_eq!(normalized_concurrency(4), 4);
    }

    #[test]
    fn bounded_parallel_map_empty_input() {
        let result: Vec<i32> = bounded_parallel_map(vec![], 4, |x: i32| x + 1);
        assert!(result.is_empty());
    }

    #[test]
    fn bounded_parallel_map_single_item() {
        let result = bounded_parallel_map(vec![42], 4, |x| x * 2);
        assert_eq!(result, vec![84]);
    }

    #[test]
    fn partition_plan_for_empty() {
        let plan = ParallelPartitionPlan::for_item_count(0, 4);
        // Even for empty input, chunk_size is at least 1 (items/concurrency rounded up)
        assert!(plan.chunk_size >= 1);
    }
}
