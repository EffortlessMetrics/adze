use adze_concurrency_parse_core::parse_positive_usize_or_default;
use proptest::prelude::*;

fn model_parse(value: Option<&str>, default: usize) -> usize {
    value
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .filter(|parsed| *parsed > 0)
        .unwrap_or(default)
}

proptest! {
    #[test]
    fn parse_positive_values_round_trip(
        default in 0usize..1024,
        value in 1usize..1_000_000usize,
    ) {
        let raw = format!(" {value} ");
        prop_assert_eq!(parse_positive_usize_or_default(Some(&raw), default), value);
    }

    #[test]
    fn parse_zero_or_negative_like_inputs_fall_back_to_default(
        default in 0usize..1024,
        value in i64::MIN..=0,
    ) {
        let raw = value.to_string();
        prop_assert_eq!(parse_positive_usize_or_default(Some(&raw), default), default);
    }

    #[test]
    fn parse_unparseable_inputs_follow_model(
        default in 0usize..1024,
        bytes in prop::collection::vec(any::<u8>(), 1..64),
    ) {
        let raw = String::from_utf8_lossy(&bytes).to_string();
        let expected = model_parse(Some(&raw), default);
        prop_assert_eq!(parse_positive_usize_or_default(Some(&raw), default), expected);
    }
}
