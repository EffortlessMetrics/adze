//! Contract lock test - verifies that public API remains stable.

use adze_concurrency_init_bootstrap_core::{
    ConcurrencyCaps, DEFAULT_RAYON_NUM_THREADS, DEFAULT_TOKIO_WORKER_THREADS,
    RAYON_NUM_THREADS_ENV, TOKIO_WORKER_THREADS_ENV, init_concurrency_caps,
    init_concurrency_caps_with_caps,
};

/// Verify all public types exist and have expected structure.
#[test]
fn test_contract_lock_types() {
    // Verify ConcurrencyCaps struct exists with expected fields
    let caps = ConcurrencyCaps {
        rayon_threads: 4,
        tokio_worker_threads: 2,
    };

    // Verify fields are accessible
    assert_eq!(caps.rayon_threads, 4);
    assert_eq!(caps.tokio_worker_threads, 2);
}

/// Verify all public constants exist with expected values.
#[test]
fn test_contract_lock_constants() {
    // Verify DEFAULT_RAYON_NUM_THREADS constant exists
    assert_eq!(DEFAULT_RAYON_NUM_THREADS, 4);

    // Verify DEFAULT_TOKIO_WORKER_THREADS constant exists
    assert_eq!(DEFAULT_TOKIO_WORKER_THREADS, 2);

    // Verify RAYON_NUM_THREADS_ENV constant exists
    assert_eq!(RAYON_NUM_THREADS_ENV, "RAYON_NUM_THREADS");

    // Verify TOKIO_WORKER_THREADS_ENV constant exists
    assert_eq!(TOKIO_WORKER_THREADS_ENV, "TOKIO_WORKER_THREADS");
}

/// Verify all public functions exist with expected signatures.
#[test]
fn test_contract_lock_functions() {
    // Verify init_concurrency_caps function exists
    let _fn_ptr: Option<fn()> = Some(init_concurrency_caps);

    // Verify init_concurrency_caps_with_caps function exists
    let _fn_ptr: Option<fn(ConcurrencyCaps)> = Some(init_concurrency_caps_with_caps);
}
