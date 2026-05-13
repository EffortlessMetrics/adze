use adze::concurrency_caps::bounded::bounded_parallel_map as owner_bounded_parallel_map;
use adze::concurrency_caps::contract::bounded_parallel_map as caps_bounded_parallel_map;

type TransformFn = fn(i32) -> i32;
type MapFn = fn(Vec<i32>, usize, TransformFn) -> Vec<i32>;

fn model_transform(value: i32) -> i32 {
    value.wrapping_mul(17).wrapping_add(3)
}

#[test]
fn caps_core_reexport_matches_map_owner_module_for_multiset_outputs() {
    let input: Vec<i32> = (0..1024).collect();

    for concurrency in 0..=32 {
        let mut caps = caps_bounded_parallel_map(input.clone(), concurrency, model_transform);
        let mut owner = owner_bounded_parallel_map(input.clone(), concurrency, model_transform);

        caps.sort_unstable();
        owner.sort_unstable();
        assert_eq!(caps, owner, "concurrency={concurrency}");
    }
}

#[test]
fn caps_core_reexport_is_type_compatible_with_map_owner_module() {
    fn accepts_owner_fn(f: MapFn) -> MapFn {
        f
    }

    let returned = accepts_owner_fn(caps_bounded_parallel_map::<i32, i32, TransformFn>);
    let mut output = returned((0..64).collect(), 4, model_transform);
    output.sort_unstable();

    let mut expected = owner_bounded_parallel_map((0..64).collect(), 4, model_transform);
    expected.sort_unstable();
    assert_eq!(output, expected);
}
