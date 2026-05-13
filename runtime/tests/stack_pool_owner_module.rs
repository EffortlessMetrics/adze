use adze::stack_pool::{PoolStats, StackPool, get_thread_local_pool, init_thread_local_pool};
use std::collections::HashSet;
use std::rc::Rc;

#[test]
fn runtime_owner_module_exposes_stack_pool_contract() {
    let pool: StackPool<u32> = StackPool::new(4);
    let _debug = format!("{pool:?}");

    let stats = PoolStats {
        total_allocations: 0,
        reuse_count: 0,
        pool_hits: 0,
        pool_misses: 0,
        max_pool_depth: 0,
    };

    assert_eq!(stats.total_allocations, 0);
    assert_eq!(stats.reuse_count, 0);
    assert_eq!(stats.pool_hits, 0);
    assert_eq!(stats.pool_misses, 0);
    assert_eq!(stats.max_pool_depth, 0);
    assert_eq!(PoolStats::default(), stats);

    let mut set = HashSet::new();
    set.insert(stats);
    assert!(set.contains(&stats));
}

#[test]
fn runtime_owner_module_reuses_released_stacks() {
    let pool: StackPool<u16> = StackPool::new(4);

    let mut stack = pool.acquire();
    stack.push(7);
    pool.release(stack);

    let reused = pool.acquire_with_capacity(32);

    assert!(reused.is_empty());
    assert!(reused.capacity() >= 32);
    assert_eq!(pool.stats().pool_hits, 1);
    assert_eq!(pool.stats().reuse_count, 1);
}

#[test]
fn runtime_owner_module_keeps_thread_local_pool() {
    init_thread_local_pool(4);

    let pool: Rc<StackPool<u32>> = get_thread_local_pool();
    let stack = pool.acquire();
    pool.release(stack);

    let same_pool = get_thread_local_pool();

    assert_eq!(pool.stats().pool_hits, same_pool.stats().pool_hits);
}
