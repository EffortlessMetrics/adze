use adze_concurrency_caps_contract_core::{
    ConcurrencyCaps as CapsConcurrencyCaps, current_caps as caps_current_caps,
};
use adze_concurrency_env_core::{
    ConcurrencyCaps as EnvConcurrencyCaps, current_caps as env_current_caps,
};

#[test]
fn caps_core_reexport_matches_env_core_defaults() {
    assert_eq!(
        CapsConcurrencyCaps::default(),
        EnvConcurrencyCaps::default()
    );
}

#[test]
fn caps_core_reexport_matches_env_core_current_caps() {
    assert_eq!(caps_current_caps(), env_current_caps());
}

#[test]
fn caps_core_reexport_is_type_compatible_with_env_core() {
    fn accepts_env_type(value: EnvConcurrencyCaps) -> EnvConcurrencyCaps {
        value
    }

    let runtime_value = CapsConcurrencyCaps::default();
    let returned = accepts_env_type(runtime_value);
    assert_eq!(returned, EnvConcurrencyCaps::default());
}
