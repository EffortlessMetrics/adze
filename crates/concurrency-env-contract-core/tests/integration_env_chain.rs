//! Cross-crate integration tests for the env chain:
//! `concurrency-parse-core` → `concurrency-env-contract-core` → `concurrency-env-core`
//!
//! These tests validate that the environment configuration chain works correctly end-to-end.

use adze_concurrency_env_contract_core::{
    ConcurrencyCaps, DEFAULT_RAYON_NUM_THREADS, DEFAULT_TOKIO_WORKER_THREADS,
    RAYON_NUM_THREADS_ENV, TOKIO_WORKER_THREADS_ENV, current_caps, parse_positive_usize_or_default,
};
use adze_concurrency_env_core as env_core;
use adze_concurrency_parse_core as parse_core;

/// Test that the env chain correctly parses and returns default values.
#[test]
fn test_env_chain_defaults_are_consistent() {
    // Given: No environment overrides
    let contract_caps = ConcurrencyCaps::from_lookup(|_| None);
    let env_caps = env_core::ConcurrencyCaps::from_lookup(|_| None);

    // Then: Both should return the same defaults
    assert_eq!(contract_caps, env_caps);
    assert_eq!(contract_caps.rayon_threads, DEFAULT_RAYON_NUM_THREADS);
    assert_eq!(
        contract_caps.tokio_worker_threads,
        DEFAULT_TOKIO_WORKER_THREADS
    );
}

/// Test that parse_core functions are correctly re-exported through the chain.
#[test]
fn test_env_chain_parse_function_reexport() {
    // The parse function should work identically at both levels
    assert_eq!(
        parse_positive_usize_or_default(Some("42"), 10),
        parse_core::parse_positive_usize_or_default(Some("42"), 10)
    );
    assert_eq!(
        parse_positive_usize_or_default(None, 10),
        parse_core::parse_positive_usize_or_default(None, 10)
    );
    assert_eq!(
        parse_positive_usize_or_default(Some("0"), 10),
        parse_core::parse_positive_usize_or_default(Some("0"), 10)
    );
}

/// Test that the env chain handles custom lookup functions correctly.
#[test]
fn test_env_chain_custom_lookup() {
    let caps = ConcurrencyCaps::from_lookup(|name| match name {
        "RAYON_NUM_THREADS" => Some("8".to_string()),
        "TOKIO_WORKER_THREADS" => Some("4".to_string()),
        _ => None,
    });

    assert_eq!(caps.rayon_threads, 8);
    assert_eq!(caps.tokio_worker_threads, 4);
}

/// Test that env-core types are compatible with env-contract-core types.
#[test]
fn test_env_chain_type_compatibility() {
    fn accepts_contract_caps(caps: ConcurrencyCaps) -> ConcurrencyCaps {
        caps
    }

    // env-core re-exports from env-contract-core, so types should be identical
    let env_caps = env_core::ConcurrencyCaps::default();
    let returned = accepts_contract_caps(env_caps);
    assert_eq!(returned, ConcurrencyCaps::default());
}

/// Test that the environment variable constants are consistent across the chain.
#[test]
fn test_env_chain_constants_are_consistent() {
    assert_eq!(RAYON_NUM_THREADS_ENV, env_core::RAYON_NUM_THREADS_ENV);
    assert_eq!(TOKIO_WORKER_THREADS_ENV, env_core::TOKIO_WORKER_THREADS_ENV);
    assert_eq!(
        DEFAULT_RAYON_NUM_THREADS,
        env_core::DEFAULT_RAYON_NUM_THREADS
    );
    assert_eq!(
        DEFAULT_TOKIO_WORKER_THREADS,
        env_core::DEFAULT_TOKIO_WORKER_THREADS
    );
}

/// Test that current_caps returns valid values.
#[test]
fn test_env_chain_current_caps_returns_valid_values() {
    let caps = current_caps();

    // Should always return at least the minimum valid values
    assert!(caps.rayon_threads >= 1);
    assert!(caps.tokio_worker_threads >= 1);
}

/// Test that the chain handles invalid input gracefully.
#[test]
fn test_env_chain_handles_invalid_input() {
    let caps = ConcurrencyCaps::from_lookup(|name| match name {
        "RAYON_NUM_THREADS" => Some("invalid".to_string()),
        "TOKIO_WORKER_THREADS" => Some("-5".to_string()),
        _ => None,
    });

    // Should fall back to defaults for invalid input
    assert_eq!(caps.rayon_threads, DEFAULT_RAYON_NUM_THREADS);
    assert_eq!(caps.tokio_worker_threads, DEFAULT_TOKIO_WORKER_THREADS);
}
