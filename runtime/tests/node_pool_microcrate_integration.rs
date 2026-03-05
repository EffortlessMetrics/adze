use adze::pool::NodePool;
use adze_node_pool_core as core_node_pool;

#[test]
fn runtime_reexport_matches_microcrate_behavior() {
    let runtime_pool: NodePool<u32> = NodePool::with_capacity(2);
    let core_pool: core_node_pool::NodePool<u32> = core_node_pool::NodePool::with_capacity(2);

    let runtime_value = runtime_pool.get_or(|| 7);
    let core_value = core_pool.get_or(|| 7);

    runtime_pool.put(runtime_value);
    core_pool.put(core_value);

    assert_eq!(runtime_pool.size(), core_pool.size());
    assert_eq!(runtime_pool.stats().puts, core_pool.stats().puts);
}

#[test]
fn runtime_reexport_is_type_compatible() {
    fn accepts_core_type(value: core_node_pool::NodePool<u16>) -> core_node_pool::NodePool<u16> {
        value
    }

    let pool: NodePool<u16> = NodePool::new();
    let _ = accepts_core_type(pool);
}
