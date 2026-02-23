use adze_concurrency_map_core::normalized_concurrency as map_core_normalized_concurrency;
use adze_concurrency_normalize_core::normalized_concurrency as normalize_core_normalized_concurrency;

#[test]
fn map_core_reexport_matches_normalize_core_behavior() {
    for value in [0, 1, 2, 8, 64, usize::MAX] {
        assert_eq!(
            map_core_normalized_concurrency(value),
            normalize_core_normalized_concurrency(value)
        );
    }
}

#[test]
fn map_core_reexport_is_type_compatible_with_normalize_core() {
    fn accepts_normalize_fn(f: fn(usize) -> usize) -> fn(usize) -> usize {
        f
    }

    let returned = accepts_normalize_fn(map_core_normalized_concurrency);
    assert_eq!(returned(0), normalize_core_normalized_concurrency(0));
}
