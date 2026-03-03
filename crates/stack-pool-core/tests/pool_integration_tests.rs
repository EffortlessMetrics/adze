use adze_stack_pool_core::{StackPool, get_thread_local_pool, init_thread_local_pool};

#[test]
fn new_pool_has_zero_stats() {
    let pool: StackPool<u32> = StackPool::new(4);
    let stats = pool.stats();
    assert_eq!(stats.total_allocations, 0);
    assert_eq!(stats.reuse_count, 0);
    assert_eq!(stats.pool_hits, 0);
    assert_eq!(stats.pool_misses, 0);
    assert_eq!(stats.max_pool_depth, 0);
}

#[test]
fn acquire_release_acquire_reuses() {
    let pool: StackPool<u32> = StackPool::new(4);
    let mut stack = pool.acquire();
    stack.push(1);
    stack.push(2);
    pool.release(stack);

    let reused = pool.acquire();
    assert!(reused.is_empty());
    assert_eq!(pool.stats().reuse_count, 1);
    assert_eq!(pool.stats().pool_hits, 1);
}

#[test]
fn acquire_with_capacity_finds_large_enough_stack() {
    let pool: StackPool<u32> = StackPool::new(4);
    let small = Vec::with_capacity(32);
    let large = Vec::with_capacity(512);
    pool.release(small);
    pool.release(large);

    let acquired = pool.acquire_with_capacity(256);
    assert!(acquired.capacity() >= 256);
    assert_eq!(pool.stats().pool_hits, 1);
}

#[test]
fn release_respects_max_pool_size() {
    let pool: StackPool<u32> = StackPool::new(2);
    pool.release(Vec::with_capacity(8));
    pool.release(Vec::with_capacity(8));
    pool.release(Vec::with_capacity(8)); // over limit
    assert_eq!(pool.stats().max_pool_depth, 2);
}

#[test]
fn release_rejects_oversized_stacks() {
    let pool: StackPool<u32> = StackPool::new(4);
    let big = vec![0u32; 5000];
    pool.release(big);
    assert_eq!(pool.stats().max_pool_depth, 0);
}

#[test]
fn clone_stack_preserves_contents() {
    let pool: StackPool<u32> = StackPool::new(4);
    let original = vec![10, 20, 30];
    let cloned = pool.clone_stack(&original);
    assert_eq!(cloned, vec![10, 20, 30]);
}

#[test]
fn clear_empties_pool() {
    let pool: StackPool<u32> = StackPool::new(4);
    let s = pool.acquire();
    pool.release(s);
    pool.clear();
    pool.reset_stats();

    let _ = pool.acquire();
    assert_eq!(pool.stats().pool_hits, 0);
    assert_eq!(pool.stats().pool_misses, 1);
}

#[test]
fn reset_stats_clears_all_counters() {
    let pool: StackPool<u32> = StackPool::new(4);
    let s = pool.acquire();
    pool.release(s);
    let _ = pool.acquire();

    pool.reset_stats();
    let stats = pool.stats();
    assert_eq!(stats.total_allocations, 0);
    assert_eq!(stats.reuse_count, 0);
    assert_eq!(stats.pool_hits, 0);
    assert_eq!(stats.pool_misses, 0);
    assert_eq!(stats.max_pool_depth, 0);
}

#[test]
fn debug_format_contains_pool_info() {
    let pool: StackPool<u32> = StackPool::new(4);
    let debug = format!("{pool:?}");
    assert!(debug.contains("StackPool"));
    assert!(debug.contains("max_pool_size"));
}

#[test]
fn thread_local_pool_initializes_and_works() {
    init_thread_local_pool(8);
    let pool = get_thread_local_pool();
    let stack = pool.acquire();
    assert_eq!(stack.capacity(), 256);
    pool.release(stack);

    let reused = pool.acquire();
    assert!(reused.is_empty());
    assert_eq!(pool.stats().reuse_count, 1);
}
