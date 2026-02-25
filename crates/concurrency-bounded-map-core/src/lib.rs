//! Core bounded parallel map implementation.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use rayon::prelude::*;

pub use adze_concurrency_plan_core::{ParallelPartitionPlan, normalized_concurrency};

/// Run a bounded parallel map operation.
///
/// This keeps work partitioned by `concurrency`, while preserving all outputs.
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
}
