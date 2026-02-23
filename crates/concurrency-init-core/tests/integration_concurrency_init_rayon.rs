use adze_concurrency_init_core::init_rayon_global_once as init_core_fn;
use adze_concurrency_init_rayon_core::init_rayon_global_once as rayon_core_fn;

#[test]
fn init_core_reexport_matches_rayon_core_behavior() {
    for threads in [0usize, 1, 2, 4, 8, 32, 128] {
        assert_eq!(init_core_fn(threads), rayon_core_fn(threads));
    }
}

#[test]
fn init_core_reexport_is_type_compatible_with_rayon_core() {
    fn accepts_core_fn(f: fn(usize) -> Result<(), String>) -> fn(usize) -> Result<(), String> {
        f
    }

    let returned = accepts_core_fn(init_core_fn);
    assert_eq!(returned(4), rayon_core_fn(4));
}
