//! String parsing helpers for concurrency cap configuration values.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

/// Parse an optional positive integer value, falling back to `default`.
///
/// `None`, parse failures, and `0` all resolve to `default`.
#[must_use]
pub fn parse_positive_usize_or_default(value: Option<&str>, default: usize) -> usize {
    value
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .filter(|parsed| *parsed > 0)
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::parse_positive_usize_or_default;

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
}
