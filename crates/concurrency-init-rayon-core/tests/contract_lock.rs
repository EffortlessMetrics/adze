use adze_concurrency_init_rayon_core::init_rayon_global_once;

#[test]
fn contract_repeated_initialization_returns_ok() {
    assert!(init_rayon_global_once(1).is_ok());
    assert!(init_rayon_global_once(4).is_ok());
}

#[test]
fn contract_first_result_is_stable_for_subsequent_calls() {
    let first = init_rayon_global_once(2);
    let second = init_rayon_global_once(32);

    assert_eq!(first, second);
    assert!(first.is_ok());
}
