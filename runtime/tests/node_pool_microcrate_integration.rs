use adze::pool::NodePool as RuntimeNodePool;
use adze_node_pool_core::NodePool as CoreNodePool;
use std::sync::Arc;

#[derive(Default)]
struct Node(u32);

#[test]
fn runtime_reexport_matches_microcrate_behavior() {
    let runtime_pool = RuntimeNodePool::<Node>::with_capacity(1);
    let core_pool = CoreNodePool::<Node>::with_capacity(1);

    let runtime_node = runtime_pool.get_or_default();
    let core_node = core_pool.get_or_default();

    runtime_pool.put(runtime_node);
    core_pool.put(core_node);

    assert_eq!(runtime_pool.size(), core_pool.size());

    let runtime_stats = runtime_pool.stats();
    let core_stats = core_pool.stats();

    assert_eq!(runtime_stats.gets, core_stats.gets);
    assert_eq!(runtime_stats.puts, core_stats.puts);
    assert_eq!(runtime_stats.misses, core_stats.misses);
    assert_eq!(runtime_stats.drops, core_stats.drops);
}

#[test]
fn runtime_reexport_stays_type_compatible() {
    fn accepts_core_type(value: CoreNodePool<Node>) -> CoreNodePool<Node> {
        value
    }

    let runtime_pool = RuntimeNodePool::<Node>::new();
    let _returned = accepts_core_type(runtime_pool);
}

#[test]
fn pool_does_not_store_shared_nodes() {
    let pool = RuntimeNodePool::<Node>::with_capacity(4);

    let node = Arc::new(Node(7));
    let clone = Arc::clone(&node);
    pool.put(node);

    assert_eq!(pool.size(), 0);
    assert_eq!(clone.0, 7);
}
