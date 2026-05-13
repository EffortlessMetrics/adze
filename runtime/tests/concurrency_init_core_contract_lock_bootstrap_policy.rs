//! Contract lock test - verifies that bootstrap policy API remains stable.

use adze::concurrency_caps::init::bootstrap_caps;
use adze_concurrency_env_contract_core::ConcurrencyCaps;

/// Verify all public functions exist with expected signatures.
#[test]
fn test_contract_lock_functions() {
    let caps = ConcurrencyCaps {
        rayon_threads: 4,
        tokio_worker_threads: 2,
    };

    let result = bootstrap_caps(caps);

    assert_eq!(result.rayon_threads, 4);
    assert_eq!(result.tokio_worker_threads, 2);

    let _fn_ptr: Option<fn(ConcurrencyCaps) -> ConcurrencyCaps> = Some(bootstrap_caps);
}

/// Verify function behavior with edge cases.
#[test]
fn test_contract_lock_bootstrap_caps_normalizes_zero() {
    let caps = ConcurrencyCaps {
        rayon_threads: 0,
        tokio_worker_threads: 2,
    };

    let result = bootstrap_caps(caps);

    assert_eq!(result.rayon_threads, 1);
    assert_eq!(result.tokio_worker_threads, 2);
}
