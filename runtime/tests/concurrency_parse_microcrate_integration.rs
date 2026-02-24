use adze::concurrency_caps::parse_positive_usize_or_default as runtime_parse;
use adze_concurrency_parse_core::parse_positive_usize_or_default as parse_core_parse;

#[test]
fn runtime_reexport_matches_microcrate_behavior() {
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
                runtime_parse(value, default),
                parse_core_parse(value, default)
            );
        }
    }
}

#[test]
fn runtime_reexport_stays_type_compatible() {
    fn accepts_core_fn(f: fn(Option<&str>, usize) -> usize) -> fn(Option<&str>, usize) -> usize {
        f
    }

    let returned = accepts_core_fn(runtime_parse);
    assert_eq!(returned(Some(" 17 "), 5), parse_core_parse(Some(" 17 "), 5));
}
