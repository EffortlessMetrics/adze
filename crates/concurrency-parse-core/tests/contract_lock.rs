use adze_concurrency_parse_core::parse_positive_usize_or_default;

#[test]
fn contract_zero_and_invalid_values_use_default() {
    assert_eq!(parse_positive_usize_or_default(Some("0"), 5), 5);
    assert_eq!(parse_positive_usize_or_default(Some("invalid"), 5), 5);
}

#[test]
fn contract_trimmed_positive_values_are_accepted() {
    assert_eq!(parse_positive_usize_or_default(Some(" 99 "), 5), 99);
}
