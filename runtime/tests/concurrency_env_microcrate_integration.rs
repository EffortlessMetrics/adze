use adze::concurrency_caps::{
    ConcurrencyCaps as RuntimeConcurrencyCaps, current_caps as runtime_current_caps,
};
use adze_concurrency_env_core::{
    ConcurrencyCaps as CoreConcurrencyCaps, current_caps as core_current_caps,
};

#[test]
fn runtime_reexport_matches_microcrate_defaults() {
    assert_eq!(
        RuntimeConcurrencyCaps::default(),
        CoreConcurrencyCaps::default()
    );
}

#[test]
fn runtime_reexport_matches_microcrate_current_caps() {
    assert_eq!(runtime_current_caps(), core_current_caps());
}

#[test]
fn runtime_reexport_stays_type_compatible() {
    fn accepts_core_type(value: CoreConcurrencyCaps) -> CoreConcurrencyCaps {
        value
    }

    let runtime_value = RuntimeConcurrencyCaps::default();
    let returned = accepts_core_type(runtime_value);
    assert_eq!(returned, CoreConcurrencyCaps::default());
}
