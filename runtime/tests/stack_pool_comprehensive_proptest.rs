// Comprehensive property tests for StackPool
use adze::stack_pool::*;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// StackPool construction
// ---------------------------------------------------------------------------

#[test]
fn pool_new() {
    let pool = StackPool::<u32>::new(10);
    let stats = pool.stats();
    assert_eq!(stats.total_allocations, 0);
    assert_eq!(stats.reuse_count, 0);
}

#[test]
fn pool_new_zero_size() {
    let pool = StackPool::<u32>::new(0);
    let _v = pool.acquire();
}

// ---------------------------------------------------------------------------
// Acquire / release
// ---------------------------------------------------------------------------

#[test]
fn acquire_returns_empty_vec() {
    let pool = StackPool::<u32>::new(10);
    let v = pool.acquire();
    assert!(v.is_empty());
}

#[test]
fn acquire_with_capacity() {
    let pool = StackPool::<u32>::new(10);
    let v = pool.acquire_with_capacity(50);
    assert!(v.is_empty());
    assert!(v.capacity() >= 50);
}

#[test]
fn release_then_acquire() {
    let pool = StackPool::<u32>::new(10);
    let mut v = pool.acquire();
    v.push(1);
    v.push(2);
    pool.release(v);
    let v2 = pool.acquire();
    assert!(v2.is_empty());
}

#[test]
fn clone_stack() {
    let pool = StackPool::<u32>::new(10);
    let source = vec![1u32, 2, 3, 4, 5];
    let cloned = pool.clone_stack(&source);
    assert_eq!(cloned, source);
}

#[test]
fn clone_empty_stack() {
    let pool = StackPool::<u32>::new(10);
    let source: Vec<u32> = vec![];
    let cloned = pool.clone_stack(&source);
    assert!(cloned.is_empty());
}

// ---------------------------------------------------------------------------
// Stats tracking
// ---------------------------------------------------------------------------

#[test]
fn stats_track_allocations() {
    let pool = StackPool::<u32>::new(10);
    let _v1 = pool.acquire();
    let _v2 = pool.acquire();
    let stats = pool.stats();
    assert_eq!(stats.total_allocations, 2);
}

#[test]
fn stats_pool_hits_after_release() {
    let pool = StackPool::<u32>::new(10);
    let v = pool.acquire();
    pool.release(v);
    let _v2 = pool.acquire();
    let stats = pool.stats();
    assert!(stats.pool_hits > 0 || stats.reuse_count > 0);
}

#[test]
fn stats_pool_misses_on_cold_start() {
    let pool = StackPool::<u32>::new(10);
    let _v = pool.acquire();
    let stats = pool.stats();
    assert!(stats.pool_misses > 0 || stats.total_allocations > 0);
}

#[test]
fn stats_reset() {
    let pool = StackPool::<u32>::new(10);
    let _v = pool.acquire();
    pool.reset_stats();
    let stats = pool.stats();
    assert_eq!(stats.total_allocations, 0);
    assert_eq!(stats.reuse_count, 0);
}

// ---------------------------------------------------------------------------
// Clear
// ---------------------------------------------------------------------------

#[test]
fn clear_empties_pool() {
    let pool = StackPool::<u32>::new(10);
    let v = pool.acquire();
    pool.release(v);
    pool.clear();
    let v2 = pool.acquire();
    assert!(v2.is_empty());
}

// ---------------------------------------------------------------------------
// Multiple release/acquire cycles
// ---------------------------------------------------------------------------

#[test]
fn multiple_cycles() {
    let pool = StackPool::<u32>::new(5);
    for i in 0..10 {
        let mut v = pool.acquire();
        v.push(i);
        pool.release(v);
    }
    let stats = pool.stats();
    // First acquire is a fresh alloc; subsequent ones reuse from pool
    assert!(stats.total_allocations >= 1);
    assert!(stats.reuse_count >= 1);
}

// ---------------------------------------------------------------------------
// Multiple types
// ---------------------------------------------------------------------------

#[test]
fn pool_with_string_type() {
    let pool = StackPool::<String>::new(10);
    let mut v = pool.acquire();
    v.push("hello".to_string());
    pool.release(v);
    let v2 = pool.acquire();
    assert!(v2.is_empty());
}

#[test]
fn pool_with_tuple_type() {
    let pool = StackPool::<(u32, u32)>::new(5);
    let mut v = pool.acquire();
    v.push((1, 2));
    pool.release(v);
}

// ---------------------------------------------------------------------------
// Thread-local pool
// ---------------------------------------------------------------------------

#[test]
fn thread_local_pool_init() {
    init_thread_local_pool(16);
    let pool = get_thread_local_pool();
    let v = pool.acquire();
    assert!(v.is_empty());
    pool.release(v);
}

// ---------------------------------------------------------------------------
// PoolStats debug
// ---------------------------------------------------------------------------

#[test]
fn pool_stats_debug() {
    let pool = StackPool::<u32>::new(10);
    let stats = pool.stats();
    let debug = format!("{:?}", stats);
    assert!(debug.contains("PoolStats"));
}

#[test]
fn pool_stats_max_pool_depth() {
    let pool = StackPool::<u32>::new(10);
    let v1 = pool.acquire();
    let v2 = pool.acquire();
    pool.release(v1);
    pool.release(v2);
    let stats = pool.stats();
    assert!(stats.max_pool_depth >= 1);
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn acquire_always_returns_empty(cap in 0usize..100) {
        let pool = StackPool::<u32>::new(10);
        let v = pool.acquire_with_capacity(cap);
        prop_assert!(v.is_empty());
    }

    #[test]
    fn clone_preserves_content(data in proptest::collection::vec(0u32..1000, 0..50)) {
        let pool = StackPool::<u32>::new(10);
        let cloned = pool.clone_stack(&data);
        prop_assert_eq!(cloned, data);
    }

    #[test]
    fn acquire_release_cycle(n in 1usize..20) {
        let pool = StackPool::<u32>::new(10);
        for _ in 0..n {
            let mut v = pool.acquire();
            v.push(42);
            pool.release(v);
        }
        let stats = pool.stats();
        // total_allocations counts new + reused, reuse >= n-1
        prop_assert!(stats.total_allocations >= 1);
        if n > 1 {
            prop_assert!(stats.reuse_count >= 1);
        }
    }

    #[test]
    fn release_clears_content(data in proptest::collection::vec(0u32..1000, 1..20)) {
        let pool = StackPool::<u32>::new(10);
        let mut v = pool.acquire();
        v.extend_from_slice(&data);
        pool.release(v);
        let v2 = pool.acquire();
        prop_assert!(v2.is_empty());
    }

    #[test]
    fn pool_size_bounds_depth(max_size in 1usize..10, releases in 1usize..20) {
        let pool = StackPool::<u32>::new(max_size);
        for _ in 0..releases {
            let v = pool.acquire();
            pool.release(v);
        }
        let stats = pool.stats();
        prop_assert!(stats.max_pool_depth <= max_size);
    }
}
