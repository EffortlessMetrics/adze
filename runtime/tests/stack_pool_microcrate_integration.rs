use adze::stack_pool::{StackPool, get_thread_local_pool, init_thread_local_pool};
use adze_stack_pool_core as core_stack_pool;

#[test]
fn runtime_reexport_matches_microcrate_behavior() {
    let runtime_pool: StackPool<u16> = StackPool::new(4);
    let core_pool: core_stack_pool::StackPool<u16> = core_stack_pool::StackPool::new(4);

    let mut runtime_stack = runtime_pool.acquire();
    let mut core_stack = core_pool.acquire();

    runtime_stack.push(7);
    core_stack.push(7);

    runtime_pool.release(runtime_stack);
    core_pool.release(core_stack);

    let runtime_reused = runtime_pool.acquire_with_capacity(32);
    let core_reused = core_pool.acquire_with_capacity(32);

    assert!(runtime_reused.is_empty());
    assert!(core_reused.is_empty());
    assert_eq!(runtime_reused.capacity(), core_reused.capacity());
    assert_eq!(runtime_pool.stats(), core_pool.stats());
}

#[test]
fn runtime_reexport_is_type_compatible() {
    fn accepts_core_type(
        value: core_stack_pool::StackPool<u16>,
    ) -> core_stack_pool::StackPool<u16> {
        value
    }

    let pool: StackPool<u16> = StackPool::new(8);
    let _ = accepts_core_type(pool);
}

#[test]
fn runtime_reexport_thread_local_pool_is_forwarded() {
    init_thread_local_pool(4);

    let runtime_pool = get_thread_local_pool();
    runtime_pool.release(runtime_pool.acquire());

    let core_pool = core_stack_pool::get_thread_local_pool();

    assert_eq!(runtime_pool.stats().pool_hits, core_pool.stats().pool_hits);
}
