//! Rayon global thread-pool initialization utilities for process-wide concurrency caps.

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
pub use adze_concurrency_init_bootstrap_core::init_concurrency_caps;
pub use adze_concurrency_init_rayon_core::{init_rayon_global_once, is_already_initialized_error};

#[cfg(test)]
mod tests {
    use super::*;

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
    fn already_initialized_error_classifier_requires_both_tokens() {
        assert!(!is_already_initialized_error(
            "thread pool already initialized"
        ));
        assert!(!is_already_initialized_error(
            "global thread pool initialized"
        ));
    }

    #[test]
    fn default_caps_have_expected_constants() {
        assert_eq!(DEFAULT_RAYON_NUM_THREADS, 4);
        assert_eq!(DEFAULT_TOKIO_WORKER_THREADS, 2);
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
    fn parse_positive_usize_or_default_returns_default_for_none() {
        assert_eq!(parse_positive_usize_or_default(None, 7), 7);
    }

    #[test]
    fn parse_positive_usize_or_default_returns_default_for_zero() {
        assert_eq!(parse_positive_usize_or_default(Some("0"), 5), 5);
    }

    #[test]
    fn parse_positive_usize_or_default_parses_valid_value() {
        assert_eq!(parse_positive_usize_or_default(Some("42"), 1), 42);
    }

    #[test]
    fn env_var_constants_match_expected_names() {
        assert_eq!(RAYON_NUM_THREADS_ENV, "RAYON_NUM_THREADS");
        assert_eq!(TOKIO_WORKER_THREADS_ENV, "TOKIO_WORKER_THREADS");
    }

    #[test]
    fn concurrency_caps_default_matches_constants() {
        let caps = ConcurrencyCaps::default();
        assert_eq!(caps.rayon_threads, DEFAULT_RAYON_NUM_THREADS);
        assert_eq!(caps.tokio_worker_threads, DEFAULT_TOKIO_WORKER_THREADS);
    }
}
