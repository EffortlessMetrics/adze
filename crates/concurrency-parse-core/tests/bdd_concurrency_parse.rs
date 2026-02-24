use adze_concurrency_parse_core::parse_positive_usize_or_default;

#[test]
fn given_missing_value_when_parsing_then_default_is_returned() {
    // Given / When
    let parsed = parse_positive_usize_or_default(None, 7);

    // Then
    assert_eq!(parsed, 7);
}

#[test]
fn given_trimmed_positive_value_when_parsing_then_value_is_returned() {
    // Given / When
    let parsed = parse_positive_usize_or_default(Some(" 42 "), 7);

    // Then
    assert_eq!(parsed, 42);
}

#[test]
fn given_zero_or_invalid_value_when_parsing_then_default_is_returned() {
    // Given / When
    let zero = parse_positive_usize_or_default(Some("0"), 7);
    let invalid = parse_positive_usize_or_default(Some("invalid"), 7);

    // Then
    assert_eq!(zero, 7);
    assert_eq!(invalid, 7);
}
