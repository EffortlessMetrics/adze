use adze_concurrency_caps_contract_core::bounded_parallel_map as caps_bounded_parallel_map;
use adze_concurrency_map_core::bounded_parallel_map as core_bounded_parallel_map;

type TransformFn = fn(i32) -> i32;
type MapFn = fn(Vec<i32>, usize, TransformFn) -> Vec<i32>;

fn model_transform(value: i32) -> i32 {
    value.wrapping_mul(17).wrapping_add(3)
}

#[test]
fn caps_core_reexport_matches_map_core_for_multiset_outputs() {
    let input: Vec<i32> = (0..1024).collect();

    for concurrency in 0..=32 {
        let mut caps = caps_bounded_parallel_map(input.clone(), concurrency, model_transform);
        let mut core = core_bounded_parallel_map(input.clone(), concurrency, model_transform);

        caps.sort_unstable();
        core.sort_unstable();
        assert_eq!(caps, core, "concurrency={concurrency}");
    }
}

#[test]
fn caps_core_reexport_is_type_compatible_with_map_core() {
    fn accepts_core_fn(f: MapFn) -> MapFn {
        f
    }

    let returned = accepts_core_fn(caps_bounded_parallel_map::<i32, i32, TransformFn>);
    let mut output = returned((0..64).collect(), 4, model_transform);
    output.sort_unstable();

    let mut expected = core_bounded_parallel_map((0..64).collect(), 4, model_transform);
    expected.sort_unstable();
    assert_eq!(output, expected);
}
