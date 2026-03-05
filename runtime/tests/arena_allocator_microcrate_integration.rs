use adze::arena_allocator::{NodeHandle as RuntimeHandle, TreeArena as RuntimeArena, TreeNode};
use adze_arena_allocator_core::{NodeHandle as CoreHandle, TreeArena as CoreArena};

#[test]
fn runtime_reexport_matches_microcrate_behavior() {
    let mut runtime = RuntimeArena::with_capacity(2);
    let mut core = CoreArena::with_capacity(2);

    for value in [10, 20, 30] {
        let rh = runtime.alloc(TreeNode::leaf(value));
        let ch = core.alloc(TreeNode::leaf(value));
        assert_eq!(runtime.get(rh).value(), core.get(ch).value());
    }

    assert_eq!(runtime.num_chunks(), core.num_chunks());
    assert_eq!(runtime.len(), core.len());
}

#[test]
fn runtime_reexport_stays_type_compatible() {
    fn accepts_core_type(h: CoreHandle) -> CoreHandle {
        h
    }

    let runtime_handle = RuntimeHandle::new(1, 2);
    let returned = accepts_core_type(runtime_handle);
    assert_eq!(returned, CoreHandle::new(1, 2));
}
