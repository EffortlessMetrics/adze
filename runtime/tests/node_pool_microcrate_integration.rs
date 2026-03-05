use adze::pool::NodePool;
use adze_node_pool_core as core_node_pool;

#[test]
fn runtime_reexport_matches_microcrate_behavior() {
    let runtime_pool = NodePool::<u32>::with_capacity(2);
    let core_pool = core_node_pool::NodePool::<u32>::with_capacity(2);

    runtime_pool.put(runtime_pool.get_or_default());
    core_pool.put(core_pool.get_or_default());

    let runtime_stats = runtime_pool.stats();
    let core_stats = core_pool.stats();

    assert_eq!(runtime_pool.size(), core_pool.size());
    assert_eq!(runtime_stats.gets, core_stats.gets);
    assert_eq!(runtime_stats.puts, core_stats.puts);
    assert_eq!(runtime_stats.misses, core_stats.misses);
}

#[test]
fn runtime_reexport_is_type_compatible() {
    fn accepts_core_type(value: core_node_pool::NodePool<u32>) -> core_node_pool::NodePool<u32> {
        value
    }

    let pool = NodePool::<u32>::new();
    let _ = accepts_core_type(pool);
}
