#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for the Node API in adze-runtime (runtime2).

use adze_runtime::tree::Tree;

// ---------------------------------------------------------------------------
// Helper: build a leaf tree (no children)
// ---------------------------------------------------------------------------
fn leaf(symbol: u32, start: usize, end: usize) -> Tree {
    Tree::new_for_testing(symbol, start, end, vec![])
}

// ---------------------------------------------------------------------------
// Helper: build a tree with children
// ---------------------------------------------------------------------------
fn branch(symbol: u32, start: usize, end: usize, children: Vec<Tree>) -> Tree {
    Tree::new_for_testing(symbol, start, end, children)
}

// ===== 1. Stub tree basics =====

#[test]
fn stub_root_kind_id_is_zero() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.kind_id(), 0);
}

#[test]
fn stub_root_byte_range_is_empty() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
    assert_eq!(root.byte_range(), 0..0);
}

#[test]
fn stub_root_has_no_children() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.child_count(), 0);
    assert_eq!(root.named_child_count(), 0);
}

#[test]
fn stub_child_out_of_bounds_returns_none() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert!(root.child(0).is_none());
    assert!(root.child(100).is_none());
}

// ===== 2. kind / kind_id =====

#[test]
fn kind_returns_unknown_without_language() {
    let tree = leaf(42, 0, 5);
    let root = tree.root_node();
    assert_eq!(root.kind(), "unknown");
}

#[test]
fn kind_id_matches_symbol() {
    let tree = leaf(7, 0, 3);
    assert_eq!(tree.root_node().kind_id(), 7);
}

#[test]
fn kind_id_truncates_large_symbol() {
    // symbol stored as u32 but kind_id returns u16
    let tree = leaf(300, 0, 1);
    assert_eq!(tree.root_node().kind_id(), 300u16);
}

// ===== 3. Byte positions =====

#[test]
fn start_end_byte_on_leaf() {
    let tree = leaf(1, 10, 20);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 10);
    assert_eq!(root.end_byte(), 20);
}

#[test]
fn byte_range_consistency() {
    let tree = leaf(1, 5, 15);
    let root = tree.root_node();
    assert_eq!(root.byte_range(), root.start_byte()..root.end_byte());
}

#[test]
fn zero_width_node() {
    let tree = leaf(1, 7, 7);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), root.end_byte());
    assert!(root.byte_range().is_empty());
}

// ===== 4. Positions (currently dummy) =====

#[test]
fn start_position_is_origin() {
    let tree = leaf(1, 100, 200);
    let pos = tree.root_node().start_position();
    assert_eq!(pos.row, 0);
    assert_eq!(pos.column, 0);
}

#[test]
fn end_position_is_origin() {
    let tree = leaf(1, 100, 200);
    let pos = tree.root_node().end_position();
    assert_eq!(pos.row, 0);
    assert_eq!(pos.column, 0);
}

// ===== 5. Boolean flags =====

#[test]
fn is_named_always_true() {
    let tree = leaf(1, 0, 5);
    assert!(tree.root_node().is_named());
}

#[test]
fn is_missing_always_false() {
    let tree = leaf(1, 0, 5);
    assert!(!tree.root_node().is_missing());
}

#[test]
fn is_error_always_false() {
    let tree = leaf(1, 0, 5);
    assert!(!tree.root_node().is_error());
}

#[test]
fn boolean_flags_consistent_across_tree() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let root = tree.root_node();
    assert!(root.is_named());
    assert!(!root.is_missing());
    assert!(!root.is_error());
    for i in 0..root.child_count() {
        let child = root.child(i).unwrap();
        assert!(child.is_named());
        assert!(!child.is_missing());
        assert!(!child.is_error());
    }
}

// ===== 6. child_count / named_child_count =====

#[test]
fn child_count_leaf() {
    let tree = leaf(1, 0, 5);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn child_count_with_children() {
    let tree = branch(
        0,
        0,
        15,
        vec![leaf(1, 0, 5), leaf(2, 5, 10), leaf(3, 10, 15)],
    );
    assert_eq!(tree.root_node().child_count(), 3);
}

#[test]
fn named_child_count_equals_child_count() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

// ===== 7. child() access =====

#[test]
fn child_returns_correct_symbols() {
    let tree = branch(
        0,
        0,
        9,
        vec![leaf(10, 0, 3), leaf(20, 3, 6), leaf(30, 6, 9)],
    );
    let root = tree.root_node();
    assert_eq!(root.child(0).unwrap().kind_id(), 10);
    assert_eq!(root.child(1).unwrap().kind_id(), 20);
    assert_eq!(root.child(2).unwrap().kind_id(), 30);
}

#[test]
fn child_returns_none_past_end() {
    let tree = branch(0, 0, 5, vec![leaf(1, 0, 5)]);
    let root = tree.root_node();
    assert!(root.child(0).is_some());
    assert!(root.child(1).is_none());
    assert!(root.child(usize::MAX).is_none());
}

#[test]
fn child_byte_ranges_preserved() {
    let tree = branch(0, 0, 20, vec![leaf(1, 0, 8), leaf(2, 8, 20)]);
    let root = tree.root_node();
    let c0 = root.child(0).unwrap();
    let c1 = root.child(1).unwrap();
    assert_eq!(c0.start_byte(), 0);
    assert_eq!(c0.end_byte(), 8);
    assert_eq!(c1.start_byte(), 8);
    assert_eq!(c1.end_byte(), 20);
}

// ===== 8. named_child() =====

#[test]
fn named_child_same_as_child() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let root = tree.root_node();
    for i in 0..root.child_count() {
        let c = root.child(i).unwrap();
        let nc = root.named_child(i).unwrap();
        assert_eq!(c.kind_id(), nc.kind_id());
        assert_eq!(c.start_byte(), nc.start_byte());
    }
}

#[test]
fn named_child_none_past_end() {
    let tree = leaf(1, 0, 5);
    assert!(tree.root_node().named_child(0).is_none());
}

// ===== 9. child_by_field_name =====

#[test]
fn child_by_field_name_always_none() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let root = tree.root_node();
    assert!(root.child_by_field_name("left").is_none());
    assert!(root.child_by_field_name("right").is_none());
    assert!(root.child_by_field_name("").is_none());
}

// ===== 10. parent / siblings =====

#[test]
fn parent_always_none() {
    let tree = branch(0, 0, 5, vec![leaf(1, 0, 5)]);
    assert!(tree.root_node().parent().is_none());
    assert!(tree.root_node().child(0).unwrap().parent().is_none());
}

#[test]
fn next_sibling_always_none() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let c0 = tree.root_node().child(0).unwrap();
    assert!(c0.next_sibling().is_none());
}

#[test]
fn prev_sibling_always_none() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let c1 = tree.root_node().child(1).unwrap();
    assert!(c1.prev_sibling().is_none());
}

// ===== 11. utf8_text =====

#[test]
fn utf8_text_extracts_slice() {
    let source = b"hello world";
    let tree = branch(0, 0, 11, vec![leaf(1, 0, 5), leaf(2, 6, 11)]);
    let root = tree.root_node();
    assert_eq!(root.utf8_text(source).unwrap(), "hello world");
    assert_eq!(root.child(0).unwrap().utf8_text(source).unwrap(), "hello");
    assert_eq!(root.child(1).unwrap().utf8_text(source).unwrap(), "world");
}

#[test]
fn utf8_text_empty_range() {
    let source = b"abc";
    let tree = leaf(1, 2, 2);
    assert_eq!(tree.root_node().utf8_text(source).unwrap(), "");
}

#[test]
fn utf8_text_unicode() {
    let source = "café".as_bytes(); // 'é' is 2 bytes in UTF-8
    let tree = leaf(1, 0, source.len());
    assert_eq!(tree.root_node().utf8_text(source).unwrap(), "café");
}

#[test]
fn utf8_text_invalid_utf8() {
    let source: &[u8] = &[0xff, 0xfe, 0xfd];
    let tree = leaf(1, 0, 3);
    assert!(tree.root_node().utf8_text(source).is_err());
}

// ===== 12. Deep / nested trees =====

#[test]
fn nested_children_preserved() {
    // grandchild structure: root -> child -> grandchild
    let grandchild = leaf(3, 0, 2);
    let child = branch(2, 0, 2, vec![grandchild]);
    let tree = branch(1, 0, 2, vec![child]);

    let root = tree.root_node();
    assert_eq!(root.child_count(), 1);
    let c = root.child(0).unwrap();
    assert_eq!(c.kind_id(), 2);
    assert_eq!(c.child_count(), 1);
    let gc = c.child(0).unwrap();
    assert_eq!(gc.kind_id(), 3);
    assert_eq!(gc.child_count(), 0);
}

#[test]
fn deeply_nested_tree() {
    // Build a 10-level deep tree
    let mut current = leaf(10, 0, 1);
    for sym in (0..10).rev() {
        current = branch(sym, 0, 1, vec![current]);
    }
    let root = current.root_node();
    assert_eq!(root.kind_id(), 0);

    // Walk down 10 levels
    let mut node = root;
    for expected_sym in 0..=10 {
        assert_eq!(node.kind_id(), expected_sym as u16);
        if expected_sym < 10 {
            assert_eq!(node.child_count(), 1);
            node = node.child(0).unwrap();
        } else {
            assert_eq!(node.child_count(), 0);
        }
    }
}

// ===== 13. Wide trees =====

#[test]
fn wide_tree_many_children() {
    let children: Vec<Tree> = (0..50)
        .map(|i| leaf(i + 1, i as usize, (i + 1) as usize))
        .collect();
    let tree = branch(0, 0, 50, children);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 50);
    for i in 0..50 {
        let c = root.child(i).unwrap();
        assert_eq!(c.kind_id(), (i + 1) as u16);
        assert_eq!(c.start_byte(), i);
        assert_eq!(c.end_byte(), i + 1);
    }
}

// ===== 14. Debug formatting =====

#[test]
fn debug_format_contains_kind_and_range() {
    let tree = leaf(5, 10, 20);
    let dbg = format!("{:?}", tree.root_node());
    assert!(dbg.contains("Node"));
    assert!(dbg.contains("10..20"));
}

#[test]
fn debug_format_stub() {
    let tree = Tree::new_stub();
    let dbg = format!("{:?}", tree.root_node());
    assert!(dbg.contains("0..0"));
}

// ===== 15. Copy semantics =====

#[test]
fn node_is_copy() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let root = tree.root_node();
    let copy = root; // Node is Copy
    assert_eq!(root.kind_id(), copy.kind_id());
    assert_eq!(root.start_byte(), copy.start_byte());
    assert_eq!(root.child_count(), copy.child_count());
}

// ===== 16. Tree clone independence =====

#[test]
fn cloned_tree_nodes_independent() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5)]);
    let cloned = tree.clone();
    // Both yield the same data
    assert_eq!(tree.root_node().kind_id(), cloned.root_node().kind_id());
    assert_eq!(
        tree.root_node().child_count(),
        cloned.root_node().child_count()
    );
}

// ===== 17. Multiple children with same symbol =====

#[test]
fn children_with_duplicate_symbols() {
    let tree = branch(0, 0, 9, vec![leaf(5, 0, 3), leaf(5, 3, 6), leaf(5, 6, 9)]);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 3);
    for i in 0..3 {
        assert_eq!(root.child(i).unwrap().kind_id(), 5);
    }
    // Distinguished by byte range
    assert_eq!(root.child(0).unwrap().start_byte(), 0);
    assert_eq!(root.child(1).unwrap().start_byte(), 3);
    assert_eq!(root.child(2).unwrap().start_byte(), 6);
}

// ===== 18. utf8_text on children of a sentence =====

#[test]
fn utf8_text_children_cover_source() {
    let source = b"fn main()";
    let tree = branch(
        0,
        0,
        9,
        vec![leaf(1, 0, 2), leaf(2, 3, 7), leaf(3, 7, 8), leaf(4, 8, 9)],
    );
    let root = tree.root_node();
    assert_eq!(root.child(0).unwrap().utf8_text(source).unwrap(), "fn");
    assert_eq!(root.child(1).unwrap().utf8_text(source).unwrap(), "main");
    assert_eq!(root.child(2).unwrap().utf8_text(source).unwrap(), "(");
    assert_eq!(root.child(3).unwrap().utf8_text(source).unwrap(), ")");
}

// ===== 19. new_for_testing flattens root children =====

#[test]
fn new_for_testing_flattens_children() {
    let inner = branch(2, 5, 10, vec![leaf(3, 5, 7), leaf(4, 7, 10)]);
    let tree = Tree::new_for_testing(1, 0, 15, vec![leaf(5, 0, 5), inner]);

    let root = tree.root_node();
    assert_eq!(root.kind_id(), 1);
    assert_eq!(root.child_count(), 2);

    let c0 = root.child(0).unwrap();
    assert_eq!(c0.kind_id(), 5);
    assert_eq!(c0.child_count(), 0);

    let c1 = root.child(1).unwrap();
    assert_eq!(c1.kind_id(), 2);
    assert_eq!(c1.child_count(), 2);
    assert_eq!(c1.child(0).unwrap().kind_id(), 3);
    assert_eq!(c1.child(1).unwrap().kind_id(), 4);
}

// ===== 20. Edge case: very large byte ranges =====

#[test]
fn large_byte_range() {
    let big = usize::MAX / 2;
    let tree = leaf(1, big, big + 100);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), big);
    assert_eq!(root.end_byte(), big + 100);
    assert_eq!(root.byte_range(), big..big + 100);
}

// ===== 21. Tree::new_stub vs new_for_testing equivalence =====

#[test]
fn stub_equivalent_to_empty_for_testing() {
    let stub = Tree::new_stub();
    let manual = Tree::new_for_testing(0, 0, 0, vec![]);
    assert_eq!(stub.root_node().kind_id(), manual.root_node().kind_id());
    assert_eq!(
        stub.root_node().start_byte(),
        manual.root_node().start_byte()
    );
    assert_eq!(stub.root_node().end_byte(), manual.root_node().end_byte());
    assert_eq!(
        stub.root_node().child_count(),
        manual.root_node().child_count()
    );
}

// ===== 22. Grandchild utf8_text =====

#[test]
fn grandchild_utf8_text() {
    let source = b"(1+2)";
    let tree = branch(
        0,
        0,
        5,
        vec![branch(
            1,
            0,
            5,
            vec![
                leaf(2, 0, 1),
                leaf(3, 1, 2),
                leaf(4, 2, 3),
                leaf(5, 3, 4),
                leaf(6, 4, 5),
            ],
        )],
    );
    let inner = tree.root_node().child(0).unwrap();
    assert_eq!(inner.child(0).unwrap().utf8_text(source).unwrap(), "(");
    assert_eq!(inner.child(1).unwrap().utf8_text(source).unwrap(), "1");
    assert_eq!(inner.child(2).unwrap().utf8_text(source).unwrap(), "+");
    assert_eq!(inner.child(3).unwrap().utf8_text(source).unwrap(), "2");
    assert_eq!(inner.child(4).unwrap().utf8_text(source).unwrap(), ")");
}

// ===== 23. Single-child tree =====

#[test]
fn single_child_tree() {
    let tree = branch(0, 0, 5, vec![leaf(1, 0, 5)]);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 1);
    let child = root.child(0).unwrap();
    assert_eq!(child.kind_id(), 1);
    assert_eq!(child.start_byte(), 0);
    assert_eq!(child.end_byte(), 5);
    assert_eq!(child.child_count(), 0);
}

// ===== 24. Mixed depth tree =====

#[test]
fn mixed_depth_children() {
    // root has a deep child and a shallow leaf
    let deep = branch(1, 0, 3, vec![branch(2, 0, 2, vec![leaf(3, 0, 1)])]);
    let shallow = leaf(4, 3, 5);
    let tree = branch(0, 0, 5, vec![deep, shallow]);

    let root = tree.root_node();
    assert_eq!(root.child_count(), 2);

    // deep path
    let d0 = root.child(0).unwrap();
    assert_eq!(d0.child_count(), 1);
    let d1 = d0.child(0).unwrap();
    assert_eq!(d1.child_count(), 1);
    let d2 = d1.child(0).unwrap();
    assert_eq!(d2.child_count(), 0);
    assert_eq!(d2.kind_id(), 3);

    // shallow sibling
    let s = root.child(1).unwrap();
    assert_eq!(s.child_count(), 0);
    assert_eq!(s.kind_id(), 4);
}

// ===== 25. Point display format =====

#[test]
fn point_display() {
    let p = adze_runtime::Point::new(2, 5);
    let display = format!("{p}");
    // Display is 1-indexed: row+1:column+1
    assert_eq!(display, "3:6");
}

// ===== 26. Point ordering =====

#[test]
fn point_ordering() {
    let a = adze_runtime::Point::new(0, 0);
    let b = adze_runtime::Point::new(0, 5);
    let c = adze_runtime::Point::new(1, 0);
    assert!(a < b);
    assert!(b < c);
    assert!(a < c);
}

// ===== 27. Node methods on every child of a 5-child tree =====

#[test]
fn all_methods_on_every_child() {
    let children: Vec<Tree> = (0..5)
        .map(|i| leaf(i + 1, (i * 3) as usize, ((i + 1) * 3) as usize))
        .collect();
    let tree = branch(0, 0, 15, children);
    let root = tree.root_node();

    for i in 0..5 {
        let c = root.child(i).unwrap();
        assert_eq!(c.kind_id(), (i + 1) as u16);
        assert_eq!(c.start_byte(), i * 3);
        assert_eq!(c.end_byte(), (i + 1) * 3);
        assert_eq!(c.byte_range(), (i * 3)..((i + 1) * 3));
        assert_eq!(c.start_position().row, 0);
        assert_eq!(c.end_position().row, 0);
        assert!(c.is_named());
        assert!(!c.is_missing());
        assert!(!c.is_error());
        assert_eq!(c.child_count(), 0);
        assert_eq!(c.named_child_count(), 0);
        assert!(c.child(0).is_none());
        assert!(c.named_child(0).is_none());
        assert!(c.child_by_field_name("x").is_none());
        assert!(c.parent().is_none());
        assert!(c.next_sibling().is_none());
        assert!(c.prev_sibling().is_none());
    }
}

// ===== 28. root_kind on Tree =====

#[test]
fn tree_root_kind() {
    let tree = Tree::new_for_testing(42, 0, 10, vec![]);
    assert_eq!(tree.root_kind(), 42);
}

// ===== 29. language / source_bytes on testing trees =====

#[test]
fn testing_tree_has_no_language() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    assert!(tree.language().is_none());
}

#[test]
fn testing_tree_has_no_source() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    assert!(tree.source_bytes().is_none());
}

// ===== 30. Debug on Tree =====

#[test]
fn tree_debug_not_empty() {
    let tree = branch(0, 0, 5, vec![leaf(1, 0, 5)]);
    let dbg = format!("{tree:?}");
    assert!(dbg.contains("Tree"));
}

// ===== 31. utf8_text multibyte boundaries =====

#[test]
fn utf8_text_multibyte_char_boundary() {
    let source = "a→b".as_bytes(); // → is 3 bytes (E2 86 92)
    // source layout: a(1) →(3) b(1) = 5 bytes
    assert_eq!(source.len(), 5);
    let tree = branch(0, 0, 5, vec![leaf(1, 0, 1), leaf(2, 1, 4), leaf(3, 4, 5)]);
    let root = tree.root_node();
    assert_eq!(root.child(0).unwrap().utf8_text(source).unwrap(), "a");
    assert_eq!(root.child(1).unwrap().utf8_text(source).unwrap(), "→");
    assert_eq!(root.child(2).unwrap().utf8_text(source).unwrap(), "b");
}

// ===== 32. Adjacent children with no gaps =====

#[test]
fn adjacent_children_no_gaps() {
    let source = b"abcdef";
    let tree = branch(0, 0, 6, vec![leaf(1, 0, 2), leaf(2, 2, 4), leaf(3, 4, 6)]);
    let root = tree.root_node();
    let mut reconstructed = String::new();
    for i in 0..root.child_count() {
        reconstructed.push_str(root.child(i).unwrap().utf8_text(source).unwrap());
    }
    assert_eq!(reconstructed, "abcdef");
}

// ===== 33. Child iteration in order =====

#[test]
fn child_iteration_preserves_order() {
    let children: Vec<Tree> = (1..=10)
        .map(|i| leaf(i as u32, (i - 1) * 2, i * 2))
        .collect();
    let tree = branch(0, 0, 20, children);
    let root = tree.root_node();

    for i in 1..=10 {
        let child = root.child((i - 1) as usize).unwrap();
        assert_eq!(child.kind_id(), i as u16);
    }
}

// ===== 34. Node with all methods called (comprehensive call) =====

#[test]
fn comprehensive_node_method_call() {
    let tree = branch(
        100,
        5,
        25,
        vec![
            leaf(101, 5, 10),
            leaf(102, 10, 15),
            leaf(103, 15, 20),
            leaf(104, 20, 25),
        ],
    );
    let root = tree.root_node();

    // Call all read methods
    let kind_id = root.kind_id();
    let kind = root.kind();
    let start = root.start_byte();
    let end = root.end_byte();
    let range = root.byte_range();
    let start_pos = root.start_position();
    let end_pos = root.end_position();
    let is_named = root.is_named();
    let is_missing = root.is_missing();
    let is_error = root.is_error();
    let child_cnt = root.child_count();
    let named_cnt = root.named_child_count();

    // Verify calls succeeded
    assert_eq!(kind_id, 100);
    assert_eq!(kind, "unknown");
    assert_eq!(start, 5);
    assert_eq!(end, 25);
    assert_eq!(range, 5..25);
    assert_eq!(start_pos.row, 0);
    assert_eq!(end_pos.column, 0);
    assert!(is_named);
    assert!(!is_missing);
    assert!(!is_error);
    assert_eq!(child_cnt, 4);
    assert_eq!(named_cnt, 4);

    // Call child access methods
    assert!(root.child(0).is_some());
    assert!(root.child(4).is_none());
    assert!(root.named_child(0).is_some());
    assert!(root.child_by_field_name("test").is_none());
    assert!(root.parent().is_none());
    assert!(root.next_sibling().is_none());
    assert!(root.prev_sibling().is_none());
}

// ===== 35. Empty and single-byte nodes =====

#[test]
fn empty_byte_range_nodes() {
    let tree = branch(1, 0, 0, vec![leaf(2, 0, 0)]);
    let root = tree.root_node();
    assert!(root.byte_range().is_empty());
    let child = root.child(0).unwrap();
    assert!(child.byte_range().is_empty());
}

// ===== 36. Byte range consistency across methods =====

#[test]
fn byte_range_method_consistency() {
    let tree = leaf(5, 100, 200);
    let node = tree.root_node();
    let start = node.start_byte();
    let end = node.end_byte();
    let range = node.byte_range();

    assert_eq!(range.start, start);
    assert_eq!(range.end, end);
    assert_eq!(range.len(), end - start);
}

// ===== 37. Node equality via Copy =====

#[test]
fn node_copy_equality() {
    let tree = branch(0, 0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
    let node1 = tree.root_node();
    let node2 = tree.root_node();

    // Since Node is Copy, both should reference same data
    assert_eq!(node1.kind_id(), node2.kind_id());
    assert_eq!(node1.start_byte(), node2.start_byte());
    assert_eq!(node1.end_byte(), node2.end_byte());
}

// ===== 38. Multiple sequential child access =====

#[test]
fn sequential_child_access_consistency() {
    let tree = branch(
        0,
        0,
        15,
        vec![
            leaf(10, 0, 3),
            leaf(20, 3, 6),
            leaf(30, 6, 9),
            leaf(40, 9, 12),
            leaf(50, 12, 15),
        ],
    );
    let root = tree.root_node();

    // Access same child multiple times
    let c0a = root.child(0).unwrap();
    let c0b = root.child(0).unwrap();
    assert_eq!(c0a.kind_id(), c0b.kind_id());
    assert_eq!(c0a.start_byte(), c0b.start_byte());

    // Access different children
    let c1 = root.child(1).unwrap();
    assert_ne!(c0a.kind_id(), c1.kind_id());
}

// ===== 39. Tree with maximum practical size =====

#[test]
fn large_children_count() {
    let children: Vec<Tree> = (0..100).map(|i| leaf((i % 256) as u32, i, i + 1)).collect();
    let tree = branch(0, 0, 100, children);
    let root = tree.root_node();

    assert_eq!(root.child_count(), 100);
    for i in 0..100 {
        let child = root.child(i).unwrap();
        assert_eq!(child.start_byte(), i);
        assert_eq!(child.end_byte(), i + 1);
    }
}

// ===== 40. Point struct comprehensive tests =====

#[test]
fn point_construction_and_access() {
    let p = adze_runtime::Point::new(5, 10);
    assert_eq!(p.row, 5);
    assert_eq!(p.column, 10);
}

#[test]
fn point_equality() {
    let p1 = adze_runtime::Point::new(1, 2);
    let p2 = adze_runtime::Point::new(1, 2);
    assert_eq!(p1, p2);
    let p3 = adze_runtime::Point::new(1, 3);
    assert_ne!(p1, p3);
}

// ===== 41. Deeply nested tree with width at each level =====

#[test]
fn deep_and_wide_tree() {
    // Build tree: root -> [3 children, each with 3 children]
    let mut children = vec![];
    for i in 0..3 {
        let grandchildren: Vec<Tree> = (0..3).map(|j| leaf((i * 10 + j) as u32, 0, 1)).collect();
        let child = branch(i as u32, 0, 3, grandchildren);
        children.push(child);
    }
    let tree = branch(0, 0, 3, children);
    let root = tree.root_node();

    assert_eq!(root.child_count(), 3);
    for i in 0..3 {
        let child = root.child(i).unwrap();
        assert_eq!(child.child_count(), 3);
    }
}

// ===== 42. All siblings are siblings to each other =====

#[test]
fn siblings_are_at_same_level() {
    let tree = branch(
        0,
        0,
        12,
        vec![leaf(1, 0, 3), leaf(2, 3, 6), leaf(3, 6, 9), leaf(4, 9, 12)],
    );
    let root = tree.root_node();

    // All children have no parent, sibling, or parent
    for i in 0..4 {
        let child = root.child(i).unwrap();
        assert!(child.parent().is_none());
        assert!(child.next_sibling().is_none());
        assert!(child.prev_sibling().is_none());
    }
}

// ===== 43. utf8_text on all children of multilevel tree =====

#[test]
fn utf8_text_entire_nested_structure() {
    let source = b"(a+b)";
    let tree = branch(
        0,
        0,
        5,
        vec![branch(
            1,
            0,
            5,
            vec![
                leaf(2, 0, 1),
                leaf(3, 1, 2),
                leaf(4, 2, 3),
                leaf(5, 3, 4),
                leaf(6, 4, 5),
            ],
        )],
    );

    let root = tree.root_node();
    let inner = root.child(0).unwrap();
    assert_eq!(inner.child(0).unwrap().utf8_text(source).unwrap(), "(");
    assert_eq!(inner.child(1).unwrap().utf8_text(source).unwrap(), "a");
    assert_eq!(inner.child(2).unwrap().utf8_text(source).unwrap(), "+");
    assert_eq!(inner.child(3).unwrap().utf8_text(source).unwrap(), "b");
    assert_eq!(inner.child(4).unwrap().utf8_text(source).unwrap(), ")");
}
