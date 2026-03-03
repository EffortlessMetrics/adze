use adze_stack_pool_core::{StackPool, get_thread_local_pool, init_thread_local_pool};

// ── 1. Pool allocation and deallocation ──────────────────────────────────

#[test]
fn fresh_pool_has_zero_stats() {
    let pool: StackPool<u32> = StackPool::new(8);
    let stats = pool.stats();
    assert_eq!(stats.total_allocations, 0);
    assert_eq!(stats.reuse_count, 0);
    assert_eq!(stats.pool_hits, 0);
    assert_eq!(stats.pool_misses, 0);
    assert_eq!(stats.max_pool_depth, 0);
}

#[test]
fn multiple_allocations_tracked_correctly() {
    let pool: StackPool<u32> = StackPool::new(4);
    let s1 = pool.acquire();
    let s2 = pool.acquire();
    let s3 = pool.acquire();
    assert_eq!(pool.stats().total_allocations, 3);
    assert_eq!(pool.stats().pool_misses, 3);
    assert_eq!(pool.stats().pool_hits, 0);
    // Clean up to avoid leak warnings
    drop(s1);
    drop(s2);
    drop(s3);
}

#[test]
fn release_then_reacquire_does_not_increment_total_allocations() {
    let pool: StackPool<u32> = StackPool::new(4);
    let s = pool.acquire();
    pool.release(s);
    let _reused = pool.acquire();
    // Only 1 real allocation; the second was a reuse
    assert_eq!(pool.stats().total_allocations, 1);
    assert_eq!(pool.stats().pool_hits, 1);
}

// ── 2. Stack push/pop operations ─────────────────────────────────────────

#[test]
fn push_and_pop_on_acquired_stack() {
    let pool: StackPool<i32> = StackPool::new(4);
    let mut stack = pool.acquire();
    stack.push(10);
    stack.push(20);
    stack.push(30);
    assert_eq!(stack.pop(), Some(30));
    assert_eq!(stack.pop(), Some(20));
    assert_eq!(stack.pop(), Some(10));
    assert_eq!(stack.pop(), None);
    pool.release(stack);
}

#[test]
fn push_pop_cycle_then_reuse_yields_empty_stack() {
    let pool: StackPool<u64> = StackPool::new(4);
    let mut stack = pool.acquire();
    for i in 0..100 {
        stack.push(i);
    }
    for _ in 0..50 {
        stack.pop();
    }
    assert_eq!(stack.len(), 50);
    pool.release(stack);

    let reused = pool.acquire();
    assert!(reused.is_empty());
    pool.release(reused);
}

#[test]
fn stack_retains_capacity_after_push_pop_and_reuse() {
    let pool: StackPool<u32> = StackPool::new(4);
    let mut stack = pool.acquire();
    // Push enough to potentially grow beyond initial 256
    for i in 0..300 {
        stack.push(i);
    }
    let grown_capacity = stack.capacity();
    assert!(grown_capacity >= 300);
    pool.release(stack);

    let reused = pool.acquire();
    // Reused stack keeps the grown capacity
    assert!(reused.capacity() >= grown_capacity);
    assert!(reused.is_empty());
    pool.release(reused);
}

// ── 3. Pool capacity growth ──────────────────────────────────────────────

#[test]
fn pool_depth_grows_up_to_max() {
    let pool: StackPool<u32> = StackPool::new(3);
    let s1 = pool.acquire();
    let s2 = pool.acquire();
    let s3 = pool.acquire();
    pool.release(s1);
    assert_eq!(pool.stats().max_pool_depth, 1);
    pool.release(s2);
    assert_eq!(pool.stats().max_pool_depth, 2);
    pool.release(s3);
    assert_eq!(pool.stats().max_pool_depth, 3);
}

#[test]
fn pool_depth_stops_at_max_size() {
    let pool: StackPool<u32> = StackPool::new(2);
    let s1 = pool.acquire();
    let s2 = pool.acquire();
    let s3 = pool.acquire();
    pool.release(s1);
    pool.release(s2);
    pool.release(s3); // exceeds max_pool_size=2, should be dropped
    assert_eq!(pool.stats().max_pool_depth, 2);
}

#[test]
fn acquire_with_capacity_prefers_best_fit() {
    let pool: StackPool<u8> = StackPool::new(4);
    // Seed pool with various capacities
    let small: Vec<u8> = Vec::with_capacity(32);
    let medium: Vec<u8> = Vec::with_capacity(128);
    let large: Vec<u8> = Vec::with_capacity(512);
    pool.release(small);
    pool.release(medium);
    pool.release(large);

    // Request capacity 100: should pick the first >= 100 (128)
    let acquired = pool.acquire_with_capacity(100);
    assert!(acquired.capacity() >= 100);
}

// ── 4. Stack reuse after return ──────────────────────────────────────────

#[test]
fn reused_stacks_are_always_cleared() {
    let pool: StackPool<String> = StackPool::new(4);
    let mut stack = pool.acquire();
    stack.push("hello".to_string());
    stack.push("world".to_string());
    pool.release(stack);

    let reused = pool.acquire();
    assert!(reused.is_empty(), "reused stack should be empty");
    pool.release(reused);
}

#[test]
fn repeated_acquire_release_cycles_reuse_correctly() {
    let pool: StackPool<u32> = StackPool::new(2);
    for i in 0..10 {
        let mut s = pool.acquire();
        s.push(i);
        pool.release(s);
    }
    let stats = pool.stats();
    // First acquire is a miss, subsequent 9 are hits
    assert_eq!(stats.pool_misses, 1);
    assert_eq!(stats.pool_hits, 9);
    assert_eq!(stats.reuse_count, 9);
    assert_eq!(stats.total_allocations, 1);
}

#[test]
fn clone_stack_reuses_pooled_stack_when_available() {
    let pool: StackPool<u32> = StackPool::new(4);
    // Seed pool with a large-capacity stack
    let big: Vec<u32> = Vec::with_capacity(512);
    pool.release(big);
    pool.reset_stats();

    let source = vec![1, 2, 3, 4, 5];
    let cloned = pool.clone_stack(&source);
    assert_eq!(&cloned[..], &source[..]);
    // Should have reused the pooled stack (hit)
    assert_eq!(pool.stats().pool_hits, 1);
}

// ── 5. Multiple stacks from same pool ────────────────────────────────────

#[test]
fn multiple_simultaneous_stacks_from_one_pool() {
    let pool: StackPool<u32> = StackPool::new(4);
    let mut s1 = pool.acquire();
    let mut s2 = pool.acquire();
    let mut s3 = pool.acquire();

    s1.push(1);
    s2.push(2);
    s3.push(3);

    // They are independent
    assert_eq!(s1, vec![1]);
    assert_eq!(s2, vec![2]);
    assert_eq!(s3, vec![3]);

    pool.release(s1);
    pool.release(s2);
    pool.release(s3);

    assert_eq!(pool.stats().total_allocations, 3);
    assert_eq!(pool.stats().max_pool_depth, 3);
}

#[test]
fn interleaved_acquire_and_release() {
    let pool: StackPool<u32> = StackPool::new(4);

    let s1 = pool.acquire(); // miss
    let s2 = pool.acquire(); // miss
    pool.release(s1);
    let s3 = pool.acquire(); // hit (reuses s1)
    pool.release(s2);
    pool.release(s3);

    let stats = pool.stats();
    assert_eq!(stats.pool_misses, 2);
    assert_eq!(stats.pool_hits, 1);
    assert_eq!(stats.total_allocations, 2);
}

// ── 6. Edge cases ────────────────────────────────────────────────────────

#[test]
fn zero_capacity_pool_never_pools() {
    let pool: StackPool<u32> = StackPool::new(0);
    let s = pool.acquire();
    pool.release(s);
    // Pool can't hold anything, so release just drops
    assert_eq!(pool.stats().max_pool_depth, 0);

    let _ = pool.acquire(); // must allocate fresh
    assert_eq!(pool.stats().pool_misses, 2);
    assert_eq!(pool.stats().pool_hits, 0);
}

#[test]
fn acquire_with_zero_capacity() {
    let pool: StackPool<u32> = StackPool::new(4);
    let s = pool.acquire_with_capacity(0);
    // Should still work; Vec::with_capacity(0) is valid
    assert!(s.is_empty());
    pool.release(s);
}

#[test]
fn empty_stack_operations() {
    let pool: StackPool<u32> = StackPool::new(4);
    let stack = pool.acquire();
    assert!(stack.is_empty());
    assert_eq!(stack.len(), 0);
    pool.release(stack);
}

#[test]
fn release_empty_vec_with_no_capacity() {
    let pool: StackPool<u32> = StackPool::new(4);
    let empty: Vec<u32> = Vec::new();
    pool.release(empty);
    // Should be accepted (capacity 0 <= 4096)
    assert_eq!(pool.stats().max_pool_depth, 1);
}

#[test]
fn release_exactly_at_max_capacity_boundary() {
    let pool: StackPool<u8> = StackPool::new(4);
    let at_limit: Vec<u8> = Vec::with_capacity(4096);
    pool.release(at_limit);
    assert_eq!(pool.stats().max_pool_depth, 1);
}

#[test]
fn release_one_over_max_capacity_boundary() {
    let pool: StackPool<u8> = StackPool::new(4);
    let over_limit: Vec<u8> = Vec::with_capacity(4097);
    pool.release(over_limit);
    assert_eq!(pool.stats().max_pool_depth, 0);
}

#[test]
fn clone_empty_slice() {
    let pool: StackPool<u32> = StackPool::new(4);
    let empty: &[u32] = &[];
    let cloned = pool.clone_stack(empty);
    assert!(cloned.is_empty());
}

#[test]
fn clone_single_element() {
    let pool: StackPool<u32> = StackPool::new(4);
    let single = vec![42u32];
    let cloned = pool.clone_stack(&single);
    assert_eq!(cloned, vec![42]);
}

#[test]
fn debug_format_includes_pool_info() {
    let pool: StackPool<u32> = StackPool::new(4);
    let _ = pool.acquire();
    let debug = format!("{:?}", pool);
    assert!(debug.contains("StackPool"));
    assert!(debug.contains("max_pool_size"));
}

// ── 7. Thread safety (thread-local isolation) ────────────────────────────

#[test]
fn thread_local_pool_auto_initializes() {
    let pool = get_thread_local_pool();
    let s = pool.acquire();
    assert!(s.is_empty());
    pool.release(s);
}

#[test]
fn thread_local_pool_returns_same_instance() {
    init_thread_local_pool(8);
    let a = get_thread_local_pool();
    let b = get_thread_local_pool();
    assert!(std::rc::Rc::ptr_eq(&a, &b));
}

#[test]
fn thread_local_pools_are_isolated_across_threads() {
    use std::sync::mpsc;

    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

    let t1 = std::thread::spawn(move || {
        init_thread_local_pool(4);
        let pool = get_thread_local_pool();
        let s = pool.acquire();
        pool.release(s);
        tx1.send(pool.stats().total_allocations).unwrap();
    });

    let t2 = std::thread::spawn(move || {
        init_thread_local_pool(4);
        let pool = get_thread_local_pool();
        let _s1 = pool.acquire();
        let _s2 = pool.acquire();
        tx2.send(pool.stats().total_allocations).unwrap();
    });

    t1.join().unwrap();
    t2.join().unwrap();

    let allocs_t1 = rx1.recv().unwrap();
    let allocs_t2 = rx2.recv().unwrap();
    // Thread 1 made 1 allocation, thread 2 made 2 — they're independent
    assert_eq!(allocs_t1, 1);
    assert_eq!(allocs_t2, 2);
}

// ── 8. Memory efficiency ─────────────────────────────────────────────────

#[test]
fn pool_reuse_avoids_repeated_allocation() {
    let pool: StackPool<u32> = StackPool::new(4);
    for _ in 0..100 {
        let s = pool.acquire();
        pool.release(s);
    }
    let stats = pool.stats();
    // Only 1 real allocation for 100 acquire/release cycles
    assert_eq!(stats.total_allocations, 1);
    assert_eq!(stats.pool_hits, 99);
}

#[test]
fn capacity_preserved_across_reuse_cycles() {
    let pool: StackPool<u32> = StackPool::new(4);
    let mut s = pool.acquire();
    // Grow the stack well beyond default 256
    for i in 0..500 {
        s.push(i);
    }
    let cap = s.capacity();
    pool.release(s);

    let reused = pool.acquire();
    assert_eq!(
        reused.capacity(),
        cap,
        "capacity should be preserved on reuse"
    );
    pool.release(reused);
}

#[test]
fn oversized_stack_not_pooled_saves_memory() {
    let pool: StackPool<u32> = StackPool::new(4);
    // Release a huge stack — it should be dropped, not pooled
    let huge: Vec<u32> = Vec::with_capacity(10_000);
    pool.release(huge);

    // Next acquire must allocate fresh (miss)
    let _ = pool.acquire();
    assert_eq!(pool.stats().pool_hits, 0);
    assert_eq!(pool.stats().pool_misses, 1);
}

#[test]
fn clear_frees_all_pooled_stacks() {
    let pool: StackPool<u32> = StackPool::new(4);
    for _ in 0..4 {
        let s = pool.acquire();
        pool.release(s);
    }
    pool.clear();
    pool.reset_stats();

    // After clear, everything is a miss
    let _ = pool.acquire();
    assert_eq!(pool.stats().pool_hits, 0);
    assert_eq!(pool.stats().pool_misses, 1);
}

#[test]
fn acquire_with_capacity_falls_back_to_fresh_when_none_big_enough() {
    let pool: StackPool<u32> = StackPool::new(4);
    // Seed pool with small stacks
    for _ in 0..3 {
        let small: Vec<u32> = Vec::with_capacity(16);
        pool.release(small);
    }
    pool.reset_stats();

    // Request much larger capacity — no pooled stack qualifies
    let s = pool.acquire_with_capacity(1024);
    assert!(s.capacity() >= 1024);
    assert_eq!(pool.stats().pool_misses, 1);
    assert_eq!(pool.stats().pool_hits, 0);
}

// ── PoolStats equality / hash ────────────────────────────────────────────

#[test]
fn pool_stats_default_is_all_zeros() {
    let stats = adze_stack_pool_core::PoolStats::default();
    assert_eq!(stats.total_allocations, 0);
    assert_eq!(stats.reuse_count, 0);
    assert_eq!(stats.pool_hits, 0);
    assert_eq!(stats.pool_misses, 0);
    assert_eq!(stats.max_pool_depth, 0);
}

#[test]
fn pool_stats_equality() {
    let pool: StackPool<u32> = StackPool::new(4);
    let s = pool.acquire();
    pool.release(s);
    let _ = pool.acquire();

    let a = pool.stats();
    let b = pool.stats();
    assert_eq!(a, b);
}
