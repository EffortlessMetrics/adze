use adze_concurrency_env_contract_core::parse_positive_usize_or_default as contract_parse;
use adze_concurrency_parse_core::parse_positive_usize_or_default as parse_core_parse;

#[test]
fn parse_adapter_matches_parse_core() {
    for default in 0usize..=64 {
        for raw in [
            None,
            Some(""),
            Some("0"),
            Some(" 1 "),
            Some("42"),
            Some("invalid"),
        ] {
            assert_eq!(contract_parse(raw, default), parse_core_parse(raw, default));
        }
    }
}
