use adze_stack_pool_core::StackPool;
use proptest::prelude::*;

fn run_ops(ops: &[u8], pool: &StackPool<usize>, max_pool_size: usize) -> usize {
    let mut active: Vec<Vec<usize>> = Vec::new();
    let mut total_acquisitions = 0usize;

    for &raw in ops {
        match raw % 5 {
            0 => {
                active.push(pool.acquire());
                total_acquisitions += 1;
            }
            1 => {
                let capacity = usize::from(raw % 128) + 1;
                active.push(pool.acquire_with_capacity(capacity));
                total_acquisitions += 1;
            }
            2 => {
                if let Some(mut stack) = active.pop() {
                    for value in 0..usize::from(raw % 4) {
                        stack.push(value);
                    }
                    pool.release(stack);
                }
            }
            3 => {
                if !active.is_empty() {
                    let idx = usize::from(raw) % active.len();
                    let clone = pool.clone_stack(&active[idx]);
                    active.push(clone);
                    total_acquisitions += 1;
                }
            }
            _ => {
                let _ = pool.stats();
            }
        }

        let stats = pool.stats();
        assert_eq!(stats.total_allocations, stats.pool_misses);
        assert_eq!(stats.reuse_count, stats.pool_hits);
        assert!(stats.max_pool_depth <= max_pool_size);
        assert_eq!(stats.pool_hits + stats.pool_misses, total_acquisitions);
    }

    for stack in active {
        pool.release(stack);
    }

    total_acquisitions
}

proptest! {
    #[test]
    fn arbitrary_operations_keep_pool_stats_consistent(ops in prop::collection::vec(any::<u8>(), 0..2048)) {
        let max_pool_size = 16usize;
        let pool: StackPool<usize> = StackPool::new(max_pool_size);
        let total_acquisitions = run_ops(&ops, &pool, max_pool_size);

        let stats = pool.stats();
        assert_eq!(stats.total_allocations, stats.pool_misses);
        assert_eq!(stats.reuse_count, stats.pool_hits);
        assert_eq!(stats.pool_hits + stats.pool_misses, total_acquisitions);
        assert!(stats.max_pool_depth <= max_pool_size);
    }

    #[test]
    fn acquired_stacks_are_always_empty(ops in prop::collection::vec(any::<u8>(), 1..128)) {
        let pool: StackPool<u32> = StackPool::new(8);
        for &raw in &ops {
            // Release a non-empty stack, then re-acquire — must be cleared
            let mut stack = pool.acquire();
            stack.push(raw as u32);
            stack.push(42);
            pool.release(stack);

            let reacquired = pool.acquire();
            prop_assert!(reacquired.is_empty(), "acquired stack was not empty");
            pool.release(reacquired);
        }
    }

    #[test]
    fn acquire_with_capacity_returns_sufficient_capacity(
        requested in 1usize..512,
    ) {
        let pool: StackPool<u8> = StackPool::new(4);
        let stack = pool.acquire_with_capacity(requested);
        prop_assert!(stack.capacity() >= requested,
            "capacity {} < requested {}", stack.capacity(), requested);
    }

    #[test]
    fn clone_stack_is_exact_copy(
        data in prop::collection::vec(0u32..1000, 0..128),
    ) {
        let pool: StackPool<u32> = StackPool::new(4);
        let cloned = pool.clone_stack(&data);
        prop_assert_eq!(&cloned[..], &data[..]);
    }

    #[test]
    fn reset_stats_is_idempotent(ops in prop::collection::vec(any::<u8>(), 0..64)) {
        let pool: StackPool<u32> = StackPool::new(4);
        for _ in &ops {
            let s = pool.acquire();
            pool.release(s);
        }
        pool.reset_stats();
        let after_first = pool.stats();
        pool.reset_stats();
        let after_second = pool.stats();
        prop_assert_eq!(after_first, after_second);
    }

    #[test]
    fn clear_then_acquire_always_misses(
        release_count in 1usize..32,
    ) {
        let pool: StackPool<u32> = StackPool::new(64);
        for _ in 0..release_count {
            let s = pool.acquire();
            pool.release(s);
        }
        pool.reset_stats();
        pool.clear();

        let _s = pool.acquire();
        let stats = pool.stats();
        prop_assert_eq!(stats.pool_hits, 0);
        prop_assert_eq!(stats.pool_misses, 1);
    }

    #[test]
    fn oversized_stacks_are_never_pooled(
        extra in 0usize..64,
    ) {
        let pool: StackPool<u32> = StackPool::new(8);
        // capacity > 4096 should be rejected
        let big: Vec<u32> = Vec::with_capacity(4097 + extra);
        pool.release(big);
        prop_assert_eq!(pool.stats().max_pool_depth, 0);
    }

    #[test]
    fn pool_depth_never_exceeds_max(
        max_size in 1usize..32,
        release_count in 0usize..128,
    ) {
        let pool: StackPool<u32> = StackPool::new(max_size);
        for _ in 0..release_count {
            let s = pool.acquire();
            pool.release(s);
        }
        prop_assert!(pool.stats().max_pool_depth <= max_size);
    }
}
