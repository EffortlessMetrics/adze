//! Pure policy for bootstrap concurrency-caps normalization.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use adze_concurrency_env_core::ConcurrencyCaps;
use adze_concurrency_normalize_core::normalized_concurrency;

/// Normalize bootstrap caps to a safe, process-init-ready configuration.
#[must_use]
pub fn bootstrap_caps(caps: ConcurrencyCaps) -> ConcurrencyCaps {
    ConcurrencyCaps {
        rayon_threads: normalized_concurrency(caps.rayon_threads),
        tokio_worker_threads: caps.tokio_worker_threads,
    }
}

#[cfg(test)]
mod tests {
    use super::bootstrap_caps;
    use adze_concurrency_env_core::{ConcurrencyCaps, DEFAULT_TOKIO_WORKER_THREADS};

    #[test]
    fn bootstrap_caps_normalizes_zero_rayon_threads() {
        let caps = bootstrap_caps(ConcurrencyCaps {
            rayon_threads: 0,
            tokio_worker_threads: DEFAULT_TOKIO_WORKER_THREADS,
        });

        assert_eq!(caps.rayon_threads, 1);
        assert_eq!(caps.tokio_worker_threads, DEFAULT_TOKIO_WORKER_THREADS);
    }
}
