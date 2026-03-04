//! Shared constants for concurrency environment configuration.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

/// Environment variable used for Rayon global thread-pool caps.
pub const RAYON_NUM_THREADS_ENV: &str = "RAYON_NUM_THREADS";

/// Environment variable used for Tokio worker-thread caps.
pub const TOKIO_WORKER_THREADS_ENV: &str = "TOKIO_WORKER_THREADS";

/// Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid.
pub const DEFAULT_RAYON_NUM_THREADS: usize = 4;

/// Default worker count used for Tokio when `TOKIO_WORKER_THREADS` is unset/invalid.
pub const DEFAULT_TOKIO_WORKER_THREADS: usize = 2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_var_constants_match_expected_names() {
        assert_eq!(RAYON_NUM_THREADS_ENV, "RAYON_NUM_THREADS");
        assert_eq!(TOKIO_WORKER_THREADS_ENV, "TOKIO_WORKER_THREADS");
    }

    #[test]
    fn defaults_match_expected_values() {
        assert_eq!(DEFAULT_RAYON_NUM_THREADS, 4);
        assert_eq!(DEFAULT_TOKIO_WORKER_THREADS, 2);
    }
}
