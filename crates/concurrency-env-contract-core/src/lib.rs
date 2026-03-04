//! Shared contracts for concurrency environment configuration.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use std::env;

pub use adze_concurrency_env_vars_core::{
    DEFAULT_RAYON_NUM_THREADS, DEFAULT_TOKIO_WORKER_THREADS, RAYON_NUM_THREADS_ENV,
    TOKIO_WORKER_THREADS_ENV,
};
pub use adze_concurrency_parse_core::parse_positive_usize_or_default;

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
    /// This supports deterministic testing without mutating process-wide environment
    /// state.
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
    fn default_caps_use_expected_constants() {
        let caps = ConcurrencyCaps::default();
        assert_eq!(caps.rayon_threads, DEFAULT_RAYON_NUM_THREADS);
        assert_eq!(caps.tokio_worker_threads, DEFAULT_TOKIO_WORKER_THREADS);
    }

    #[test]
    fn from_lookup_returns_defaults_when_none() {
        let caps = ConcurrencyCaps::from_lookup(|_| None);
        assert_eq!(caps, ConcurrencyCaps::default());
    }

    #[test]
    fn from_lookup_parses_valid_values() {
        let caps = ConcurrencyCaps::from_lookup(|name| match name {
            "RAYON_NUM_THREADS" => Some("16".to_string()),
            "TOKIO_WORKER_THREADS" => Some("8".to_string()),
            _ => None,
        });
        assert_eq!(caps.rayon_threads, 16);
        assert_eq!(caps.tokio_worker_threads, 8);
    }

    #[test]
    fn from_lookup_falls_back_on_zero() {
        let caps = ConcurrencyCaps::from_lookup(|_| Some("0".to_string()));
        assert_eq!(caps, ConcurrencyCaps::default());
    }

    #[test]
    fn from_lookup_falls_back_on_invalid() {
        let caps = ConcurrencyCaps::from_lookup(|_| Some("not_a_number".to_string()));
        assert_eq!(caps, ConcurrencyCaps::default());
    }

    #[test]
    fn clone_and_eq() {
        let caps = ConcurrencyCaps::from_lookup(|_| Some("3".to_string()));
        let cloned = caps;
        assert_eq!(caps, cloned);
    }

    #[test]
    fn debug_format_is_readable() {
        let caps = ConcurrencyCaps::default();
        let dbg = format!("{caps:?}");
        assert!(dbg.contains("rayon_threads"));
        assert!(dbg.contains("tokio_worker_threads"));
    }

    #[test]
    fn env_var_constants_match_expected_names() {
        assert_eq!(RAYON_NUM_THREADS_ENV, "RAYON_NUM_THREADS");
        assert_eq!(TOKIO_WORKER_THREADS_ENV, "TOKIO_WORKER_THREADS");
    }
}
