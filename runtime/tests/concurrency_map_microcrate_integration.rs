use adze::concurrency_caps::bounded_parallel_map as runtime_bounded_parallel_map;
use adze_concurrency_map_core::bounded_parallel_map as core_bounded_parallel_map;

fn model_transform(value: i32) -> i32 {
    value.wrapping_mul(17).wrapping_add(3)
}

#[test]
fn runtime_reexport_matches_microcrate_behavior() {
    let input: Vec<i32> = (0..1024).collect();

    for concurrency in 0..=32 {
        let mut runtime = runtime_bounded_parallel_map(input.clone(), concurrency, model_transform);
        let mut core = core_bounded_parallel_map(input.clone(), concurrency, model_transform);
        runtime.sort_unstable();
        core.sort_unstable();
        assert_eq!(runtime, core, "concurrency={concurrency}");
    }
}

#[test]
fn runtime_reexport_stays_type_compatible() {
    type TransformFn = fn(i32) -> i32;
    type BoundedMapFn = fn(Vec<i32>, usize, TransformFn) -> Vec<i32>;

    fn accepts_core_fn(f: BoundedMapFn) -> BoundedMapFn {
        f
    }

    let returned = accepts_core_fn(runtime_bounded_parallel_map::<i32, i32, TransformFn>);
    let mut output = returned((0..64).collect(), 4, model_transform);
    output.sort_unstable();

    let mut expected = core_bounded_parallel_map((0..64).collect(), 4, model_transform);
    expected.sort_unstable();
    assert_eq!(output, expected);
}
