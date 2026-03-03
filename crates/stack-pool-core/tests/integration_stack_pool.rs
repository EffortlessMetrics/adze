use adze_stack_pool_core::{get_thread_local_pool, init_thread_local_pool};
use std::rc::Rc;

#[test]
fn external_consumer_can_reconfigure_and_reuse_thread_local_pool() {
    init_thread_local_pool(4);

    let pool = get_thread_local_pool();
    let same_pool = get_thread_local_pool();
    assert!(Rc::ptr_eq(&pool, &same_pool));

    let stack = pool.acquire();
    assert!(stack.is_empty());
    assert_eq!(pool.stats().pool_misses, 1);

    pool.release(stack);
    let _ = pool.acquire();
    assert_eq!(pool.stats().pool_hits, 1);
}
