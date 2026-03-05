use adze::error_recovery::{
    ErrorRecoveryConfigBuilder as RuntimeBuilder, ErrorRecoveryState as RuntimeState,
    RecoveryStrategy as RuntimeStrategy,
};
use adze_error_recovery_core::{
    ErrorRecoveryConfigBuilder as CoreBuilder, ErrorRecoveryState as CoreState,
    RecoveryStrategy as CoreStrategy,
};

#[test]
fn runtime_reexport_matches_microcrate_strategy_behavior() {
    let runtime_config = RuntimeBuilder::new().add_insertable_token(7).build();
    let core_config = CoreBuilder::new().add_insertable_token(7).build();

    let mut runtime_state = RuntimeState::new(runtime_config);
    let mut core_state = CoreState::new(core_config);

    let runtime = runtime_state.determine_recovery_strategy(&[7], None, (0, 0), 0);
    let core = core_state.determine_recovery_strategy(&[7], None, (0, 0), 0);

    assert_eq!(runtime, RuntimeStrategy::TokenInsertion);
    assert_eq!(core, CoreStrategy::TokenInsertion);
}
