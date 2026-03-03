//! Contract-focused re-export layer for concurrency caps utilities.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_concurrency_env_core::{
    ConcurrencyCaps, DEFAULT_RAYON_NUM_THREADS, DEFAULT_TOKIO_WORKER_THREADS,
    RAYON_NUM_THREADS_ENV, TOKIO_WORKER_THREADS_ENV, current_caps, parse_positive_usize_or_default,
};
pub use adze_concurrency_init_core::{
    init_concurrency_caps, init_rayon_global_once, is_already_initialized_error,
};
pub use adze_concurrency_map_core::{
    ParallelPartitionPlan, bounded_parallel_map, normalized_concurrency,
};

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
    fn init_is_idempotent() {
        init_concurrency_caps();
        init_concurrency_caps();
    }

    #[test]
    fn low_level_rayon_init_is_idempotent() {
        assert!(init_rayon_global_once(1).is_ok());
        assert!(init_rayon_global_once(8).is_ok());
    }

    #[test]
    fn already_initialized_error_classifier_is_case_insensitive() {
        assert!(is_already_initialized_error(
            "The GlObAl thread pool has AlReAdY been initialized"
        ));
    }

    #[test]
    fn default_caps_have_expected_constants() {
        let caps = ConcurrencyCaps::default();
        assert_eq!(caps.rayon_threads, DEFAULT_RAYON_NUM_THREADS);
        assert_eq!(caps.tokio_worker_threads, DEFAULT_TOKIO_WORKER_THREADS);
    }

    #[test]
    fn env_var_constants_are_correct() {
        assert_eq!(RAYON_NUM_THREADS_ENV, "RAYON_NUM_THREADS");
        assert_eq!(TOKIO_WORKER_THREADS_ENV, "TOKIO_WORKER_THREADS");
    }

    #[test]
    fn current_caps_returns_valid_values() {
        let caps = current_caps();
        assert!(caps.rayon_threads >= 1 || caps.rayon_threads == DEFAULT_RAYON_NUM_THREADS);
        assert!(
            caps.tokio_worker_threads >= 1
                || caps.tokio_worker_threads == DEFAULT_TOKIO_WORKER_THREADS
        );
    }

    #[test]
    fn parse_positive_usize_or_default_basic_behavior() {
        assert_eq!(parse_positive_usize_or_default(None, 10), 10);
        assert_eq!(parse_positive_usize_or_default(Some("0"), 5), 5);
        assert_eq!(parse_positive_usize_or_default(Some("42"), 1), 42);
        assert_eq!(parse_positive_usize_or_default(Some("invalid"), 3), 3);
    }

    #[test]
    fn bounded_parallel_map_single_element() {
        let result = bounded_parallel_map(vec![99], 4, |x| x + 1);
        assert_eq!(result, vec![100]);
    }

    #[test]
    fn partition_plan_zero_items_is_safe() {
        let plan = ParallelPartitionPlan::for_item_count(0, 4);
        assert!(plan.use_direct_parallel_iter);
        assert!(plan.chunk_size >= 1);
    }
}
