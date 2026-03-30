//! Contract lock test - verifies that public API remains stable.

use adze_stack_pool_core::{PoolStats, StackPool, get_thread_local_pool, init_thread_local_pool};
use std::rc::Rc;

/// Verify all public types exist and have expected structure.
#[test]
fn test_contract_lock_types() {
    // Verify StackPool struct exists
    let pool: StackPool<u32> = StackPool::new(4);

    // Verify Debug trait is implemented
    let _debug = format!("{pool:?}");

    // Verify PoolStats struct exists with expected fields
    let stats = PoolStats {
        total_allocations: 0,
        reuse_count: 0,
        pool_hits: 0,
        pool_misses: 0,
        max_pool_depth: 0,
    };

    // Verify fields are accessible
    assert_eq!(stats.total_allocations, 0);
    assert_eq!(stats.reuse_count, 0);
    assert_eq!(stats.pool_hits, 0);
    assert_eq!(stats.pool_misses, 0);
    assert_eq!(stats.max_pool_depth, 0);

    // Verify Debug trait is implemented for PoolStats
    let _debug_stats = format!("{stats:?}");

    // Verify Default trait is implemented for PoolStats
    let default_stats = PoolStats::default();
    assert_eq!(default_stats.total_allocations, 0);

    // Verify Clone trait is implemented for PoolStats
    let _cloned_stats = stats.clone();

    // Verify Copy trait is implemented for PoolStats
    let _copied: PoolStats = stats;

    // Verify PartialEq trait is implemented for PoolStats
    assert_eq!(stats, stats);

    // Verify Eq trait is implemented for PoolStats
    let other = PoolStats::default();
    assert_eq!(stats, other);

    // Verify Hash trait is implemented for PoolStats (compile-time check)
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(stats);
}

/// Verify all public methods on StackPool exist with expected signatures.
#[test]
fn test_contract_lock_stack_pool_methods() {
    let pool: StackPool<u32> = StackPool::new(4);

    // Verify new method exists
    let _pool2: StackPool<i32> = StackPool::new(8);

    // Verify acquire method exists and returns Vec<T>
    let stack: Vec<u32> = pool.acquire();
    assert!(stack.capacity() >= 256);

    // Verify release method exists
    pool.release(stack);

    // Verify acquire_with_capacity method exists
    let stack_with_cap = pool.acquire_with_capacity(512);
    assert!(stack_with_cap.capacity() >= 512);
    pool.release(stack_with_cap);

    // Verify clone_stack method exists
    let original = vec![1u32, 2, 3];
    let _cloned = pool.clone_stack(&original);

    // Verify stats method exists and returns PoolStats
    let _stats: PoolStats = pool.stats();

    // Verify reset_stats method exists
    pool.reset_stats();

    // Verify clear method exists
    pool.clear();
}

/// Verify all public functions exist with expected signatures.
#[test]
fn test_contract_lock_functions() {
    // Verify init_thread_local_pool function exists
    init_thread_local_pool(16);

    // Verify get_thread_local_pool function exists and returns Rc<StackPool<u32>>
    let pool: Rc<StackPool<u32>> = get_thread_local_pool();
    let _stack = pool.acquire();
}
