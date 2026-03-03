//! Idempotent Rayon global thread-pool initialization primitives.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use std::sync::OnceLock;

pub use adze_concurrency_init_classifier_core::is_already_initialized_error;

/// Initialize Rayon global thread-pool once for the current process.
///
/// The first invocation determines the process-wide result; all subsequent
/// calls return that same result.
///
/// A `num_threads` value of `0` is normalized to `1`.
pub fn init_rayon_global_once(num_threads: usize) -> Result<(), String> {
    RAYON_INIT_RESULT
        .get_or_init(|| init_rayon_global(num_threads.max(1)))
        .clone()
}

static RAYON_INIT_RESULT: OnceLock<Result<(), String>> = OnceLock::new();

fn init_rayon_global(num_threads: usize) -> Result<(), String> {
    match rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
    {
        Ok(()) => Ok(()),
        Err(error) => {
            let message = error.to_string();
            if is_already_initialized_error(&message) {
                Ok(())
            } else {
                Err(message)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{init_rayon_global_once, is_already_initialized_error};

    #[test]
    fn init_is_idempotent() {
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
    fn init_with_zero_threads_normalizes_and_succeeds() {
        assert!(init_rayon_global_once(0).is_ok());
    }

    #[test]
    fn init_result_is_stable_across_different_thread_counts() {
        let first = init_rayon_global_once(2);
        let second = init_rayon_global_once(64);
        assert_eq!(first, second);
    }

    #[test]
    fn init_with_one_thread_succeeds() {
        assert!(init_rayon_global_once(1).is_ok());
    }

    #[test]
    fn init_with_large_thread_count_succeeds() {
        assert!(init_rayon_global_once(1024).is_ok());
    }
}
