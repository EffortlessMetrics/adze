use adze_concurrency_caps_core::init_concurrency_caps as caps_init_concurrency_caps;
use adze_concurrency_init_core::init_concurrency_caps as init_core_init_concurrency_caps;

#[test]
fn caps_core_reexport_matches_init_core_behavior() {
    caps_init_concurrency_caps();
    init_core_init_concurrency_caps();
    caps_init_concurrency_caps();
}

#[test]
fn caps_core_reexport_is_type_compatible_with_init_core() {
    fn accepts_core_fn(f: fn()) -> fn() {
        f
    }

    let returned = accepts_core_fn(caps_init_concurrency_caps);
    returned();
    init_core_init_concurrency_caps();
}
