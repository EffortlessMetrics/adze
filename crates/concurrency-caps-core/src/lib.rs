//! Compatibility façade for concurrency cap primitives and contracts.
//!
//! # Examples
//!
//! ```
//! use adze_concurrency_caps_core::{ConcurrencyCaps, parse_positive_usize_or_default};
//!
//! // Build caps from a custom lookup (no env mutation)
//! let caps = ConcurrencyCaps::from_lookup(|_| None);
//! assert_eq!(caps.rayon_threads, 4);   // default
//! assert_eq!(caps.tokio_worker_threads, 2); // default
//!
//! // Parse helper returns default for invalid input
//! assert_eq!(parse_positive_usize_or_default(Some("0"), 8), 8);
//! assert_eq!(parse_positive_usize_or_default(Some("16"), 8), 16);
//! ```

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

/// Environment-based concurrency cap configuration and defaults.
pub use adze_concurrency_env_core::{
    ConcurrencyCaps, DEFAULT_RAYON_NUM_THREADS, DEFAULT_TOKIO_WORKER_THREADS,
    RAYON_NUM_THREADS_ENV, TOKIO_WORKER_THREADS_ENV, current_caps, parse_positive_usize_or_default,
};
/// One-time concurrency initialization helpers (rayon global pool, caps bootstrap).
pub use adze_concurrency_init_core::{
    init_concurrency_caps, init_rayon_global_once, is_already_initialized_error,
};
/// Bounded parallel map and partition planning utilities.
pub use adze_concurrency_map_core::{
    ParallelPartitionPlan, bounded_parallel_map, normalized_concurrency,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_defaults_via_facade() {
        let caps = ConcurrencyCaps::from_lookup(|_| None);
        assert_eq!(caps.rayon_threads, DEFAULT_RAYON_NUM_THREADS);
        assert_eq!(caps.tokio_worker_threads, DEFAULT_TOKIO_WORKER_THREADS);
    }

    #[test]
    fn parse_helper_via_facade() {
        assert_eq!(parse_positive_usize_or_default(Some("10"), 1), 10);
        assert_eq!(parse_positive_usize_or_default(Some("0"), 5), 5);
        assert_eq!(parse_positive_usize_or_default(None, 3), 3);
    }

    #[test]
    fn bounded_map_via_facade() {
        let mut result = bounded_parallel_map(vec![1, 2, 3], 2, |x| x * 10);
        result.sort();
        assert_eq!(result, vec![10, 20, 30]);
    }

    #[test]
    fn normalized_concurrency_via_facade() {
        assert_eq!(normalized_concurrency(0), 1);
        assert_eq!(normalized_concurrency(4), 4);
    }

    #[test]
    fn partition_plan_via_facade() {
        let plan = ParallelPartitionPlan::for_item_count(10, 3);
        assert!(plan.chunk_size > 0);
    }
}
