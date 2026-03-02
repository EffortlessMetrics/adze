//! Environment-backed concurrency cap policy and parsing helpers.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use std::env;

use adze_concurrency_policy_core::resolve_caps_from_lookup;
pub use adze_concurrency_policy_core::{
    DEFAULT_RAYON_NUM_THREADS, DEFAULT_TOKIO_WORKER_THREADS, RAYON_NUM_THREADS_ENV,
    TOKIO_WORKER_THREADS_ENV, parse_positive_usize_or_default,
};

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
    pub fn from_lookup<F>(lookup: F) -> Self
    where
        F: FnMut(&str) -> Option<String>,
    {
        let (rayon_threads, tokio_worker_threads) = resolve_caps_from_lookup(lookup);
        Self {
            rayon_threads,
            tokio_worker_threads,
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
    fn from_lookup_uses_defaults_when_unset() {
        let caps = ConcurrencyCaps::from_lookup(|_| None);
        assert_eq!(caps, ConcurrencyCaps::default());
    }
}
