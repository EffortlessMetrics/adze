use adze::concurrency_caps::normalized_concurrency as runtime_normalized_concurrency;
use adze_concurrency_normalize_core::normalized_concurrency as normalize_core_normalized_concurrency;

#[test]
fn runtime_reexport_matches_normalize_microcrate_behavior() {
    for value in [0, 1, 2, 8, 64, usize::MAX] {
        assert_eq!(
            runtime_normalized_concurrency(value),
            normalize_core_normalized_concurrency(value)
        );
    }
}

#[test]
fn runtime_reexport_stays_type_compatible() {
    fn accepts_normalize_fn(f: fn(usize) -> usize) -> fn(usize) -> usize {
        f
    }

    let returned = accepts_normalize_fn(runtime_normalized_concurrency);
    assert_eq!(returned(0), normalize_core_normalized_concurrency(0));
}
