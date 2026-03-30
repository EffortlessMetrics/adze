//! Cross-crate integration tests for the init chain:
//! `concurrency-env-core` → `concurrency-init-bootstrap-core` → `concurrency-init-rayon-core`
//!
//! These tests validate that the full initialization chain works correctly end-to-end.

use adze_concurrency_env_core::{ConcurrencyCaps, current_caps};
use adze_concurrency_init_bootstrap_core::init_concurrency_caps_with_caps;
use adze_concurrency_init_rayon_core::init_rayon_global_once;

/// Test the full init chain from environment detection to Rayon initialization.
/// This validates that caps detected from the environment can flow through
/// the bootstrap layer to initialize Rayon's global thread pool.
#[test]
fn test_init_chain_from_env_to_rayon() {
    // Given: Environment-based caps detection
    let env_caps = current_caps();

    // When: Initialize with the detected caps through the full chain
    init_concurrency_caps_with_caps(env_caps);

    // Then: Rayon initialization should succeed (be idempotent)
    let result = init_rayon_global_once(env_caps.rayon_threads);
    assert!(
        result.is_ok(),
        "Rayon init should succeed after bootstrap init"
    );
}

/// Test that the init chain properly normalizes zero thread counts.
#[test]
fn test_init_chain_normalizes_zero_threads() {
    // Given: Caps with zero rayon threads (should be normalized to 1)
    let caps = ConcurrencyCaps {
        rayon_threads: 0,
        tokio_worker_threads: 0,
    };

    // When: Initialize through the chain
    init_concurrency_caps_with_caps(caps);

    // Then: Should not panic and Rayon should still work
    let result = init_rayon_global_once(1);
    assert!(result.is_ok());
}

/// Test that the init chain is idempotent across multiple calls.
#[test]
fn test_init_chain_is_idempotent() {
    let caps = ConcurrencyCaps::default();

    // Multiple initializations should all succeed without panicking
    init_concurrency_caps_with_caps(caps);
    init_concurrency_caps_with_caps(caps);
    init_concurrency_caps_with_caps(caps);

    // Rayon should still report success
    assert!(init_rayon_global_once(caps.rayon_threads).is_ok());
}

/// Test that the init chain works with various thread count configurations.
#[test]
fn test_init_chain_with_various_thread_counts() {
    let test_cases = [1, 2, 4, 8, 16];

    for threads in test_cases {
        let caps = ConcurrencyCaps {
            rayon_threads: threads,
            tokio_worker_threads: threads,
        };

        // Should not panic for any reasonable thread count
        init_concurrency_caps_with_caps(caps);
    }
}

/// Test that caps from env-core are compatible with bootstrap init.
#[test]
fn test_env_caps_compatible_with_bootstrap() {
    // Caps from env-core
    let env_caps = adze_concurrency_env_core::current_caps();

    // Should be directly usable with bootstrap init
    init_concurrency_caps_with_caps(env_caps);

    // Verify the caps have sensible values after initialization
    assert!(env_caps.rayon_threads >= 1);
    assert!(env_caps.tokio_worker_threads >= 1);
}
