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
pub use adze_concurrency_init_rayon_core::{init_rayon_global_once, is_already_initialized_error};

/// Initialize Rayon global thread-pool caps once for the process.
///
/// Calling this function multiple times is safe and idempotent.
pub fn init_concurrency_caps() {
    let caps = current_caps();

    if let Err(message) = init_rayon_global_once(caps.rayon_threads) {
        panic!("failed to initialize rayon global thread pool: {message}");
    }

    eprintln!(
        "Concurrency caps initialized: {RAYON_NUM_THREADS_ENV}={}, {TOKIO_WORKER_THREADS_ENV}={}",
        caps.rayon_threads, caps.tokio_worker_threads
    );
}

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
}
