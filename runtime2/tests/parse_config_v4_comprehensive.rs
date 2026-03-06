//! Comprehensive tests for Parser construction, configuration, Tree output,
//! and edge cases in the runtime2 crate.

use adze_runtime::parser::Parser;
use adze_runtime::tree::Tree;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn stub_language() -> adze_runtime::Language {
    adze_runtime::test_helpers::stub_language()
}

fn multi_symbol_language(n: usize) -> adze_runtime::Language {
    adze_runtime::test_helpers::multi_symbol_test_language(n)
}

// ===========================================================================
// 1. Parser construction (8 tests)
// ===========================================================================

#[test]
fn parser_new_returns_no_language() {
    let p = Parser::new();
    assert!(p.language().is_none());
}

#[test]
fn parser_new_returns_no_timeout() {
    let p = Parser::new();
    assert!(p.timeout().is_none());
}

#[test]
fn parser_default_is_equivalent_to_new() {
    let p1 = Parser::new();
    let p2 = Parser::default();
    // Both should start with no language and no timeout.
    assert!(p1.language().is_none());
    assert!(p2.language().is_none());
    assert!(p1.timeout().is_none());
    assert!(p2.timeout().is_none());
}

#[test]
fn parser_new_is_debug() {
    let p = Parser::new();
    let dbg = format!("{:?}", p);
    assert!(dbg.contains("Parser"));
}

#[test]
fn parser_multiple_independent_instances() {
    let mut p1 = Parser::new();
    let p2 = Parser::new();
    p1.set_timeout(Duration::from_secs(5));
    assert!(p1.timeout().is_some());
    assert!(p2.timeout().is_none());
}

#[test]
fn parser_new_set_language_succeeds() {
    let mut p = Parser::new();
    let lang = stub_language();
    assert!(p.set_language(lang).is_ok());
}

#[test]
fn parser_new_language_round_trip() {
    let mut p = Parser::new();
    let lang = stub_language();
    p.set_language(lang).unwrap();
    assert!(p.language().is_some());
}

#[test]
fn parser_new_set_language_twice() {
    let mut p = Parser::new();
    p.set_language(stub_language()).unwrap();
    p.set_language(stub_language()).unwrap();
    assert!(p.language().is_some());
}

// ===========================================================================
// 2. Parser timeout / config defaults (8 tests)
// ===========================================================================

#[test]
fn timeout_default_is_none() {
    let p = Parser::new();
    assert_eq!(p.timeout(), None);
}

#[test]
fn timeout_set_and_get_1s() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_secs(1));
    assert_eq!(p.timeout(), Some(Duration::from_secs(1)));
}

#[test]
fn timeout_set_zero_duration() {
    let mut p = Parser::new();
    p.set_timeout(Duration::ZERO);
    assert_eq!(p.timeout(), Some(Duration::ZERO));
}

#[test]
fn timeout_overwrite_replaces_value() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_secs(1));
    p.set_timeout(Duration::from_secs(99));
    assert_eq!(p.timeout(), Some(Duration::from_secs(99)));
}

#[test]
fn timeout_large_value() {
    let mut p = Parser::new();
    let dur = Duration::from_secs(86400);
    p.set_timeout(dur);
    assert_eq!(p.timeout(), Some(dur));
}

#[test]
fn timeout_nanos_precision() {
    let mut p = Parser::new();
    let dur = Duration::from_nanos(42);
    p.set_timeout(dur);
    assert_eq!(p.timeout(), Some(dur));
}

#[test]
fn timeout_millis_precision() {
    let mut p = Parser::new();
    let dur = Duration::from_millis(250);
    p.set_timeout(dur);
    assert_eq!(p.timeout(), Some(dur));
}

#[test]
fn timeout_max_duration() {
    let mut p = Parser::new();
    let dur = Duration::MAX;
    p.set_timeout(dur);
    assert_eq!(p.timeout(), Some(dur));
}

// ===========================================================================
// 3. Parser config customization (5 tests)
// ===========================================================================

#[test]
fn set_language_then_timeout_order_independent() {
    let mut p = Parser::new();
    p.set_language(stub_language()).unwrap();
    p.set_timeout(Duration::from_secs(2));
    assert!(p.language().is_some());
    assert_eq!(p.timeout(), Some(Duration::from_secs(2)));
}

#[test]
fn set_timeout_then_language_order_independent() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_millis(100));
    p.set_language(stub_language()).unwrap();
    assert!(p.language().is_some());
    assert_eq!(p.timeout(), Some(Duration::from_millis(100)));
}

#[test]
fn replace_language_preserves_timeout() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_secs(7));
    p.set_language(stub_language()).unwrap();
    p.set_language(multi_symbol_language(3)).unwrap();
    assert_eq!(p.timeout(), Some(Duration::from_secs(7)));
}

#[test]
fn language_version_accessible() {
    let mut p = Parser::new();
    p.set_language(stub_language()).unwrap();
    let lang = p.language().unwrap();
    // Version is 0 by default in the stub language builder.
    assert_eq!(lang.version, 0);
}

#[test]
fn language_symbol_count_matches() {
    let mut p = Parser::new();
    let lang = multi_symbol_language(5);
    p.set_language(lang).unwrap();
    assert_eq!(p.language().unwrap().symbol_count, 5);
}

// ===========================================================================
// 4. Parser state management (8 tests)
// ===========================================================================

#[test]
fn parser_without_language_returns_no_language() {
    let p = Parser::new();
    assert!(p.language().is_none());
}

#[test]
fn parser_language_ref_symbol_names() {
    let mut p = Parser::new();
    p.set_language(multi_symbol_language(3)).unwrap();
    let lang = p.language().unwrap();
    assert_eq!(lang.symbol_names.len(), 3);
    assert_eq!(lang.symbol_name(0), Some("symbol_0"));
}

#[test]
fn parser_language_is_terminal() {
    let mut p = Parser::new();
    p.set_language(multi_symbol_language(2)).unwrap();
    let lang = p.language().unwrap();
    // The test helper makes all symbols terminal.
    assert!(lang.is_terminal(0));
    assert!(lang.is_terminal(1));
}

#[test]
fn parser_language_is_visible() {
    let mut p = Parser::new();
    p.set_language(multi_symbol_language(2)).unwrap();
    let lang = p.language().unwrap();
    assert!(lang.is_visible(0));
}

#[test]
fn parser_reset_does_not_clear_language() {
    let mut p = Parser::new();
    p.set_language(stub_language()).unwrap();
    p.reset();
    // reset only clears arenas, not language.
    assert!(p.language().is_some());
}

#[test]
fn parser_reset_does_not_clear_timeout() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_secs(3));
    p.reset();
    assert_eq!(p.timeout(), Some(Duration::from_secs(3)));
}

#[test]
fn parser_debug_contains_language_none() {
    let p = Parser::new();
    let dbg = format!("{:?}", p);
    assert!(dbg.contains("None"), "debug output: {dbg}");
}

#[test]
fn parser_debug_after_set_language() {
    let mut p = Parser::new();
    p.set_language(stub_language()).unwrap();
    let dbg = format!("{:?}", p);
    assert!(dbg.contains("Some"), "debug output: {dbg}");
}

// ===========================================================================
// 5. Tree output from new_for_testing (8 tests)
// ===========================================================================

#[test]
fn tree_new_for_testing_leaf() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    let root = tree.root_node();
    assert_eq!(root.kind_id(), 1);
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 5);
    assert_eq!(root.child_count(), 0);
}

#[test]
fn tree_new_for_testing_with_children() {
    let child_a = Tree::new_for_testing(2, 0, 3, vec![]);
    let child_b = Tree::new_for_testing(3, 3, 6, vec![]);
    let parent = Tree::new_for_testing(1, 0, 6, vec![child_a, child_b]);
    assert_eq!(parent.root_node().child_count(), 2);
}

#[test]
fn tree_new_for_testing_nested() {
    let grandchild = Tree::new_for_testing(3, 1, 2, vec![]);
    let child = Tree::new_for_testing(2, 0, 3, vec![grandchild]);
    let root = Tree::new_for_testing(1, 0, 5, vec![child]);

    let root_node = root.root_node();
    assert_eq!(root_node.child_count(), 1);
    let c = root_node.child(0).unwrap();
    assert_eq!(c.kind_id(), 2);
    assert_eq!(c.child_count(), 1);
    let gc = c.child(0).unwrap();
    assert_eq!(gc.kind_id(), 3);
}

#[test]
fn tree_new_for_testing_root_kind() {
    let tree = Tree::new_for_testing(42, 0, 10, vec![]);
    assert_eq!(tree.root_kind(), 42);
}

#[test]
fn tree_new_for_testing_no_language() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    assert!(tree.language().is_none());
}

#[test]
fn tree_new_for_testing_no_source() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    assert!(tree.source_bytes().is_none());
}

#[test]
fn tree_new_for_testing_child_byte_ranges() {
    let child = Tree::new_for_testing(2, 10, 20, vec![]);
    let parent = Tree::new_for_testing(1, 5, 25, vec![child]);
    let c = parent.root_node().child(0).unwrap();
    assert_eq!(c.start_byte(), 10);
    assert_eq!(c.end_byte(), 20);
}

#[test]
fn tree_new_for_testing_many_children() {
    let children: Vec<Tree> = (0..10)
        .map(|i| Tree::new_for_testing(i + 1, i as usize, (i + 1) as usize, vec![]))
        .collect();
    let tree = Tree::new_for_testing(0, 0, 10, children);
    assert_eq!(tree.root_node().child_count(), 10);
}

// ===========================================================================
// 6. Tree / Parser clone & debug (5 tests)
// ===========================================================================

#[test]
fn tree_clone_is_independent() {
    let tree = Tree::new_for_testing(1, 0, 10, vec![]);
    let cloned = tree.clone();
    assert_eq!(cloned.root_kind(), tree.root_kind());
    assert_eq!(
        cloned.root_node().start_byte(),
        tree.root_node().start_byte()
    );
}

#[test]
fn tree_clone_preserves_children() {
    let child = Tree::new_for_testing(2, 0, 3, vec![]);
    let tree = Tree::new_for_testing(1, 0, 5, vec![child]);
    let cloned = tree.clone();
    assert_eq!(cloned.root_node().child_count(), 1);
}

#[test]
fn tree_debug_output() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    let dbg = format!("{:?}", tree);
    assert!(dbg.contains("Tree"), "debug output: {dbg}");
}

#[test]
fn tree_stub_debug() {
    let tree = Tree::new_stub();
    let dbg = format!("{:?}", tree);
    assert!(dbg.contains("Tree"), "debug output: {dbg}");
}

#[test]
fn parser_debug_format() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_secs(10));
    let dbg = format!("{:?}", p);
    assert!(dbg.contains("Parser"), "debug output: {dbg}");
}

// ===========================================================================
// 7. Parser reset and reuse (5 tests)
// ===========================================================================

#[test]
fn parser_reset_on_fresh_parser() {
    let mut p = Parser::new();
    p.reset();
    assert!(p.language().is_none());
}

#[test]
fn parser_reset_then_set_language() {
    let mut p = Parser::new();
    p.set_language(stub_language()).unwrap();
    p.reset();
    // Language is still set after reset.
    assert!(p.language().is_some());
    // Can set again.
    p.set_language(multi_symbol_language(4)).unwrap();
    assert_eq!(p.language().unwrap().symbol_count, 4);
}

#[test]
fn parser_reuse_after_replace_language() {
    let mut p = Parser::new();
    p.set_language(multi_symbol_language(2)).unwrap();
    assert_eq!(p.language().unwrap().symbol_count, 2);
    p.set_language(multi_symbol_language(8)).unwrap();
    assert_eq!(p.language().unwrap().symbol_count, 8);
}

#[test]
fn parser_multiple_resets() {
    let mut p = Parser::new();
    p.set_language(stub_language()).unwrap();
    p.set_timeout(Duration::from_secs(1));
    for _ in 0..5 {
        p.reset();
    }
    assert!(p.language().is_some());
    assert_eq!(p.timeout(), Some(Duration::from_secs(1)));
}

#[test]
fn parser_reset_between_timeout_changes() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_secs(1));
    p.reset();
    p.set_timeout(Duration::from_secs(2));
    assert_eq!(p.timeout(), Some(Duration::from_secs(2)));
}

// ===========================================================================
// 8. Edge cases (8 tests)
// ===========================================================================

#[test]
fn tree_stub_has_zero_symbol() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_kind(), 0);
}

#[test]
fn tree_stub_root_node_zero_range() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
    assert_eq!(root.byte_range(), 0..0);
}

#[test]
fn tree_stub_has_no_children() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn tree_new_for_testing_zero_range() {
    let tree = Tree::new_for_testing(5, 0, 0, vec![]);
    assert_eq!(tree.root_node().byte_range(), 0..0);
}

#[test]
fn tree_new_for_testing_large_symbol() {
    let tree = Tree::new_for_testing(u32::MAX, 0, 1, vec![]);
    assert_eq!(tree.root_kind(), u32::MAX);
    // kind_id truncates to u16
    assert_eq!(tree.root_node().kind_id(), u16::MAX);
}

#[test]
fn tree_clone_stub_independent() {
    let t1 = Tree::new_stub();
    let t2 = t1.clone();
    assert_eq!(t1.root_kind(), t2.root_kind());
}

#[test]
fn parser_parse_without_language_returns_error() {
    let mut p = Parser::new();
    let result = p.parse(b"hello", None);
    assert!(result.is_err());
}

#[test]
fn tree_new_for_testing_child_out_of_bounds() {
    let tree = Tree::new_for_testing(1, 0, 5, vec![]);
    assert!(tree.root_node().child(0).is_none());
    assert!(tree.root_node().child(999).is_none());
}
