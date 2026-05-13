//! Single-responsibility helpers for normalized concurrency bounds.

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
