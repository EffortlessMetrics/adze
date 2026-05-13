//! String parsing helpers for concurrency cap configuration values.

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
