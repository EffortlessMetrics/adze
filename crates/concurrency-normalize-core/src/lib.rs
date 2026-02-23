//! Single-responsibility helpers for normalized concurrency bounds.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

/// Minimum valid concurrency value.
pub const MIN_CONCURRENCY: usize = 1;

/// Normalize a requested concurrency value.
///
/// A value of `0` is treated as `1` to avoid invalid worker counts.
#[must_use]
pub const fn normalized_concurrency(concurrency: usize) -> usize {
    if concurrency == 0 {
        MIN_CONCURRENCY
    } else {
        concurrency
    }
}

#[cfg(test)]
mod tests {
    use super::{MIN_CONCURRENCY, normalized_concurrency};

    #[test]
    fn minimum_concurrency_is_stable() {
        assert_eq!(MIN_CONCURRENCY, 1);
    }

    #[test]
    fn normalized_concurrency_is_never_zero() {
        assert_eq!(normalized_concurrency(0), 1);
        assert_eq!(normalized_concurrency(1), 1);
        assert_eq!(normalized_concurrency(8), 8);
    }
}
