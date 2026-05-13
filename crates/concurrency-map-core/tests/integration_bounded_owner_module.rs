use adze_concurrency_map_core::bounded::bounded_parallel_map as bounded_owner_map;
use adze_concurrency_map_core::bounded_parallel_map as facade_bounded_map;

type TransformFn = fn(i32) -> i32;
type MapFn = fn(Vec<i32>, usize, TransformFn) -> Vec<i32>;

#[test]
fn map_core_facade_matches_bounded_owner_module_for_multiset_outputs() {
    let input: Vec<i32> = (0..1024).collect();

    for concurrency in 0..=32 {
        let mut bounded = bounded_owner_map(input.clone(), concurrency, |value| value * 3 + 1);
        let mut facade = facade_bounded_map(input.clone(), concurrency, |value| value * 3 + 1);

        bounded.sort_unstable();
        facade.sort_unstable();
        assert_eq!(bounded, facade, "concurrency={concurrency}");
    }
}

#[test]
fn map_core_type_compatible_with_bounded_owner_module() {
    fn accepts_bounded_map_fn(f: MapFn) -> MapFn {
        f
    }

    let returned = accepts_bounded_map_fn(bounded_owner_map);
    let mut output = returned((0..64).collect(), 4, |value| value * 2);
    let mut expected = facade_bounded_map((0..64).collect(), 4, |value| value * 2);
    output.sort_unstable();
    expected.sort_unstable();
    assert_eq!(output, expected);
}
