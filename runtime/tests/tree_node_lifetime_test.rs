//! Tree<'arena> and Node<'arena> Lifetime Integration Tests (Phase 2 Day 4)
//!
//! This test suite implements behavioral specifications from:
//! - docs/specs/PARSER_ARENA_INTEGRATION_SPEC.md (Spec 2, 4, 6)
//! - docs/specs/NODE_ARENA_SPEC.md (All specs)
//!
//! Day 4 Focus: Lifetime-bound tree and node types
//! - Add 'arena lifetime parameter to Tree
//! - Replace fields with NodeHandle and arena reference
//! - Implement Node<'arena> wrapper for arena-allocated nodes
//! - Implement Tree::root_node() and Node accessors

use rust_sitter::arena_allocator::{NodeHandle, TreeArena};
use rust_sitter::tree_node_data::TreeNodeData;

// These types will be implemented during Day 4
// For now, tests are structured to define the contract

// ============================================================================
// Spec 1: Node Creation and Access
// ============================================================================

#[test]
fn spec_1_node_creation() {
    // Node should provide access to underlying TreeNodeData

    let mut arena = TreeArena::new();
    let data = TreeNodeData::leaf(42, 10, 20);
    let handle = arena.alloc(data);

    // TODO: Uncomment when Node is implemented
    // let node = Node::new(handle, &arena);
    //
    // assert_eq!(node.symbol(), 42);
    // assert_eq!(node.byte_range(), (10, 20));
    // assert_eq!(node.start_byte(), 10);
    // assert_eq!(node.end_byte(), 20);
}

#[test]
fn spec_1_node_size() {
    // Node should be 16 bytes (handle + reference)

    // TODO: Uncomment when Node is implemented
    // use std::mem::size_of;
    // use rust_sitter::node::Node;
    //
    // assert_eq!(size_of::<Node>(), 16);
}

// ============================================================================
// Spec 2: Node is Copy
// ============================================================================

#[test]
fn spec_2_node_is_copy() {
    // Node should implement Copy (no clone required)

    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNodeData::leaf(1, 0, 10));

    // TODO: Uncomment when Node is implemented
    // let node1 = Node::new(handle, &arena);
    // let node2 = node1; // Copy, not move
    //
    // // Both usable
    // assert_eq!(node1.symbol(), 1);
    // assert_eq!(node2.symbol(), 1);
}

// ============================================================================
// Spec 3: Child Access
// ============================================================================

#[test]
fn spec_3_child_access() {
    // Node should provide indexed and iterator-based child access

    let mut arena = TreeArena::new();

    // Create children
    let child1 = arena.alloc(TreeNodeData::leaf(1, 0, 5));
    let child2 = arena.alloc(TreeNodeData::leaf(2, 5, 10));

    // Create parent
    let parent_data = TreeNodeData::branch(10, 0, 10, vec![child1, child2]);
    let parent_handle = arena.alloc(parent_data);

    // TODO: Uncomment when Node is implemented
    // let parent = Node::new(parent_handle, &arena);
    //
    // // Test child access
    // assert_eq!(parent.child_count(), 2);
    // assert!(parent.child(0).is_some());
    // assert!(parent.child(1).is_some());
    // assert!(parent.child(2).is_none());
    //
    // // Test iterator
    // let children: Vec<_> = parent.children().collect();
    // assert_eq!(children.len(), 2);
    // assert_eq!(children[0].symbol(), 1);
    // assert_eq!(children[1].symbol(), 2);
}

#[test]
fn spec_3_child_bounds_checking() {
    // child() should return None for out-of-bounds indices

    let mut arena = TreeArena::new();
    let child1 = arena.alloc(TreeNodeData::leaf(1, 0, 5));
    let parent_data = TreeNodeData::branch(10, 0, 5, vec![child1]);
    let parent_handle = arena.alloc(parent_data);

    // TODO: Uncomment when Node is implemented
    // let parent = Node::new(parent_handle, &arena);
    //
    // assert!(parent.child(0).is_some());
    // assert!(parent.child(1).is_none());
    // assert!(parent.child(100).is_none());
}

// ============================================================================
// Spec 4: Named Children
// ============================================================================

#[test]
fn spec_4_named_children() {
    // named_child_count() and named_children() should filter by is_named flag

    let mut arena = TreeArena::new();

    // Named child (is_named flag set)
    let mut named = TreeNodeData::leaf(1, 0, 5);
    named.set_named(true);
    let named_handle = arena.alloc(named);

    // Anonymous child
    let anon = TreeNodeData::leaf(2, 5, 6);
    let anon_handle = arena.alloc(anon);

    // Parent with both
    let parent_data = TreeNodeData::branch(10, 0, 6, vec![named_handle, anon_handle]);
    let parent_handle = arena.alloc(parent_data);

    // TODO: Uncomment when Node is implemented
    // let parent = Node::new(parent_handle, &arena);
    //
    // assert_eq!(parent.child_count(), 2);
    // assert_eq!(parent.named_child_count(), 1);
    //
    // let named_child = parent.named_child(0).unwrap();
    // assert_eq!(named_child.symbol(), 1);
    // assert!(named_child.is_named());
    //
    // // Second named child doesn't exist
    // assert!(parent.named_child(1).is_none());
}

#[test]
fn spec_4_named_children_iterator() {
    // named_children() iterator should yield only named children

    let mut arena = TreeArena::new();

    let mut named1 = TreeNodeData::leaf(1, 0, 5);
    named1.set_named(true);
    let named1_handle = arena.alloc(named1);

    let anon = TreeNodeData::leaf(2, 5, 6);
    let anon_handle = arena.alloc(anon);

    let mut named2 = TreeNodeData::leaf(3, 6, 10);
    named2.set_named(true);
    let named2_handle = arena.alloc(named2);

    let parent_data = TreeNodeData::branch(
        10,
        0,
        10,
        vec![named1_handle, anon_handle, named2_handle],
    );
    let parent_handle = arena.alloc(parent_data);

    // TODO: Uncomment when Node is implemented
    // let parent = Node::new(parent_handle, &arena);
    //
    // let named: Vec<_> = parent.named_children().collect();
    // assert_eq!(named.len(), 2);
    // assert_eq!(named[0].symbol(), 1);
    // assert_eq!(named[1].symbol(), 3);
}

// ============================================================================
// Spec 5: Node Flags
// ============================================================================

#[test]
fn spec_5_node_flags() {
    // Flag accessor methods should reflect TreeNodeData state

    let mut arena = TreeArena::new();

    let mut data = TreeNodeData::leaf(1, 0, 10);
    data.set_named(true);
    data.set_missing(true);
    data.set_extra(false);
    data.set_has_error(true);

    let handle = arena.alloc(data);

    // TODO: Uncomment when Node is implemented
    // let node = Node::new(handle, &arena);
    //
    // assert!(node.is_named());
    // assert!(node.is_missing());
    // assert!(!node.is_extra());
    // assert!(node.has_error());
}

#[test]
fn spec_5_all_flags_false() {
    // Default flags should all be false

    let mut arena = TreeArena::new();
    let data = TreeNodeData::leaf(1, 0, 10);
    let handle = arena.alloc(data);

    // TODO: Uncomment when Node is implemented
    // let node = Node::new(handle, &arena);
    //
    // assert!(!node.is_named());
    // assert!(!node.is_missing());
    // assert!(!node.is_extra());
    // assert!(!node.has_error());
}

// ============================================================================
// Spec 6: Data Access
// ============================================================================

#[test]
fn spec_6_data_access() {
    // node.data() should return reference to underlying TreeNodeData

    let mut arena = TreeArena::new();
    let data = TreeNodeData::leaf(42, 100, 200);
    let handle = arena.alloc(data);

    // TODO: Uncomment when Node is implemented
    // let node = Node::new(handle, &arena);
    // let retrieved_data = node.data();
    //
    // assert_eq!(retrieved_data.symbol(), 42);
    // assert_eq!(retrieved_data.byte_range(), (100, 200));
}

// ============================================================================
// Tree<'arena> Integration Tests
// ============================================================================

#[test]
fn tree_lifetime_parameter() {
    // Tree should have 'arena lifetime parameter

    // TODO: Uncomment when Tree<'arena> is implemented
    // use rust_sitter::parser_v4::Tree;
    //
    // let mut arena = TreeArena::new();
    // let root = arena.alloc(TreeNodeData::leaf(1, 0, 10));
    //
    // let tree = Tree {
    //     root,
    //     arena: &arena,
    //     error_count: 0,
    // };
    //
    // // Tree borrows arena with explicit lifetime
    // let _ = tree.root_node();
}

#[test]
fn tree_root_node() {
    // Tree::root_node() should return Node<'arena>

    // TODO: Uncomment when Tree<'arena> and Node are implemented
    // use rust_sitter::parser_v4::Tree;
    //
    // let mut arena = TreeArena::new();
    // let root_data = TreeNodeData::leaf(42, 0, 100);
    // let root_handle = arena.alloc(root_data);
    //
    // let tree = Tree {
    //     root: root_handle,
    //     arena: &arena,
    //     error_count: 0,
    // };
    //
    // let root = tree.root_node();
    // assert_eq!(root.symbol(), 42);
    // assert_eq!(root.byte_range(), (0, 100));
}

#[test]
fn tree_get_node() {
    // Tree::get_node(handle) should return Node<'arena>

    // TODO: Uncomment when Tree<'arena> and Node are implemented
    // use rust_sitter::parser_v4::Tree;
    //
    // let mut arena = TreeArena::new();
    //
    // let child = arena.alloc(TreeNodeData::leaf(5, 0, 10));
    // let root = arena.alloc(TreeNodeData::branch(10, 0, 10, vec![child]));
    //
    // let tree = Tree {
    //     root,
    //     arena: &arena,
    //     error_count: 0,
    // };
    //
    // let child_node = tree.get_node(child);
    // assert_eq!(child_node.symbol(), 5);
}

// ============================================================================
// Iterator Tests
// ============================================================================

#[test]
fn children_iterator_lazy() {
    // children() iterator should be lazy (not allocate until consumed)

    let mut arena = TreeArena::new();
    let child1 = arena.alloc(TreeNodeData::leaf(1, 0, 5));
    let child2 = arena.alloc(TreeNodeData::leaf(2, 5, 10));
    let parent_data = TreeNodeData::branch(10, 0, 10, vec![child1, child2]);
    let parent_handle = arena.alloc(parent_data);

    // TODO: Uncomment when Node is implemented
    // let parent = Node::new(parent_handle, &arena);
    //
    // // Creating iterator shouldn't allocate
    // let iter = parent.children();
    //
    // // Collecting does allocate
    // let children: Vec<_> = iter.collect();
    // assert_eq!(children.len(), 2);
}

#[test]
fn children_iterator_exactsizeiterator() {
    // NodeChildren should implement ExactSizeIterator

    let mut arena = TreeArena::new();
    let child1 = arena.alloc(TreeNodeData::leaf(1, 0, 5));
    let child2 = arena.alloc(TreeNodeData::leaf(2, 5, 10));
    let parent_data = TreeNodeData::branch(10, 0, 10, vec![child1, child2]);
    let parent_handle = arena.alloc(parent_data);

    // TODO: Uncomment when Node is implemented
    // let parent = Node::new(parent_handle, &arena);
    // let iter = parent.children();
    //
    // assert_eq!(iter.len(), 2);
}

// ============================================================================
// Compilation Test: Lifetime Safety
// ============================================================================

/// This test verifies that lifetime system prevents use-after-free.
/// It should compile successfully because we DON'T try to use node after arena drop.
#[test]
fn lifetime_safety_compiles() {
    // This pattern is safe and should compile

    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNodeData::leaf(1, 0, 10));

    // TODO: Uncomment when Node is implemented
    // {
    //     let node = Node::new(handle, &arena);
    //     let _ = node.symbol(); // Use node while arena is alive
    // } // node dropped here
    // // arena still alive

    // This is fine - we don't use node after it's dropped
}

// NOTE: The following test should NOT compile (lifetime error)
// We include it commented out to document the expected behavior
/*
#[test]
fn lifetime_safety_prevents_use_after_free() {
    let node = {
        let mut arena = TreeArena::new();
        let handle = arena.alloc(TreeNodeData::leaf(1, 0, 10));
        Node::new(handle, &arena)
    }; // arena dropped here

    // Compilation error: node.arena doesn't live long enough
    let _ = node.symbol();
}
*/
