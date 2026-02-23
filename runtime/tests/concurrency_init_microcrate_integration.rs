use adze::concurrency_caps::init_concurrency_caps as runtime_init_concurrency_caps;
use adze_concurrency_init_core::init_concurrency_caps as core_init_concurrency_caps;

#[test]
fn runtime_reexport_matches_microcrate_init_behavior() {
    runtime_init_concurrency_caps();
    core_init_concurrency_caps();
    runtime_init_concurrency_caps();
}

#[test]
fn runtime_reexport_stays_type_compatible() {
    fn accepts_core_fn(f: fn()) -> fn() {
        f
    }

    let returned = accepts_core_fn(runtime_init_concurrency_caps);
    returned();
    core_init_concurrency_caps();
}
