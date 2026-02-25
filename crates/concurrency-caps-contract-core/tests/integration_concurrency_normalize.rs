use adze_concurrency_caps_contract_core::normalized_concurrency as caps_core_normalized_concurrency;
use adze_concurrency_normalize_core::normalized_concurrency as normalize_core_normalized_concurrency;

#[test]
fn caps_core_reexport_matches_normalize_core_behavior() {
    for value in [0, 1, 2, 8, 64, usize::MAX] {
        assert_eq!(
            caps_core_normalized_concurrency(value),
            normalize_core_normalized_concurrency(value)
        );
    }
}

#[test]
fn caps_core_reexport_is_type_compatible_with_normalize_core() {
    fn accepts_normalize_fn(f: fn(usize) -> usize) -> fn(usize) -> usize {
        f
    }

    let returned = accepts_normalize_fn(caps_core_normalized_concurrency);
    assert_eq!(returned(0), normalize_core_normalized_concurrency(0));
}
