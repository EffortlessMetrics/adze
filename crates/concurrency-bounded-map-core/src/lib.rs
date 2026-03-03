//! Core bounded parallel map implementation.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use rayon::prelude::*;

/// Re-exported partition planning types used by the bounded parallel map.
pub use adze_concurrency_plan_core::{ParallelPartitionPlan, normalized_concurrency};

/// Run a bounded parallel map operation.
///
/// This keeps work partitioned by `concurrency`, while preserving all outputs.
///
/// # Examples
///
/// ```
/// use adze_concurrency_bounded_map_core::bounded_parallel_map;
///
/// let input: Vec<i32> = (0..10).collect();
/// let mut result = bounded_parallel_map(input, 4, |x| x * 2);
/// result.sort();
/// assert_eq!(result, vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18]);
/// ```
#[must_use]
pub fn bounded_parallel_map<T, R, F>(items: Vec<T>, concurrency: usize, f: F) -> Vec<R>
where
    T: Send,
    R: Send,
    F: Fn(T) -> R + Send + Sync,
{
    let plan = ParallelPartitionPlan::for_item_count(items.len(), concurrency);

    if items.is_empty() {
        return Vec::new();
    }

    if plan.use_direct_parallel_iter {
        return items.into_par_iter().map(f).collect();
    }

    items
        .into_par_iter()
        .chunks(plan.chunk_size)
        .flat_map(|chunk| chunk.into_iter().map(&f).collect::<Vec<_>>())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_concurrency_is_never_zero() {
        assert_eq!(normalized_concurrency(0), 1);
        assert_eq!(normalized_concurrency(1), 1);
        assert_eq!(normalized_concurrency(8), 8);
    }

    #[test]
    fn bounded_parallel_map_handles_zero_concurrency() {
        let mut result = bounded_parallel_map((0..64).collect::<Vec<_>>(), 0, |x| x * 2);
        result.sort_unstable();

        let expected: Vec<i32> = (0..64).map(|x| x * 2).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn bounded_parallel_map_handles_empty_input() {
        let output: Vec<i32> = bounded_parallel_map(Vec::<i32>::new(), 8, |value| value * 2);
        assert!(output.is_empty());
    }

    #[test]
    fn bounded_parallel_map_single_element() {
        let result = bounded_parallel_map(vec![7], 4, |x| x + 1);
        assert_eq!(result, vec![8]);
    }

    #[test]
    fn bounded_parallel_map_concurrency_exceeds_items() {
        let mut result = bounded_parallel_map(vec![1, 2, 3], 100, |x| x * 10);
        result.sort_unstable();
        assert_eq!(result, vec![10, 20, 30]);
    }

    #[test]
    fn bounded_parallel_map_preserves_output_length() {
        let input: Vec<i32> = (0..100).collect();
        let result = bounded_parallel_map(input.clone(), 4, |x| x * 2);
        assert_eq!(result.len(), input.len());
    }

    #[test]
    fn bounded_parallel_map_with_concurrency_one() {
        let mut result = bounded_parallel_map(vec![3, 1, 4, 1, 5], 1, |x| x * x);
        result.sort_unstable();
        assert_eq!(result, vec![1, 1, 9, 16, 25]);
    }

    #[test]
    fn partition_plan_zero_concurrency_normalizes() {
        let plan = ParallelPartitionPlan::for_item_count(10, 0);
        assert!(plan.concurrency >= 1);
        assert!(plan.chunk_size >= 1);
    }

    #[test]
    fn partition_plan_empty_items_is_safe() {
        let plan = ParallelPartitionPlan::for_item_count(0, 4);
        assert!(plan.use_direct_parallel_iter);
        assert!(plan.chunk_size >= 1);
    }
}
