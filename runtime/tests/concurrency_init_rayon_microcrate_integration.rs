use adze::concurrency_caps::init_rayon_global_once as runtime_init_rayon_global_once;
use adze_concurrency_init_rayon_core::init_rayon_global_once as core_init_rayon_global_once;

#[test]
fn runtime_reexport_matches_microcrate_init_behavior() {
    for threads in [0usize, 1, 2, 4, 8, 32, 128] {
        assert_eq!(
            runtime_init_rayon_global_once(threads),
            core_init_rayon_global_once(threads)
        );
    }
}

#[test]
fn runtime_reexport_stays_type_compatible() {
    fn accepts_core_fn(f: fn(usize) -> Result<(), String>) -> fn(usize) -> Result<(), String> {
        f
    }

    let returned = accepts_core_fn(runtime_init_rayon_global_once);
    assert_eq!(returned(4), core_init_rayon_global_once(4));
}
