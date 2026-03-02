//! Pure policy helpers for resolving concurrency caps from key/value lookups.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_concurrency_parse_core::parse_positive_usize_or_default;

/// Environment variable used for Rayon global thread-pool caps.
pub const RAYON_NUM_THREADS_ENV: &str = "RAYON_NUM_THREADS";

/// Environment variable used for Tokio worker-thread caps.
pub const TOKIO_WORKER_THREADS_ENV: &str = "TOKIO_WORKER_THREADS";

/// Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid.
pub const DEFAULT_RAYON_NUM_THREADS: usize = 4;

/// Default worker count used for Tokio when `TOKIO_WORKER_THREADS` is unset/invalid.
pub const DEFAULT_TOKIO_WORKER_THREADS: usize = 2;

/// Resolve `(rayon_threads, tokio_worker_threads)` from a generic lookup function.
#[must_use]
pub fn resolve_caps_from_lookup<F>(mut lookup: F) -> (usize, usize)
where
    F: FnMut(&str) -> Option<String>,
{
    (
        parse_positive_usize_or_default(
            lookup(RAYON_NUM_THREADS_ENV).as_deref(),
            DEFAULT_RAYON_NUM_THREADS,
        ),
        parse_positive_usize_or_default(
            lookup(TOKIO_WORKER_THREADS_ENV).as_deref(),
            DEFAULT_TOKIO_WORKER_THREADS,
        ),
    )
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
    fn resolve_caps_from_lookup_uses_defaults_when_unset() {
        let (rayon, tokio) = resolve_caps_from_lookup(|_| None);
        assert_eq!(rayon, DEFAULT_RAYON_NUM_THREADS);
        assert_eq!(tokio, DEFAULT_TOKIO_WORKER_THREADS);
    }
}
