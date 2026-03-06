//! Node API v6 — comprehensive tests for TreeNodeData
//!
//! Covers 8 categories × 8 tests = 64 tests:
//! 1. TreeNodeData::leaf construction and properties
//! 2. TreeNodeData::branch construction and properties
//! 3. Field access and mutation (flags, field_id)
//! 4. Byte-range–based source text extraction
//! 5. Child management (add, index, named)
//! 6. Byte range arithmetic
//! 7. Clone/Debug/equality semantics
//! 8. Edge cases: zero-length, max values, saturation

use adze::arena_allocator::NodeHandle;
use adze::tree_node_data::TreeNodeData;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Shorthand for a leaf node.
fn mk_leaf(sym: u16, start: u32, end: u32) -> TreeNodeData {
    TreeNodeData::leaf(sym, start, end)
}

/// Shorthand for a branch node with pre-built handles.
fn mk_branch(sym: u16, start: u32, end: u32, handles: Vec<NodeHandle>) -> TreeNodeData {
    TreeNodeData::branch(sym, start, end, handles)
}

/// Create `n` distinct NodeHandle values for testing.
fn handles(n: u32) -> Vec<NodeHandle> {
    (0..n).map(|i| NodeHandle::new(0, i)).collect()
}

// ===========================================================================
// 1. Leaf construction and properties
// ===========================================================================

#[test]
fn test_leaf_symbol_returns_correct_value() {
    let node = mk_leaf(42, 0, 10);
    assert_eq!(node.symbol(), 42);
}

#[test]
fn test_leaf_start_byte() {
    let node = mk_leaf(1, 5, 15);
    assert_eq!(node.start_byte(), 5);
}

#[test]
fn test_leaf_end_byte() {
    let node = mk_leaf(1, 5, 15);
    assert_eq!(node.end_byte(), 15);
}

#[test]
fn test_leaf_is_leaf_returns_true() {
    let node = mk_leaf(1, 0, 1);
    assert!(node.is_leaf());
}

#[test]
fn test_leaf_child_count_is_zero() {
    let node = mk_leaf(1, 0, 1);
    assert_eq!(node.child_count(), 0);
}

#[test]
fn test_leaf_named_child_count_is_zero() {
    let node = mk_leaf(1, 0, 1);
    assert_eq!(node.named_child_count(), 0);
}

#[test]
fn test_leaf_default_flags_all_false() {
    let node = mk_leaf(7, 0, 5);
    assert!(!node.is_named());
    assert!(!node.is_error());
    assert!(!node.is_missing());
    assert!(!node.is_extra());
}

#[test]
fn test_leaf_field_id_defaults_to_none() {
    let node = mk_leaf(1, 0, 1);
    assert!(node.field_id().is_none());
}

// ===========================================================================
// 2. Branch construction and properties
// ===========================================================================

#[test]
fn test_branch_symbol_returns_correct_value() {
    let node = mk_branch(99, 0, 50, handles(2));
    assert_eq!(node.symbol(), 99);
}

#[test]
fn test_branch_is_not_leaf() {
    let node = mk_branch(1, 0, 10, handles(1));
    assert!(!node.is_leaf());
}

#[test]
fn test_branch_child_count_matches_input() {
    let node = mk_branch(1, 0, 10, handles(5));
    assert_eq!(node.child_count(), 5);
}

#[test]
fn test_branch_children_returns_correct_handles() {
    let h = handles(3);
    let node = mk_branch(1, 0, 10, h.clone());
    assert_eq!(node.children(), &h[..]);
}

#[test]
fn test_branch_with_empty_children_is_leaf() {
    let node = mk_branch(1, 0, 10, vec![]);
    assert!(node.is_leaf());
}

#[test]
fn test_branch_byte_range_tuple() {
    let node = mk_branch(1, 10, 200, handles(2));
    assert_eq!(node.byte_range(), (10, 200));
}

#[test]
fn test_branch_default_flags_all_false() {
    let node = mk_branch(1, 0, 10, handles(2));
    assert!(!node.is_named());
    assert!(!node.is_error());
    assert!(!node.is_missing());
    assert!(!node.is_extra());
}

#[test]
fn test_branch_field_id_defaults_to_none() {
    let node = mk_branch(1, 0, 10, handles(1));
    assert!(node.field_id().is_none());
}

// ===========================================================================
// 3. Field access and mutation
// ===========================================================================

#[test]
fn test_set_named_flag_true() {
    let mut node = mk_leaf(1, 0, 5);
    node.set_named(true);
    assert!(node.is_named());
}

#[test]
fn test_set_error_flag_true() {
    let mut node = mk_leaf(1, 0, 5);
    node.set_error(true);
    assert!(node.is_error());
}

#[test]
fn test_set_missing_flag_true() {
    let mut node = mk_leaf(1, 0, 5);
    node.set_missing(true);
    assert!(node.is_missing());
}

#[test]
fn test_set_extra_flag_true() {
    let mut node = mk_leaf(1, 0, 5);
    node.set_extra(true);
    assert!(node.is_extra());
}

#[test]
fn test_set_field_id_some_value() {
    let mut node = mk_leaf(1, 0, 5);
    node.set_field_id(Some(100));
    assert_eq!(node.field_id(), Some(100));
}

#[test]
fn test_set_field_id_back_to_none() {
    let mut node = mk_leaf(1, 0, 5);
    node.set_field_id(Some(7));
    node.set_field_id(None);
    assert!(node.field_id().is_none());
}

#[test]
fn test_toggle_named_flag_off() {
    let mut node = mk_leaf(1, 0, 5);
    node.set_named(true);
    assert!(node.is_named());
    node.set_named(false);
    assert!(!node.is_named());
}

#[test]
fn test_multiple_flags_are_independent() {
    let mut node = mk_leaf(1, 0, 5);
    node.set_named(true);
    node.set_error(true);
    node.set_extra(true);
    assert!(node.is_named());
    assert!(node.is_error());
    assert!(!node.is_missing());
    assert!(node.is_extra());

    node.set_error(false);
    assert!(node.is_named());
    assert!(!node.is_error());
    assert!(node.is_extra());
}

// ===========================================================================
// 4. Source text extraction via byte ranges
// ===========================================================================

#[test]
fn test_byte_range_slices_source_correctly() {
    let source = "hello world";
    let node = mk_leaf(1, 0, 5);
    let text = &source[node.start_byte() as usize..node.end_byte() as usize];
    assert_eq!(text, "hello");
}

#[test]
fn test_byte_range_mid_source_extraction() {
    let source = "hello world";
    let node = mk_leaf(1, 6, 11);
    let text = &source[node.start_byte() as usize..node.end_byte() as usize];
    assert_eq!(text, "world");
}

#[test]
fn test_byte_len_matches_extracted_text_length() {
    let source = "abcdef";
    let node = mk_leaf(1, 1, 4);
    let text = &source[node.start_byte() as usize..node.end_byte() as usize];
    assert_eq!(node.byte_len() as usize, text.len());
}

#[test]
fn test_leaf_byte_range_for_single_char() {
    let source = "x";
    let node = mk_leaf(1, 0, 1);
    let text = &source[node.start_byte() as usize..node.end_byte() as usize];
    assert_eq!(text, "x");
    assert_eq!(node.byte_len(), 1);
}

#[test]
fn test_branch_byte_range_spans_children_text() {
    let source = "fn main() {}";
    let node = mk_branch(1, 0, 12, handles(3));
    let text = &source[node.start_byte() as usize..node.end_byte() as usize];
    assert_eq!(text, "fn main() {}");
}

#[test]
fn test_zero_length_range_extracts_empty_string() {
    let source = "abc";
    let node = mk_leaf(1, 2, 2);
    let text = &source[node.start_byte() as usize..node.end_byte() as usize];
    assert!(text.is_empty());
}

#[test]
fn test_multibyte_utf8_byte_range() {
    let source = "café";
    // 'é' is 2 bytes in UTF-8; "café" = [99, 97, 102, 195, 169]
    let node = mk_leaf(1, 0, 5);
    let text = &source[node.start_byte() as usize..node.end_byte() as usize];
    assert_eq!(text, "café");
}

#[test]
fn test_byte_range_tuple_matches_accessors() {
    let node = mk_leaf(1, 10, 20);
    let (s, e) = node.byte_range();
    assert_eq!(s, node.start_byte());
    assert_eq!(e, node.end_byte());
}

// ===========================================================================
// 5. Child management
// ===========================================================================

#[test]
fn test_add_child_increases_count() {
    let mut node = mk_leaf(1, 0, 10);
    let h = NodeHandle::new(0, 0);
    node.add_child(h);
    assert_eq!(node.child_count(), 1);
}

#[test]
fn test_add_named_child_increments_named_count() {
    let mut node = mk_leaf(1, 0, 10);
    let h = NodeHandle::new(0, 0);
    node.add_named_child(h);
    assert_eq!(node.named_child_count(), 1);
    assert_eq!(node.child_count(), 1);
}

#[test]
fn test_add_multiple_children_preserves_order() {
    let mut node = mk_leaf(1, 0, 10);
    let h0 = NodeHandle::new(0, 0);
    let h1 = NodeHandle::new(0, 1);
    let h2 = NodeHandle::new(0, 2);
    node.add_child(h0);
    node.add_child(h1);
    node.add_child(h2);
    assert_eq!(node.children()[0], h0);
    assert_eq!(node.children()[1], h1);
    assert_eq!(node.children()[2], h2);
}

#[test]
fn test_child_by_index_returns_correct_handle() {
    let h = handles(3);
    let node = mk_branch(1, 0, 10, h.clone());
    assert_eq!(node.child(0), Some(h[0]));
    assert_eq!(node.child(1), Some(h[1]));
    assert_eq!(node.child(2), Some(h[2]));
}

#[test]
fn test_child_by_index_out_of_bounds_returns_none() {
    let node = mk_branch(1, 0, 10, handles(2));
    assert!(node.child(2).is_none());
    assert!(node.child(100).is_none());
}

#[test]
fn test_children_slice_on_leaf_is_empty() {
    let node = mk_leaf(1, 0, 10);
    assert!(node.children().is_empty());
}

#[test]
fn test_add_child_to_leaf_converts_to_branch() {
    let mut node = mk_leaf(1, 0, 10);
    assert!(node.is_leaf());
    node.add_child(NodeHandle::new(0, 0));
    assert!(!node.is_leaf());
}

#[test]
fn test_mixed_named_and_unnamed_children() {
    let mut node = mk_leaf(1, 0, 10);
    node.add_child(NodeHandle::new(0, 0));
    node.add_named_child(NodeHandle::new(0, 1));
    node.add_child(NodeHandle::new(0, 2));
    node.add_named_child(NodeHandle::new(0, 3));
    assert_eq!(node.child_count(), 4);
    assert_eq!(node.named_child_count(), 2);
}

// ===========================================================================
// 6. Byte range arithmetic
// ===========================================================================

#[test]
fn test_byte_len_positive_range() {
    let node = mk_leaf(1, 10, 30);
    assert_eq!(node.byte_len(), 20);
}

#[test]
fn test_byte_len_zero_range() {
    let node = mk_leaf(1, 5, 5);
    assert_eq!(node.byte_len(), 0);
}

#[test]
fn test_byte_len_saturating_does_not_underflow() {
    // byte_len uses saturating_sub so end < start → 0
    let node = TreeNodeData::new(1, 10, 5);
    assert_eq!(node.byte_len(), 0);
}

#[test]
fn test_byte_range_at_file_start() {
    let node = mk_leaf(1, 0, 100);
    assert_eq!(node.start_byte(), 0);
    assert_eq!(node.byte_len(), 100);
}

#[test]
fn test_byte_range_single_byte() {
    let node = mk_leaf(1, 99, 100);
    assert_eq!(node.byte_len(), 1);
}

#[test]
fn test_byte_range_large_file() {
    let node = mk_leaf(1, 0, 1_000_000);
    assert_eq!(node.byte_len(), 1_000_000);
}

#[test]
fn test_byte_range_consecutive_nodes() {
    let a = mk_leaf(1, 0, 10);
    let b = mk_leaf(2, 10, 25);
    assert_eq!(a.end_byte(), b.start_byte());
    assert_eq!(a.byte_len() + b.byte_len(), 25);
}

#[test]
fn test_byte_range_with_gap_between_nodes() {
    let a = mk_leaf(1, 0, 10);
    let b = mk_leaf(2, 15, 25);
    let gap = b.start_byte() - a.end_byte();
    assert_eq!(gap, 5);
}

// ===========================================================================
// 7. Clone, Debug, and equality semantics
// ===========================================================================

#[test]
fn test_clone_preserves_symbol() {
    let original = mk_leaf(42, 0, 10);
    let cloned = original.clone();
    assert_eq!(cloned.symbol(), 42);
}

#[test]
fn test_clone_preserves_byte_range() {
    let original = mk_leaf(1, 5, 15);
    let cloned = original.clone();
    assert_eq!(cloned.byte_range(), (5, 15));
}

#[test]
fn test_clone_preserves_children() {
    let h = handles(3);
    let original = mk_branch(1, 0, 10, h.clone());
    let cloned = original.clone();
    assert_eq!(cloned.child_count(), 3);
    assert_eq!(cloned.children(), original.children());
}

#[test]
fn test_clone_preserves_flags() {
    let mut original = mk_leaf(1, 0, 5);
    original.set_named(true);
    original.set_error(true);
    let cloned = original.clone();
    assert!(cloned.is_named());
    assert!(cloned.is_error());
    assert!(!cloned.is_missing());
}

#[test]
fn test_clone_preserves_field_id() {
    let mut original = mk_leaf(1, 0, 5);
    original.set_field_id(Some(77));
    let cloned = original.clone();
    assert_eq!(cloned.field_id(), Some(77));
}

#[test]
fn test_debug_format_contains_symbol_value() {
    let node = mk_leaf(99, 0, 5);
    let debug_str = format!("{node:?}");
    assert!(debug_str.contains("99"), "Debug output: {debug_str}");
}

#[test]
fn test_node_handle_equality() {
    let a = NodeHandle::new(1, 2);
    let b = NodeHandle::new(1, 2);
    let c = NodeHandle::new(1, 3);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn test_node_handle_copy_semantics() {
    let original = NodeHandle::new(5, 10);
    let copied = original; // Copy, not move
    assert_eq!(original, copied); // original still usable
}

// ===========================================================================
// 8. Edge cases: zero-length, max values, empty, saturation
// ===========================================================================

#[test]
fn test_zero_length_range_is_leaf() {
    let node = mk_leaf(1, 100, 100);
    assert_eq!(node.byte_len(), 0);
    assert!(node.is_leaf());
}

#[test]
fn test_max_symbol_value() {
    let node = mk_leaf(u16::MAX, 0, 1);
    assert_eq!(node.symbol(), u16::MAX);
}

#[test]
fn test_max_byte_values() {
    let node = mk_leaf(1, u32::MAX - 1, u32::MAX);
    assert_eq!(node.start_byte(), u32::MAX - 1);
    assert_eq!(node.end_byte(), u32::MAX);
    assert_eq!(node.byte_len(), 1);
}

#[test]
fn test_zero_symbol() {
    let node = mk_leaf(0, 0, 0);
    assert_eq!(node.symbol(), 0);
    assert_eq!(node.byte_len(), 0);
}

#[test]
fn test_empty_children_vec_for_branch() {
    let node = mk_branch(1, 0, 10, vec![]);
    assert!(node.children().is_empty());
    assert!(node.is_leaf());
    assert_eq!(node.child_count(), 0);
}

#[test]
fn test_field_id_max_value() {
    let mut node = mk_leaf(1, 0, 5);
    node.set_field_id(Some(u16::MAX));
    assert_eq!(node.field_id(), Some(u16::MAX));
}

#[test]
fn test_named_child_count_saturation() {
    let mut node = mk_leaf(1, 0, 5);
    // Add many named children — count uses u16 internally with saturating_add
    for i in 0..10 {
        node.add_named_child(NodeHandle::new(0, i));
    }
    assert_eq!(node.named_child_count(), 10);
    assert_eq!(node.child_count(), 10);
}

#[test]
fn test_many_children_beyond_smallvec_inline() {
    // SmallVec<[NodeHandle; 3]> stores 3 inline; more spills to heap
    let h = handles(20);
    let node = mk_branch(1, 0, 100, h.clone());
    assert_eq!(node.child_count(), 20);
    for (i, handle) in h.iter().enumerate() {
        assert_eq!(node.child(i), Some(*handle));
    }
}
