//! Environment-backed concurrency cap policy and parsing helpers.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use std::env;

pub use adze_concurrency_parse_core::parse_positive_usize_or_default;

/// Environment variable used for Rayon global thread-pool caps.
pub const RAYON_NUM_THREADS_ENV: &str = "RAYON_NUM_THREADS";

/// Environment variable used for Tokio worker-thread caps.
pub const TOKIO_WORKER_THREADS_ENV: &str = "TOKIO_WORKER_THREADS";

/// Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid.
pub const DEFAULT_RAYON_NUM_THREADS: usize = 4;

/// Default worker count used for Tokio when `TOKIO_WORKER_THREADS` is unset/invalid.
pub const DEFAULT_TOKIO_WORKER_THREADS: usize = 2;

/// Snapshot of active concurrency cap values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConcurrencyCaps {
    /// Rayon global thread-pool thread count.
    pub rayon_threads: usize,
    /// Tokio worker thread count.
    pub tokio_worker_threads: usize,
}

impl ConcurrencyCaps {
    /// Read concurrency caps from process environment with stable defaults.
    #[must_use]
    pub fn from_env() -> Self {
        Self::from_lookup(|name| env::var(name).ok())
    }

    /// Build caps from a lookup function returning optional raw environment values.
    ///
    /// This supports deterministic testing without mutating process-wide environment state.
    #[must_use]
    pub fn from_lookup<F>(mut lookup: F) -> Self
    where
        F: FnMut(&str) -> Option<String>,
    {
        Self {
            rayon_threads: parse_positive_usize_or_default(
                lookup(RAYON_NUM_THREADS_ENV).as_deref(),
                DEFAULT_RAYON_NUM_THREADS,
            ),
            tokio_worker_threads: parse_positive_usize_or_default(
                lookup(TOKIO_WORKER_THREADS_ENV).as_deref(),
                DEFAULT_TOKIO_WORKER_THREADS,
            ),
        }
    }
}

impl Default for ConcurrencyCaps {
    fn default() -> Self {
        Self {
            rayon_threads: DEFAULT_RAYON_NUM_THREADS,
            tokio_worker_threads: DEFAULT_TOKIO_WORKER_THREADS,
        }
    }
}

/// Return the current caps resolved from environment values.
#[must_use]
pub fn current_caps() -> ConcurrencyCaps {
    ConcurrencyCaps::from_env()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_positive_usize_falls_back_when_missing_invalid_or_zero() {
        assert_eq!(parse_positive_usize_or_default(None, 7), 7);
        assert_eq!(parse_positive_usize_or_default(Some(""), 7), 7);
        assert_eq!(parse_positive_usize_or_default(Some("nope"), 7), 7);
        assert_eq!(parse_positive_usize_or_default(Some("0"), 7), 7);
    }

    #[test]
    fn parse_positive_usize_accepts_trimmed_positive_input() {
        assert_eq!(parse_positive_usize_or_default(Some(" 42 "), 7), 42);
    }

    #[test]
    fn from_lookup_uses_defaults_when_unset() {
        let caps = ConcurrencyCaps::from_lookup(|_| None);
        assert_eq!(caps, ConcurrencyCaps::default());
    }
}
