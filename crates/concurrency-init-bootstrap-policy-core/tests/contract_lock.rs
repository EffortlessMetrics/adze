//! Contract lock test - verifies that public API remains stable.

use adze_concurrency_env_core::ConcurrencyCaps;
use adze_concurrency_init_bootstrap_policy_core::bootstrap_caps;

/// Verify all public functions exist with expected signatures.
#[test]
fn test_contract_lock_functions() {
    // Verify bootstrap_caps function exists and returns ConcurrencyCaps
    let caps = ConcurrencyCaps {
        rayon_threads: 4,
        tokio_worker_threads: 2,
    };

    let result = bootstrap_caps(caps);

    // Verify return type is ConcurrencyCaps with expected fields
    assert_eq!(result.rayon_threads, 4);
    assert_eq!(result.tokio_worker_threads, 2);

    // Verify function signature via function pointer
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

    // Zero rayon_threads should be normalized to 1
    assert_eq!(result.rayon_threads, 1);
    assert_eq!(result.tokio_worker_threads, 2);
}
