use adze::arena_allocator::{TreeArena as RuntimeTreeArena, TreeNode as RuntimeTreeNode};
use adze_arena_allocator_core::{TreeArena as CoreTreeArena, TreeNode as CoreTreeNode};

#[test]
fn runtime_reexport_matches_microcrate_behavior() {
    let mut runtime_arena = RuntimeTreeArena::with_capacity(2);
    let mut core_arena = CoreTreeArena::with_capacity(2);

    for value in 0..5 {
        let rh = runtime_arena.alloc(RuntimeTreeNode::leaf(value));
        let ch = core_arena.alloc(CoreTreeNode::leaf(value));

        assert_eq!(runtime_arena.get(rh).symbol(), core_arena.get(ch).symbol());
    }

    assert_eq!(runtime_arena.len(), core_arena.len());
    assert_eq!(runtime_arena.num_chunks(), core_arena.num_chunks());
}

#[test]
fn runtime_reexport_stays_type_compatible() {
    fn accepts_core_type(value: CoreTreeArena) -> CoreTreeArena {
        value
    }

    let runtime_value = RuntimeTreeArena::new();
    let returned = accepts_core_type(runtime_value);
    assert_eq!(returned.len(), 0);
}
