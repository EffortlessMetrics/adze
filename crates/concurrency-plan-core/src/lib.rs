//! Pure policy helpers for bounded parallel partition planning.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

/// Minimum valid concurrency value.
pub const MIN_CONCURRENCY: usize = 1;

/// Workloads at or below `concurrency * DIRECT_PARALLEL_THRESHOLD_MULTIPLIER`
/// prefer direct parallel iteration over chunk partitioning.
pub const DIRECT_PARALLEL_THRESHOLD_MULTIPLIER: usize = 2;

/// Normalize a requested concurrency value.
///
/// A value of `0` is treated as `1` to avoid invalid worker counts.
#[must_use]
pub const fn normalized_concurrency(concurrency: usize) -> usize {
    if concurrency == 0 {
        MIN_CONCURRENCY
    } else {
        concurrency
    }
}

/// Planning metadata for bounded parallel partitioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParallelPartitionPlan {
    /// Effective non-zero concurrency used by the plan.
    pub concurrency: usize,
    /// Chunk size to use for partitioned processing. Guaranteed to be at least `1`.
    pub chunk_size: usize,
    /// Whether direct parallel iteration is preferred over chunk partitioning.
    pub use_direct_parallel_iter: bool,
}

impl ParallelPartitionPlan {
    /// Build a partition plan for `item_count` items and requested concurrency.
    #[must_use]
    pub fn for_item_count(item_count: usize, requested_concurrency: usize) -> Self {
        let concurrency = normalized_concurrency(requested_concurrency);
        let use_direct_parallel_iter =
            item_count <= concurrency.saturating_mul(DIRECT_PARALLEL_THRESHOLD_MULTIPLIER);
        let chunk_size = if item_count == 0 {
            1
        } else {
            item_count.div_ceil(concurrency)
        };

        Self {
            concurrency,
            chunk_size,
            use_direct_parallel_iter,
        }
    }
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
    fn plan_for_empty_work_is_safe() {
        let plan = ParallelPartitionPlan::for_item_count(0, 0);
        assert_eq!(plan.concurrency, 1);
        assert_eq!(plan.chunk_size, 1);
        assert!(plan.use_direct_parallel_iter);
    }

    #[test]
    fn plan_for_large_work_uses_chunking() {
        let plan = ParallelPartitionPlan::for_item_count(257, 4);
        assert_eq!(plan.concurrency, 4);
        assert_eq!(plan.chunk_size, 65);
        assert!(!plan.use_direct_parallel_iter);
    }
}
