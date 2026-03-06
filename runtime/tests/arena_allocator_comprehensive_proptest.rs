// Comprehensive property tests for TreeArena
use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// TreeArena construction
// ---------------------------------------------------------------------------

#[test]
fn arena_new_is_empty() {
    let arena = TreeArena::new();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn arena_with_capacity() {
    let arena = TreeArena::with_capacity(100);
    assert!(arena.is_empty());
    assert!(arena.capacity() >= 100);
}

// ---------------------------------------------------------------------------
// TreeNode leaf/branch
// ---------------------------------------------------------------------------

#[test]
fn leaf_node_creation() {
    let node = TreeNode::leaf(42);
    assert!(node.is_leaf());
    assert!(!node.is_branch());
    assert_eq!(node.value(), 42);
    assert_eq!(node.symbol(), 42);
    assert!(node.children().is_empty());
}

#[test]
fn branch_node_creation() {
    let node = TreeNode::branch(vec![]);
    assert!(node.is_branch());
    assert!(!node.is_leaf());
    assert_eq!(node.symbol(), 0);
}

#[test]
fn branch_with_symbol() {
    let node = TreeNode::branch_with_symbol(5, vec![]);
    assert!(node.is_branch());
    assert_eq!(node.symbol(), 5);
    assert_eq!(node.value(), 5);
}

// ---------------------------------------------------------------------------
// Alloc and get
// ---------------------------------------------------------------------------

#[test]
fn alloc_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.len(), 1);
    let n = arena.get(h);
    assert!(n.is_leaf());
    assert_eq!(n.value(), 1);
}

#[test]
fn alloc_branch_with_children() {
    let mut arena = TreeArena::new();
    let c1 = arena.alloc(TreeNode::leaf(1));
    let c2 = arena.alloc(TreeNode::leaf(2));
    let parent = arena.alloc(TreeNode::branch(vec![c1, c2]));
    assert_eq!(arena.len(), 3);
    let p = arena.get(parent);
    assert!(p.is_branch());
    assert_eq!(p.children().len(), 2);
}

#[test]
fn get_ref_matches_node() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(99));
    let r = arena.get(h);
    let node = r.get_ref();
    assert_eq!(node.value(), 99);
}

// ---------------------------------------------------------------------------
// Mutability
// ---------------------------------------------------------------------------

#[test]
fn set_value_via_mut() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    {
        let mut m = arena.get_mut(h);
        m.set_value(42);
    }
    let n = arena.get(h);
    assert_eq!(n.value(), 42);
}

// ---------------------------------------------------------------------------
// Reset and clear
// ---------------------------------------------------------------------------

#[test]
fn reset_clears_arena() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.len(), 2);
    arena.reset();
    assert!(arena.is_empty());
}

#[test]
fn clear_clears_arena() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.clear();
    assert!(arena.is_empty());
}

// ---------------------------------------------------------------------------
// Metrics
// ---------------------------------------------------------------------------

#[test]
fn metrics_empty() {
    let arena = TreeArena::new();
    let m = arena.metrics();
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
}

#[test]
fn metrics_after_alloc() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    let m = arena.metrics();
    assert_eq!(m.len(), 1);
    assert!(!m.is_empty());
    assert!(m.capacity() >= 1);
    assert!(m.num_chunks() >= 1);
    assert!(m.memory_usage() > 0);
}

#[test]
fn metrics_debug() {
    let arena = TreeArena::new();
    let m = arena.metrics();
    let debug = format!("{:?}", m);
    assert!(debug.contains("ArenaMetrics"));
}

// ---------------------------------------------------------------------------
// NodeHandle
// ---------------------------------------------------------------------------

#[test]
fn node_handle_creation() {
    let h = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(1, 5);
    assert_ne!(h, h2);
}

#[test]
fn node_handle_equality() {
    let h1 = NodeHandle::new(1, 2);
    let h2 = NodeHandle::new(1, 2);
    assert_eq!(h1, h2);
}

#[test]
fn node_handle_debug() {
    let h = NodeHandle::new(0, 0);
    let debug = format!("{:?}", h);
    assert!(debug.contains("NodeHandle"));
}

// ---------------------------------------------------------------------------
// Multiple nodes
// ---------------------------------------------------------------------------

#[test]
fn many_leaves() {
    let mut arena = TreeArena::new();
    let handles: Vec<_> = (0..100).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    assert_eq!(arena.len(), 100);
    for (i, h) in handles.iter().enumerate() {
        assert_eq!(arena.get(*h).value(), i as i32);
    }
}

#[test]
fn deep_tree() {
    let mut arena = TreeArena::new();
    let leaf = arena.alloc(TreeNode::leaf(0));
    let mut current = leaf;
    for i in 1..10 {
        current = arena.alloc(TreeNode::branch_with_symbol(i, vec![current]));
    }
    assert_eq!(arena.len(), 10);
    let root = arena.get(current);
    assert!(root.is_branch());
    assert_eq!(root.children().len(), 1);
}

#[test]
fn wide_tree() {
    let mut arena = TreeArena::new();
    let children: Vec<_> = (0..50).map(|i| arena.alloc(TreeNode::leaf(i))).collect();
    let root = arena.alloc(TreeNode::branch(children.clone()));
    let r = arena.get(root);
    assert_eq!(r.children().len(), 50);
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn leaf_preserves_value(v in -1000i32..1000) {
        let node = TreeNode::leaf(v);
        prop_assert_eq!(node.value(), v);
        prop_assert_eq!(node.symbol(), v);
        prop_assert!(node.is_leaf());
    }

    #[test]
    fn branch_preserves_symbol(s in -1000i32..1000) {
        let node = TreeNode::branch_with_symbol(s, vec![]);
        prop_assert_eq!(node.symbol(), s);
        prop_assert!(node.is_branch());
    }

    #[test]
    fn alloc_preserves_count(n in 1usize..100) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        prop_assert_eq!(arena.len(), n);
    }

    #[test]
    fn alloc_then_get(v in -1000i32..1000) {
        let mut arena = TreeArena::new();
        let h = arena.alloc(TreeNode::leaf(v));
        let node_ref = arena.get(h);
        prop_assert_eq!(node_ref.value(), v);
    }

    #[test]
    fn reset_always_empties(n in 1usize..50) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        arena.reset();
        prop_assert!(arena.is_empty());
    }

    #[test]
    fn metrics_consistent(n in 1usize..50) {
        let mut arena = TreeArena::new();
        for i in 0..n {
            arena.alloc(TreeNode::leaf(i as i32));
        }
        let m = arena.metrics();
        prop_assert_eq!(m.len(), n);
        prop_assert!(m.capacity() >= n);
        prop_assert!(m.num_chunks() >= 1);
    }

    #[test]
    fn wide_tree_children_count(width in 1usize..30) {
        let mut arena = TreeArena::new();
        let children: Vec<_> = (0..width)
            .map(|i| arena.alloc(TreeNode::leaf(i as i32)))
            .collect();
        let root = arena.alloc(TreeNode::branch(children));
        let r = arena.get(root);
        prop_assert_eq!(r.children().len(), width);
    }
}
