//! Cross-crate integration tests for the caps chain:
//! `concurrency-caps-core` → `concurrency-caps-contract-core`
//!
//! These tests validate that the caps aggregation layer correctly integrates
//! all the underlying concurrency utilities.

use adze_concurrency_caps_contract_core as caps_contract;
use adze_concurrency_caps_core as caps_core;

/// Test that caps-core and caps-contract-core expose the same ConcurrencyCaps type.
#[test]
fn test_caps_chain_type_consistency() {
    let core_caps = caps_core::ConcurrencyCaps::default();
    let contract_caps = caps_contract::ConcurrencyCaps::default();

    assert_eq!(core_caps, contract_caps);
}

/// Test that the init functions are consistent across the caps chain.
#[test]
fn test_caps_chain_init_consistency() {
    // Both should provide init_concurrency_caps
    caps_core::init_concurrency_caps();
    caps_contract::init_concurrency_caps();

    // Both should provide init_rayon_global_once
    assert!(caps_core::init_rayon_global_once(1).is_ok());
    assert!(caps_contract::init_rayon_global_once(1).is_ok());
}

/// Test that bounded_parallel_map works identically across the caps chain.
#[test]
fn test_caps_chain_bounded_map_consistency() {
    let input: Vec<i32> = (0..64).collect();
    let transform = |x: i32| x.wrapping_mul(3).wrapping_add(7);

    for concurrency in 0..=8 {
        let mut core_result =
            caps_core::bounded_parallel_map(input.clone(), concurrency, transform);
        let mut contract_result =
            caps_contract::bounded_parallel_map(input.clone(), concurrency, transform);

        core_result.sort_unstable();
        contract_result.sort_unstable();

        assert_eq!(
            core_result, contract_result,
            "Mismatch at concurrency={concurrency}"
        );
    }
}

/// Test that normalized_concurrency is consistent across the caps chain.
#[test]
fn test_caps_chain_normalized_concurrency_consistency() {
    for value in [0, 1, 2, 4, 8, 16, 64, 256, usize::MAX] {
        assert_eq!(
            caps_core::normalized_concurrency(value),
            caps_contract::normalized_concurrency(value),
            "Mismatch at value={value}"
        );
    }
}

/// Test that ParallelPartitionPlan is consistent across the caps chain.
#[test]
fn test_caps_chain_partition_plan_consistency() {
    for item_count in [0, 1, 10, 100, 1000] {
        for concurrency in [0, 1, 2, 4, 8] {
            let core_plan =
                caps_core::ParallelPartitionPlan::for_item_count(item_count, concurrency);
            let contract_plan =
                caps_contract::ParallelPartitionPlan::for_item_count(item_count, concurrency);

            assert_eq!(
                core_plan, contract_plan,
                "Mismatch at items={item_count}, concurrency={concurrency}"
            );
        }
    }
}

/// Test that the caps chain correctly aggregates env functionality.
#[test]
fn test_caps_chain_env_aggregation() {
    // caps-core should expose env-core functionality
    let core_caps = caps_core::current_caps();
    let contract_caps = caps_contract::current_caps();

    assert_eq!(core_caps, contract_caps);

    // Both should have valid values
    assert!(core_caps.rayon_threads >= 1);
    assert!(core_caps.tokio_worker_threads >= 1);
}

/// Test that the caps chain correctly aggregates parse functionality.
#[test]
fn test_caps_chain_parse_aggregation() {
    for input in [None, Some("0"), Some("1"), Some("42"), Some("invalid")] {
        assert_eq!(
            caps_core::parse_positive_usize_or_default(input, 10),
            caps_contract::parse_positive_usize_or_default(input, 10),
            "Mismatch at input={input:?}"
        );
    }
}

/// Test that the caps chain constants are consistent.
#[test]
fn test_caps_chain_constants_consistency() {
    assert_eq!(
        caps_core::DEFAULT_RAYON_NUM_THREADS,
        caps_contract::DEFAULT_RAYON_NUM_THREADS
    );
    assert_eq!(
        caps_core::DEFAULT_TOKIO_WORKER_THREADS,
        caps_contract::DEFAULT_TOKIO_WORKER_THREADS
    );
    assert_eq!(
        caps_core::RAYON_NUM_THREADS_ENV,
        caps_contract::RAYON_NUM_THREADS_ENV
    );
    assert_eq!(
        caps_core::TOKIO_WORKER_THREADS_ENV,
        caps_contract::TOKIO_WORKER_THREADS_ENV
    );
}

/// Test that is_already_initialized_error is consistent.
#[test]
fn test_caps_chain_error_classification_consistency() {
    let test_messages = [
        "The global thread pool has already been initialized",
        "thread pool already initialized",
        "global thread pool initialized",
        "some other error",
        "",
    ];

    for msg in test_messages {
        assert_eq!(
            caps_core::is_already_initialized_error(msg),
            caps_contract::is_already_initialized_error(msg),
            "Mismatch at message={msg:?}"
        );
    }
}

/// Test the full caps chain end-to-end: env → init → map.
#[test]
fn test_caps_chain_full_integration() {
    // 1. Get caps from environment
    let caps = caps_core::current_caps();

    // 2. Initialize with those caps
    caps_core::init_concurrency_caps();

    // 3. Use bounded_parallel_map with normalized concurrency
    let input: Vec<i32> = (0..100).collect();
    let concurrency = caps_core::normalized_concurrency(caps.rayon_threads);
    let result = caps_core::bounded_parallel_map(input, concurrency, |x| x * 2);

    // 4. Verify the result
    assert_eq!(result.len(), 100);
}
