use adze_stack_pool_core::StackPool;

#[test]
fn scenario_when_acquiring_then_releasing_reuses_capacity() {
    let pool: StackPool<u16> = StackPool::new(4);
    let stack = pool.acquire_with_capacity(16);
    assert_eq!(stack.capacity(), 16);

    pool.release(stack);

    let reused = pool.acquire_with_capacity(8);
    assert!(reused.capacity() >= 16);
    assert_eq!(pool.stats().pool_hits, 1);
}

#[test]
fn scenario_when_release_pool_is_too_large_it_is_dropped() {
    let pool: StackPool<u16> = StackPool::new(1);

    pool.release(Vec::<u16>::with_capacity(5000));
    assert_eq!(pool.stats().max_pool_depth, 0);
}

#[test]
fn scenario_thread_local_pool_is_available_on_demand() {
    let pool = adze_stack_pool_core::get_thread_local_pool();
    let stack = pool.acquire();

    assert!(stack.is_empty());
    assert_eq!(pool.stats().pool_misses, 1);

    pool.release(stack);
    let hit = pool.acquire();
    assert!(hit.is_empty());
    assert!(pool.stats().pool_hits >= 1);
}
