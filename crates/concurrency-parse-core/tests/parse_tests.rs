use adze_concurrency_parse_core::parse_positive_usize_or_default;

#[test]
fn none_returns_default() {
    assert_eq!(parse_positive_usize_or_default(None, 4), 4);
}

#[test]
fn empty_string_returns_default() {
    assert_eq!(parse_positive_usize_or_default(Some(""), 7), 7);
}

#[test]
fn non_numeric_returns_default() {
    assert_eq!(parse_positive_usize_or_default(Some("abc"), 5), 5);
}

#[test]
fn zero_returns_default() {
    assert_eq!(parse_positive_usize_or_default(Some("0"), 3), 3);
}

#[test]
fn positive_value_is_parsed() {
    assert_eq!(parse_positive_usize_or_default(Some("42"), 1), 42);
}

#[test]
fn whitespace_is_trimmed() {
    assert_eq!(parse_positive_usize_or_default(Some("  16  "), 1), 16);
}

#[test]
fn negative_value_returns_default() {
    assert_eq!(parse_positive_usize_or_default(Some("-1"), 10), 10);
}

#[test]
fn large_value_is_parsed() {
    assert_eq!(parse_positive_usize_or_default(Some("999999"), 1), 999999);
}
