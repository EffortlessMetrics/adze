#![allow(unexpected_cfgs)]
//! TreeNodeData Tests
//!
//! This test suite implements all behavioral specifications from
//! docs/specs/TREE_NODE_DATA_SPEC.md
//!
//! Test-Driven Development: These tests define expected behavior before implementation.

use adze::arena_allocator::NodeHandle;
use adze::tree_node_data::TreeNodeData;

// ============================================================================
// Spec 1: Basic Node Creation
// ============================================================================

#[test]
fn spec_1_basic_creation() {
    let node = TreeNodeData::new(42, 0, 10);

    assert_eq!(node.symbol(), 42);
    assert_eq!(node.start_byte(), 0);
    assert_eq!(node.end_byte(), 10);
    assert_eq!(node.child_count(), 0);
}

#[test]
fn spec_1_leaf_creation() {
    let node = TreeNodeData::leaf(5, 10, 20);

    assert_eq!(node.symbol(), 5);
    assert_eq!(node.start_byte(), 10);
    assert_eq!(node.end_byte(), 20);
    assert_eq!(node.byte_len(), 10);
    assert!(node.is_leaf());
}

#[test]
fn spec_1_branch_creation() {
    let children = vec![
        NodeHandle::new(0, 0),
        NodeHandle::new(0, 1),
        NodeHandle::new(0, 2),
    ];

    let node = TreeNodeData::branch(10, 0, 50, children.clone());

    assert_eq!(node.symbol(), 10);
    assert_eq!(node.child_count(), 3);
    assert!(!node.is_leaf());

    for (i, expected) in children.iter().enumerate() {
        assert_eq!(node.child(i), Some(*expected));
    }
}

// ============================================================================
// Spec 2: Child Management
// ============================================================================

#[test]
fn spec_2_child_management() {
    let mut node = TreeNodeData::new(1, 0, 20);

    let child1 = NodeHandle::new(0, 0);
    let child2 = NodeHandle::new(0, 1);

    node.add_child(child1);
    node.add_child(child2);

    assert_eq!(node.child_count(), 2);
    assert_eq!(node.child(0), Some(child1));
    assert_eq!(node.child(1), Some(child2));
}

#[test]
fn spec_2_children_slice() {
    let mut node = TreeNodeData::new(1, 0, 20);

    let handles = vec![
        NodeHandle::new(0, 0),
        NodeHandle::new(0, 1),
        NodeHandle::new(0, 2),
    ];

    for handle in &handles {
        node.add_child(*handle);
    }

    let children = node.children();
    assert_eq!(children.len(), 3);
    assert_eq!(children, handles.as_slice());
}

#[test]
fn spec_2_child_out_of_bounds() {
    let node = TreeNodeData::new(1, 0, 10);

    assert_eq!(node.child(0), None);
    assert_eq!(node.child(100), None);
}

// ============================================================================
// Spec 3: Named Children Tracking
// ============================================================================

#[test]
fn spec_3_named_children() {
    let mut node = TreeNodeData::new(1, 0, 20);

    // Add 2 named children
    node.add_named_child(NodeHandle::new(0, 0));
    node.add_named_child(NodeHandle::new(0, 1));

    // Add 1 unnamed child
    node.add_child(NodeHandle::new(0, 2));

    assert_eq!(node.child_count(), 3);
    assert_eq!(node.named_child_count(), 2);
}

#[test]
fn spec_3_all_named() {
    let mut node = TreeNodeData::new(1, 0, 20);

    for i in 0..5 {
        node.add_named_child(NodeHandle::new(0, i as u32));
    }

    assert_eq!(node.child_count(), 5);
    assert_eq!(node.named_child_count(), 5);
}

#[test]
fn spec_3_no_named() {
    let mut node = TreeNodeData::new(1, 0, 20);

    for i in 0..5 {
        node.add_child(NodeHandle::new(0, i as u32));
    }

    assert_eq!(node.child_count(), 5);
    assert_eq!(node.named_child_count(), 0);
}

// ============================================================================
// Spec 4: Node Flags
// ============================================================================

#[test]
fn spec_4_node_flags() {
    let mut node = TreeNodeData::new(1, 0, 10);

    // Initial state: all flags false
    assert!(!node.is_named());
    assert!(!node.is_error());
    assert!(!node.is_missing());
    assert!(!node.is_extra());

    // Set flags
    node.set_named(true);
    node.set_error(true);

    assert!(node.is_named());
    assert!(node.is_error());
    assert!(!node.is_missing());
    assert!(!node.is_extra());

    // Clear a flag
    node.set_named(false);
    assert!(!node.is_named());
    assert!(node.is_error());
}

#[test]
fn spec_4_all_flags() {
    let mut node = TreeNodeData::new(1, 0, 10);

    node.set_named(true);
    node.set_error(true);
    node.set_missing(true);
    node.set_extra(true);

    assert!(node.is_named());
    assert!(node.is_error());
    assert!(node.is_missing());
    assert!(node.is_extra());
}

#[test]
fn spec_4_flag_independence() {
    let mut node = TreeNodeData::new(1, 0, 10);

    // Set one flag
    node.set_named(true);
    assert!(node.is_named());

    // Set another flag - should not affect first
    node.set_error(true);
    assert!(node.is_named());
    assert!(node.is_error());

    // Clear first flag - should not affect second
    node.set_named(false);
    assert!(!node.is_named());
    assert!(node.is_error());
}

// ============================================================================
// Spec 5: Field Assignment
// ============================================================================

#[test]
fn spec_5_field_assignment() {
    let mut node = TreeNodeData::new(1, 0, 10);

    assert_eq!(node.field_id(), None);

    node.set_field_id(Some(5));
    assert_eq!(node.field_id(), Some(5));

    node.set_field_id(Some(10));
    assert_eq!(node.field_id(), Some(10));

    node.set_field_id(None);
    assert_eq!(node.field_id(), None);
}

#[test]
fn spec_5_field_max_value() {
    let mut node = TreeNodeData::new(1, 0, 10);

    // u16::MAX - 1 should work (MAX reserved for None in niche)
    node.set_field_id(Some(u16::MAX - 1));
    assert_eq!(node.field_id(), Some(u16::MAX - 1));
}

// ============================================================================
// Spec 6: Memory Layout
// ============================================================================

#[test]
fn spec_6_memory_layout() {
    use std::mem;

    let size = mem::size_of::<TreeNodeData>();
    assert!(size <= 64, "TreeNodeData is {} bytes, must be ≤64", size);

    // Verify alignment
    let align = mem::align_of::<TreeNodeData>();
    assert_eq!(align, 8, "TreeNodeData should be 8-byte aligned");
}

#[test]
fn spec_6_node_handle_size() {
    use std::mem;

    // Verify NodeHandle is 8 bytes as expected
    assert_eq!(mem::size_of::<NodeHandle>(), 8);
}

// ============================================================================
// Spec 7: SmallVec Optimization
// ============================================================================

#[test]
fn spec_7_smallvec_inline() {
    let mut node = TreeNodeData::new(1, 0, 20);

    // Add 3 children (should stay inline)
    node.add_child(NodeHandle::new(0, 0));
    node.add_child(NodeHandle::new(0, 1));
    node.add_child(NodeHandle::new(0, 2));

    assert_eq!(node.child_count(), 3);

    // All children accessible
    assert_eq!(node.child(0), Some(NodeHandle::new(0, 0)));
    assert_eq!(node.child(1), Some(NodeHandle::new(0, 1)));
    assert_eq!(node.child(2), Some(NodeHandle::new(0, 2)));
}

#[test]
fn spec_7_smallvec_spill() {
    let mut node = TreeNodeData::new(1, 0, 20);

    // Add 5 children (should spill to heap)
    for i in 0..5 {
        node.add_child(NodeHandle::new(0, i as u32));
    }

    assert_eq!(node.child_count(), 5);

    // All children accessible
    for i in 0..5 {
        assert_eq!(node.child(i).unwrap(), NodeHandle::new(0, i as u32));
    }
}

#[test]
fn spec_7_large_child_count() {
    let mut node = TreeNodeData::new(1, 0, 100);

    // Add many children
    for i in 0..20 {
        node.add_child(NodeHandle::new(0, i as u32));
    }

    assert_eq!(node.child_count(), 20);

    // Spot check access
    assert_eq!(node.child(0), Some(NodeHandle::new(0, 0)));
    assert_eq!(node.child(10), Some(NodeHandle::new(0, 10)));
    assert_eq!(node.child(19), Some(NodeHandle::new(0, 19)));
    assert_eq!(node.child(20), None);
}

// ============================================================================
// Byte Range Methods
// ============================================================================

#[test]
fn test_byte_range() {
    let node = TreeNodeData::new(1, 10, 50);

    assert_eq!(node.byte_range(), (10, 50));
    assert_eq!(node.byte_len(), 40);
}

#[test]
fn test_zero_length_node() {
    let node = TreeNodeData::new(1, 10, 10);

    assert_eq!(node.byte_len(), 0);
    assert!(node.byte_range().0 == node.byte_range().1);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_node() {
    let node = TreeNodeData::new(0, 0, 0);

    assert_eq!(node.symbol(), 0);
    assert_eq!(node.start_byte(), 0);
    assert_eq!(node.end_byte(), 0);
    assert_eq!(node.child_count(), 0);
    assert_eq!(node.named_child_count(), 0);
}

#[test]
fn test_max_symbol() {
    let node = TreeNodeData::new(u16::MAX, 0, 10);

    assert_eq!(node.symbol(), u16::MAX);
}

#[test]
fn test_max_byte_position() {
    let node = TreeNodeData::new(1, u32::MAX - 10, u32::MAX);

    assert_eq!(node.start_byte(), u32::MAX - 10);
    assert_eq!(node.end_byte(), u32::MAX);
    assert_eq!(node.byte_len(), 10);
}

// ============================================================================
// Mixed Operations
// ============================================================================

#[test]
fn test_complex_node() {
    let mut node = TreeNodeData::new(42, 100, 200);

    node.set_named(true);
    node.set_field_id(Some(7));
    node.add_named_child(NodeHandle::new(0, 0));
    node.add_named_child(NodeHandle::new(0, 1));
    node.add_child(NodeHandle::new(0, 2));

    assert_eq!(node.symbol(), 42);
    assert_eq!(node.byte_range(), (100, 200));
    assert_eq!(node.byte_len(), 100);
    assert_eq!(node.child_count(), 3);
    assert_eq!(node.named_child_count(), 2);
    assert_eq!(node.field_id(), Some(7));
    assert!(node.is_named());
    assert!(!node.is_error());
}

// ============================================================================
// Property Tests (if proptest is available)
// ============================================================================

#[allow(unexpected_cfgs)]
#[cfg(feature = "proptest")]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_child_access(children in vec(any::<(u32, u32)>(), 0..10)) {
            let mut node = TreeNodeData::new(1, 0, 100);
            let handles: Vec<_> = children.iter().map(|(c, n)| {
                NodeHandle::new(*c, *n)
            }).collect();

            for handle in &handles {
                node.add_child(*handle);
            }

            assert_eq!(node.child_count(), handles.len());
            for (i, handle) in handles.iter().enumerate() {
                assert_eq!(node.child(i), Some(*handle));
            }
        }

        #[test]
        fn prop_byte_range(symbol in any::<u16>(), start in 0u32..1_000_000, len in 0u32..10_000) {
            let end = start.saturating_add(len);
            let node = TreeNodeData::new(symbol, start, end);

            assert_eq!(node.symbol(), symbol);
            assert_eq!(node.start_byte(), start);
            assert_eq!(node.end_byte(), end);
            assert_eq!(node.byte_len(), end - start);
        }

        #[test]
        fn prop_flags_independence(named in any::<bool>(), error in any::<bool>(),
                                     missing in any::<bool>(), extra in any::<bool>()) {
            let mut node = TreeNodeData::new(1, 0, 10);

            node.set_named(named);
            node.set_error(error);
            node.set_missing(missing);
            node.set_extra(extra);

            assert_eq!(node.is_named(), named);
            assert_eq!(node.is_error(), error);
            assert_eq!(node.is_missing(), missing);
            assert_eq!(node.is_extra(), extra);
        }
    }
}
