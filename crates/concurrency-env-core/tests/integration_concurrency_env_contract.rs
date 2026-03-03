use adze_concurrency_env_contract_core as contract_core;
use adze_concurrency_env_core as env_core;

#[test]
fn env_core_reexports_contract_items() {
    fn accepts_contract_type(
        value: contract_core::ConcurrencyCaps,
    ) -> contract_core::ConcurrencyCaps {
        value
    }

    let env_caps = env_core::current_caps();
    let contract_caps = contract_core::current_caps();
    assert_eq!(env_caps, contract_caps);
    let returned = accepts_contract_type(env_caps);
    assert_eq!(returned, env_caps);
}

#[test]
fn env_core_parse_behavior_matches_contract_core() {
    let env_parse = env_core::parse_positive_usize_or_default as fn(Option<&str>, usize) -> usize;
    let contract_parse =
        contract_core::parse_positive_usize_or_default as fn(Option<&str>, usize) -> usize;

    assert_eq!(env_parse(Some(" 17 "), 5), contract_parse(Some(" 17 "), 5));
    assert_eq!(env_parse(None, 9), contract_parse(None, 9));
}
