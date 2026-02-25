use adze_concurrency_bounded_map_core::normalized_concurrency as bounded_map_core_normalized_concurrency;
use adze_concurrency_normalize_core::normalized_concurrency as normalize_core_normalized_concurrency;

#[test]
fn bounded_map_core_reexport_matches_normalize_core_behavior() {
    for value in [0, 1, 2, 8, 64, usize::MAX] {
        assert_eq!(
            bounded_map_core_normalized_concurrency(value),
            normalize_core_normalized_concurrency(value)
        );
    }
}

#[test]
fn bounded_map_core_normalized_is_type_compatible_with_normalize_core() {
    fn accepts_fn(f: fn(usize) -> usize) -> fn(usize) -> usize {
        f
    }

    let returned = accepts_fn(bounded_map_core_normalized_concurrency);
    assert_eq!(returned(0), normalize_core_normalized_concurrency(0));
}
