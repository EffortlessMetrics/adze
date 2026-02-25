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
}
