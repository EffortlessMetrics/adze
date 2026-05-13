use adze::concurrency_caps::contract::parse_positive_usize_or_default as caps_parse;
use adze::concurrency_caps::env::parse_positive_usize_or_default as parse_owner_parse;

#[test]
fn caps_core_reexport_matches_parse_owner_behavior() {
    for default in 0usize..=64 {
        for value in [
            None,
            Some(""),
            Some("0"),
            Some(" 1 "),
            Some("42"),
            Some("invalid"),
        ] {
            assert_eq!(
                caps_parse(value, default),
                parse_owner_parse(value, default)
            );
        }
    }
}

#[test]
fn caps_core_reexport_is_type_compatible_with_parse_owner() {
    fn accepts_core_fn(f: fn(Option<&str>, usize) -> usize) -> fn(Option<&str>, usize) -> usize {
        f
    }

    let returned = accepts_core_fn(caps_parse);
    assert_eq!(
        returned(Some(" 17 "), 5),
        parse_owner_parse(Some(" 17 "), 5)
    );
}
