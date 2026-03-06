//! Comprehensive tests for Parser configuration, Tree construction, Node navigation,
//! TreeCursor traversal, and edge cases in adze-runtime.

use adze_runtime::tree::TreeCursor;
use adze_runtime::{Parser, Point, Tree};
use std::time::Duration;

// ───────────────────────── helpers ──────────────────────────

/// Build a leaf tree node for testing.
fn leaf(symbol: u32, start: usize, end: usize) -> Tree {
    Tree::new_for_testing(symbol, start, end, vec![])
}

/// Build a tree with children for testing.
fn branch(symbol: u32, start: usize, end: usize, children: Vec<Tree>) -> Tree {
    Tree::new_for_testing(symbol, start, end, children)
}

// ═══════════════════════════════════════════════════════════
// 1. Parser creation (8 tests)
// ═══════════════════════════════════════════════════════════

#[test]
fn parser_new_returns_no_language() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
}

#[test]
fn parser_new_returns_no_timeout() {
    let parser = Parser::new();
    assert!(parser.timeout().is_none());
}

#[test]
fn parser_default_equals_new() {
    let a = Parser::new();
    let b = Parser::default();
    // Both should have no language and no timeout
    assert!(a.language().is_none());
    assert!(b.language().is_none());
    assert!(a.timeout().is_none());
    assert!(b.timeout().is_none());
}

#[test]
fn parser_debug_format_contains_parser() {
    let parser = Parser::new();
    let dbg = format!("{parser:?}");
    assert!(dbg.contains("Parser"), "Debug output: {dbg}");
}

#[test]
fn parser_set_timeout_roundtrips() {
    let mut parser = Parser::new();
    let dur = Duration::from_millis(500);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

#[test]
fn parser_set_timeout_zero() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::ZERO);
    assert_eq!(parser.timeout(), Some(Duration::ZERO));
}

#[test]
fn parser_set_timeout_large_value() {
    let mut parser = Parser::new();
    let dur = Duration::from_secs(3600);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

#[test]
fn parser_set_timeout_micros() {
    let mut parser = Parser::new();
    let dur = Duration::from_micros(250);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

// ═══════════════════════════════════════════════════════════
// 2. Parser config / timeout behaviour (8 tests)
// ═══════════════════════════════════════════════════════════

#[test]
fn parser_timeout_overwrite() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(100));
    parser.set_timeout(Duration::from_millis(200));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(200)));
}

#[test]
fn parser_timeout_nanos_precision() {
    let mut parser = Parser::new();
    let dur = Duration::from_nanos(42);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout().unwrap().as_nanos(), 42);
}

#[test]
fn parser_language_initially_none() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
}

#[test]
fn parser_set_language_rejects_empty_metadata() {
    use adze_runtime::Language;
    let mut parser = Parser::new();
    let lang = Language::builder().symbol_metadata(vec![]).build();
    // Building with empty metadata should fail or set_language should reject
    match lang {
        Ok(l) => {
            let result = parser.set_language(l);
            assert!(result.is_err(), "empty metadata should be rejected");
        }
        Err(_) => { /* builder rejected it, also fine */ }
    }
}

#[test]
fn parser_default_debug_is_consistent() {
    let dbg1 = format!("{:?}", Parser::new());
    let dbg2 = format!("{:?}", Parser::default());
    assert_eq!(dbg1, dbg2);
}

#[test]
fn parser_multiple_timeouts_last_wins() {
    let mut parser = Parser::new();
    for i in 1..=10 {
        parser.set_timeout(Duration::from_millis(i * 100));
    }
    assert_eq!(parser.timeout(), Some(Duration::from_millis(1000)));
}

#[test]
fn parser_timeout_from_secs_f64() {
    let mut parser = Parser::new();
    let dur = Duration::from_secs_f64(0.001);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

#[test]
fn parser_timeout_max_duration() {
    let mut parser = Parser::new();
    let dur = Duration::from_secs(u64::MAX / 2);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

// ═══════════════════════════════════════════════════════════
// 3. Tree construction (8 tests)
// ═══════════════════════════════════════════════════════════

#[test]
fn tree_stub_has_zero_kind() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_kind(), 0);
}

#[test]
fn tree_stub_root_node_zero_range() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
}

#[test]
fn tree_new_for_testing_sets_symbol() {
    let tree = Tree::new_for_testing(42, 0, 10, vec![]);
    assert_eq!(tree.root_kind(), 42);
}

#[test]
fn tree_new_for_testing_kind_id_truncates_to_u16() {
    // symbol stored as u32, but kind_id() returns u16
    let tree = Tree::new_for_testing(0x1_0001, 0, 5, vec![]);
    assert_eq!(tree.root_kind(), 0x1_0001); // root_kind returns u32
    assert_eq!(tree.root_node().kind_id(), 1); // truncated to u16
}

#[test]
fn tree_new_for_testing_byte_range() {
    let tree = Tree::new_for_testing(1, 10, 20, vec![]);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 10);
    assert_eq!(root.end_byte(), 20);
    assert_eq!(root.byte_range(), 10..20);
}

#[test]
fn tree_new_for_testing_with_children() {
    let child_a = leaf(2, 0, 3);
    let child_b = leaf(3, 3, 6);
    let tree = branch(1, 0, 6, vec![child_a, child_b]);
    assert_eq!(tree.root_node().child_count(), 2);
}

#[test]
fn tree_language_is_none_for_testing_tree() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    assert!(tree.language().is_none());
}

#[test]
fn tree_source_bytes_none_for_testing_tree() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    assert!(tree.source_bytes().is_none());
}

// ═══════════════════════════════════════════════════════════
// 4. Node navigation (8 tests)
// ═══════════════════════════════════════════════════════════

#[test]
fn node_child_returns_some_for_valid_index() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5), leaf(3, 5, 10)]);
    let root = tree.root_node();
    assert!(root.child(0).is_some());
    assert!(root.child(1).is_some());
}

#[test]
fn node_child_returns_none_for_out_of_bounds() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5)]);
    let root = tree.root_node();
    assert!(root.child(1).is_none());
    assert!(root.child(99).is_none());
}

#[test]
fn node_child_preserves_symbol() {
    let tree = branch(1, 0, 10, vec![leaf(7, 0, 5), leaf(8, 5, 10)]);
    let root = tree.root_node();
    assert_eq!(root.child(0).unwrap().kind_id(), 7);
    assert_eq!(root.child(1).unwrap().kind_id(), 8);
}

#[test]
fn node_named_child_count_matches_child_count_phase1() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5), leaf(3, 5, 10)]);
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

#[test]
fn node_named_child_same_as_child_phase1() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5)]);
    let root = tree.root_node();
    let c1 = root.child(0).unwrap();
    let c2 = root.named_child(0).unwrap();
    assert_eq!(c1.kind_id(), c2.kind_id());
    assert_eq!(c1.start_byte(), c2.start_byte());
}

#[test]
fn node_child_count_zero_for_leaf() {
    let tree = leaf(5, 0, 3);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn node_child_by_field_name_returns_none() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5)]);
    assert!(tree.root_node().child_by_field_name("name").is_none());
}

#[test]
fn node_parent_returns_none() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5)]);
    let child = tree.root_node().child(0).unwrap();
    assert!(child.parent().is_none());
}

// ═══════════════════════════════════════════════════════════
// 5. Node properties (8 tests)
// ═══════════════════════════════════════════════════════════

#[test]
fn node_is_named_always_true_phase1() {
    let tree = leaf(1, 0, 3);
    assert!(tree.root_node().is_named());
}

#[test]
fn node_is_error_always_false() {
    let tree = leaf(1, 0, 3);
    assert!(!tree.root_node().is_error());
}

#[test]
fn node_is_missing_always_false() {
    let tree = leaf(1, 0, 3);
    assert!(!tree.root_node().is_missing());
}

#[test]
fn node_kind_unknown_without_language() {
    let tree = leaf(42, 0, 5);
    assert_eq!(tree.root_node().kind(), "unknown");
}

#[test]
fn node_kind_id_is_u16() {
    let tree = leaf(255, 0, 5);
    let id: u16 = tree.root_node().kind_id();
    assert_eq!(id, 255);
}

#[test]
fn node_start_position_is_zero_phase1() {
    let tree = leaf(1, 10, 20);
    let pos = tree.root_node().start_position();
    assert_eq!(pos, Point::new(0, 0));
}

#[test]
fn node_end_position_is_zero_phase1() {
    let tree = leaf(1, 10, 20);
    let pos = tree.root_node().end_position();
    assert_eq!(pos, Point::new(0, 0));
}

#[test]
fn node_utf8_text_extracts_slice() {
    let source = b"hello world";
    let tree = leaf(1, 6, 11);
    let text = tree.root_node().utf8_text(source).unwrap();
    assert_eq!(text, "world");
}

// ═══════════════════════════════════════════════════════════
// 6. Tree cursor (7 tests)
// ═══════════════════════════════════════════════════════════

#[test]
fn cursor_starts_at_root() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5)]);
    let cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_goto_first_child() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5), leaf(3, 5, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 2);
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn cursor_goto_next_sibling() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5), leaf(3, 5, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 3);
}

#[test]
fn cursor_goto_next_sibling_at_last_child() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5)]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert!(!cursor.goto_next_sibling());
}

#[test]
fn cursor_goto_parent() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5)]);
    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_goto_parent_at_root_returns_false() {
    let tree = leaf(1, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_reset_returns_to_root() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5), leaf(3, 5, 10)]);
    let mut cursor = TreeCursor::new(&tree);
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    cursor.reset(&tree);
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.depth(), 0);
}

// ═══════════════════════════════════════════════════════════
// 7. Edge cases (8 tests)
// ═══════════════════════════════════════════════════════════

#[test]
fn tree_stub_has_no_children() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn tree_single_node_no_children() {
    let tree = leaf(99, 0, 100);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 0);
    assert!(root.child(0).is_none());
}

#[test]
fn tree_deeply_nested() {
    // 5 levels deep
    let mut current = leaf(5, 0, 1);
    for sym in (1..5).rev() {
        current = branch(sym, 0, 1, vec![current]);
    }
    let root = current.root_node();
    assert_eq!(root.kind_id(), 1);
    let c1 = root.child(0).unwrap();
    assert_eq!(c1.kind_id(), 2);
    let c2 = c1.child(0).unwrap();
    assert_eq!(c2.kind_id(), 3);
    let c3 = c2.child(0).unwrap();
    assert_eq!(c3.kind_id(), 4);
    let c4 = c3.child(0).unwrap();
    assert_eq!(c4.kind_id(), 5);
    assert_eq!(c4.child_count(), 0);
}

#[test]
fn tree_wide_many_children() {
    let children: Vec<Tree> = (0..20)
        .map(|i| leaf(i + 10, i as usize, (i + 1) as usize))
        .collect();
    let tree = branch(1, 0, 20, children);
    assert_eq!(tree.root_node().child_count(), 20);
    assert_eq!(tree.root_node().child(19).unwrap().kind_id(), 29);
}

#[test]
fn tree_clone_is_independent() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5)]);
    let cloned = tree.clone();
    assert_eq!(cloned.root_kind(), tree.root_kind());
    assert_eq!(
        cloned.root_node().child_count(),
        tree.root_node().child_count()
    );
}

#[test]
fn tree_debug_output_not_empty() {
    let tree = leaf(1, 0, 5);
    let dbg = format!("{tree:?}");
    assert!(!dbg.is_empty());
}

#[test]
fn cursor_leaf_no_first_child() {
    let tree = leaf(1, 0, 5);
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
}

#[test]
fn cursor_deep_traversal_depth_tracking() {
    let inner = branch(3, 0, 1, vec![leaf(4, 0, 1)]);
    let tree = branch(1, 0, 1, vec![branch(2, 0, 1, vec![inner])]);
    let mut cursor = TreeCursor::new(&tree);
    assert_eq!(cursor.depth(), 0);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 2);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 3);
    assert!(!cursor.goto_first_child()); // leaf
    assert!(cursor.goto_parent());
    assert_eq!(cursor.depth(), 2);
}

// ═══════════════════════════════════════════════════════════
// 8. Additional coverage (8 tests)
// ═══════════════════════════════════════════════════════════

#[test]
fn node_debug_format_contains_range() {
    let tree = leaf(1, 5, 15);
    let dbg = format!("{:?}", tree.root_node());
    assert!(
        dbg.contains("5..15") || dbg.contains("5, 15"),
        "Debug: {dbg}"
    );
}

#[test]
fn node_byte_range_empty_node() {
    let tree = leaf(1, 7, 7);
    assert_eq!(tree.root_node().byte_range(), 7..7);
}

#[test]
fn point_new_and_fields() {
    let p = Point::new(3, 7);
    assert_eq!(p.row, 3);
    assert_eq!(p.column, 7);
}

#[test]
fn point_display_is_one_indexed() {
    let p = Point::new(0, 0);
    assert_eq!(format!("{p}"), "1:1");
}

#[test]
fn point_equality() {
    assert_eq!(Point::new(1, 2), Point::new(1, 2));
    assert_ne!(Point::new(1, 2), Point::new(2, 1));
}

#[test]
fn point_ordering() {
    assert!(Point::new(0, 5) < Point::new(1, 0));
    assert!(Point::new(1, 0) < Point::new(1, 1));
}

#[test]
fn node_next_sibling_returns_none() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5)]);
    let child = tree.root_node().child(0).unwrap();
    assert!(child.next_sibling().is_none());
}

#[test]
fn node_prev_sibling_returns_none() {
    let tree = branch(1, 0, 10, vec![leaf(2, 0, 5)]);
    let child = tree.root_node().child(0).unwrap();
    assert!(child.prev_sibling().is_none());
}
