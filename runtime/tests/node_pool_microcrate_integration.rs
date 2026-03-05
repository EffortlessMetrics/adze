use adze::pool::NodePool as RuntimeNodePool;
use adze_node_pool_core::NodePool as CoreNodePool;

#[test]
fn runtime_reexport_matches_microcrate_behavior() {
    let runtime = RuntimeNodePool::<u32>::with_capacity(1);
    let core = CoreNodePool::<u32>::with_capacity(1);

    runtime.put(runtime.get_or(|| 7));
    core.put(core.get_or(|| 7));

    let runtime_stats = runtime.stats();
    let core_stats = core.stats();

    assert_eq!(runtime_stats.gets, core_stats.gets);
    assert_eq!(runtime_stats.puts, core_stats.puts);
    assert_eq!(runtime_stats.misses, core_stats.misses);
    assert_eq!(runtime_stats.drops, core_stats.drops);
    assert_eq!(runtime_stats.current_size, core_stats.current_size);
}

#[test]
fn runtime_reexport_stays_type_compatible() {
    fn accepts_core_type(value: CoreNodePool<u32>) -> CoreNodePool<u32> {
        value
    }

    let runtime_value = RuntimeNodePool::<u32>::new();
    let returned = accepts_core_type(runtime_value);
    assert_eq!(returned.stats().capacity, 256);
}
