//! Comprehensive edge-case tests for visitor patterns, arena allocator, and node handles.

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};
use adze::visitor::{PrettyPrintVisitor, SearchVisitor, StatsVisitor, Visitor, VisitorAction};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

use std::mem::MaybeUninit;

type Node = adze::pure_parser::ParsedNode;
use adze::pure_parser::Point;

/// Shorthand: create a ParsedNode with a given symbol and byte range.
/// Uses MaybeUninit + zero-fill to handle the pub(crate) `language` field.
fn make_node(symbol: u16, start: usize, end: usize) -> Node {
    build_node(symbol, vec![], start, end, false, true)
}

fn build_node(
    symbol: u16,
    children: Vec<Node>,
    start: usize,
    end: usize,
    is_error: bool,
    is_named: bool,
) -> Node {
    let mut uninit = MaybeUninit::<Node>::uninit();
    let ptr = uninit.as_mut_ptr();
    unsafe {
        std::ptr::write_bytes(ptr, 0, 1); // zero everything (language = null)
        std::ptr::addr_of_mut!((*ptr).symbol).write(symbol);
        std::ptr::addr_of_mut!((*ptr).children).write(children);
        std::ptr::addr_of_mut!((*ptr).start_byte).write(start);
        std::ptr::addr_of_mut!((*ptr).end_byte).write(end);
        std::ptr::addr_of_mut!((*ptr).start_point).write(Point {
            row: 0,
            column: start as u32,
        });
        std::ptr::addr_of_mut!((*ptr).end_point).write(Point {
            row: 0,
            column: end as u32,
        });
        std::ptr::addr_of_mut!((*ptr).is_extra).write(false);
        std::ptr::addr_of_mut!((*ptr).is_error).write(is_error);
        std::ptr::addr_of_mut!((*ptr).is_missing).write(false);
        std::ptr::addr_of_mut!((*ptr).is_named).write(is_named);
        std::ptr::addr_of_mut!((*ptr).field_id).write(None);
        uninit.assume_init()
    }
}

fn make_error_node(symbol: u16, start: usize, end: usize) -> Node {
    build_node(symbol, vec![], start, end, true, false)
}

fn make_unnamed_node(symbol: u16, start: usize, end: usize) -> Node {
    build_node(symbol, vec![], start, end, false, false)
}

// =========================================================================
// 1. StatsVisitor — default state
// =========================================================================

#[test]
fn stats_default_total_nodes_is_zero() {
    let s = StatsVisitor::default();
    assert_eq!(s.total_nodes, 0);
}

#[test]
fn stats_default_leaf_nodes_is_zero() {
    let s = StatsVisitor::default();
    assert_eq!(s.leaf_nodes, 0);
}

#[test]
fn stats_default_error_nodes_is_zero() {
    let s = StatsVisitor::default();
    assert_eq!(s.error_nodes, 0);
}

#[test]
fn stats_default_max_depth_is_zero() {
    let s = StatsVisitor::default();
    assert_eq!(s.max_depth, 0);
}

#[test]
fn stats_default_node_counts_is_empty() {
    let s = StatsVisitor::default();
    assert!(s.node_counts.is_empty());
}

#[test]
fn stats_default_debug_impl() {
    let s = StatsVisitor::default();
    let dbg = format!("{:?}", s);
    assert!(dbg.contains("StatsVisitor"));
}

// =========================================================================
// 2. StatsVisitor — manual enter/leave
// =========================================================================

#[test]
fn stats_enter_increments_total_nodes() {
    let mut s = StatsVisitor::default();
    let node = make_node(5, 0, 10);
    s.enter_node(&node);
    assert_eq!(s.total_nodes, 1);
}

#[test]
fn stats_enter_sets_max_depth() {
    let mut s = StatsVisitor::default();
    let node = make_node(5, 0, 10);
    s.enter_node(&node);
    assert_eq!(s.max_depth, 1);
}

#[test]
fn stats_leave_decrements_internal_depth() {
    let mut s = StatsVisitor::default();
    let node = make_node(5, 0, 10);
    s.enter_node(&node);
    s.leave_node(&node);
    // depth back to 0; entering again should set max_depth back to 1
    s.enter_node(&node);
    assert_eq!(s.max_depth, 1);
}

#[test]
fn stats_nested_depth_tracking() {
    let mut s = StatsVisitor::default();
    let n = make_node(5, 0, 10);
    s.enter_node(&n);
    s.enter_node(&n);
    s.enter_node(&n);
    assert_eq!(s.max_depth, 3);
    assert_eq!(s.total_nodes, 3);
}

#[test]
fn stats_visit_leaf_increments_leaf_count() {
    let mut s = StatsVisitor::default();
    let n = make_node(0, 0, 4);
    s.visit_leaf(&n, "test");
    assert_eq!(s.leaf_nodes, 1);
}

#[test]
fn stats_visit_error_increments_error_count() {
    let mut s = StatsVisitor::default();
    let n = make_error_node(0, 0, 1);
    s.visit_error(&n);
    assert_eq!(s.error_nodes, 1);
}

#[test]
fn stats_node_counts_tracks_kinds() {
    let mut s = StatsVisitor::default();
    // symbol 0 -> kind "end"
    let n = make_node(0, 0, 3);
    s.enter_node(&n);
    assert_eq!(*s.node_counts.get("end").unwrap(), 1);
}

#[test]
fn stats_node_counts_accumulates() {
    let mut s = StatsVisitor::default();
    let n = make_node(0, 0, 3);
    s.enter_node(&n);
    s.leave_node(&n);
    s.enter_node(&n);
    assert_eq!(*s.node_counts.get("end").unwrap(), 2);
}

// =========================================================================
// 3. StatsVisitor — multiple uses
// =========================================================================

#[test]
fn stats_reuse_without_reset_accumulates() {
    let mut s = StatsVisitor::default();
    let n = make_node(5, 0, 10);
    for _ in 0..5 {
        s.enter_node(&n);
        s.leave_node(&n);
    }
    assert_eq!(s.total_nodes, 5);
    assert_eq!(s.max_depth, 1);
}

#[test]
fn stats_different_symbols_tracked_separately() {
    let mut s = StatsVisitor::default();
    let a = make_node(0, 0, 1); // "end"
    let b = make_node(5, 0, 1); // "Expression"
    s.enter_node(&a);
    s.leave_node(&a);
    s.enter_node(&b);
    s.leave_node(&b);
    assert_eq!(s.node_counts.len(), 2);
}

#[test]
fn stats_enter_returns_continue() {
    let mut s = StatsVisitor::default();
    let n = make_node(0, 0, 1);
    assert_eq!(s.enter_node(&n), VisitorAction::Continue);
}

// =========================================================================
// 4. PrettyPrintVisitor — empty / basic output
// =========================================================================

#[test]
fn pretty_print_new_output_is_empty() {
    let pp = PrettyPrintVisitor::new();
    assert_eq!(pp.output(), "");
}

#[test]
fn pretty_print_default_output_is_empty() {
    let pp = PrettyPrintVisitor::default();
    assert_eq!(pp.output(), "");
}

#[test]
fn pretty_print_enter_produces_kind() {
    let mut pp = PrettyPrintVisitor::new();
    let n = make_node(0, 0, 3); // kind "end"
    pp.enter_node(&n);
    assert!(pp.output().contains("end"));
}

#[test]
fn pretty_print_named_node_has_tag() {
    let mut pp = PrettyPrintVisitor::new();
    let n = make_node(5, 0, 10); // named=true
    pp.enter_node(&n);
    assert!(pp.output().contains("[named]"));
}

#[test]
fn pretty_print_unnamed_node_no_named_tag() {
    let mut pp = PrettyPrintVisitor::new();
    let n = make_unnamed_node(0, 0, 1);
    pp.enter_node(&n);
    assert!(!pp.output().contains("[named]"));
}

#[test]
fn pretty_print_enter_ends_with_newline() {
    let mut pp = PrettyPrintVisitor::new();
    let n = make_node(0, 0, 1);
    pp.enter_node(&n);
    assert!(pp.output().ends_with('\n'));
}

#[test]
fn pretty_print_nested_indentation() {
    let mut pp = PrettyPrintVisitor::new();
    let n = make_node(0, 0, 1);
    pp.enter_node(&n); // depth 0 -> indent 0
    pp.enter_node(&n); // depth 1 -> indent 2 spaces
    let output = pp.output();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[1].starts_with("  ")); // two-space indent
}

#[test]
fn pretty_print_leave_reduces_indent() {
    let mut pp = PrettyPrintVisitor::new();
    let n = make_node(0, 0, 1);
    pp.enter_node(&n);
    pp.enter_node(&n);
    pp.leave_node(&n);
    pp.enter_node(&n); // should be back to indent level 1
    let output = pp.output();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 3);
    // third line (index 2) should have 2-space indent (level 1)
    assert!(lines[2].starts_with("  "));
    // but NOT 4-space indent
    assert!(!lines[2].starts_with("    "));
}

// =========================================================================
// 5. PrettyPrintVisitor — visit_leaf and special characters
// =========================================================================

#[test]
fn pretty_print_visit_leaf_includes_text() {
    let mut pp = PrettyPrintVisitor::new();
    let n = make_node(0, 0, 5);
    pp.visit_leaf(&n, "hello");
    assert!(pp.output().contains("\"hello\""));
}

#[test]
fn pretty_print_visit_leaf_empty_text() {
    let mut pp = PrettyPrintVisitor::new();
    let n = make_node(0, 0, 0);
    pp.visit_leaf(&n, "");
    assert!(pp.output().contains("\"\""));
}

#[test]
fn pretty_print_special_chars_in_leaf() {
    let mut pp = PrettyPrintVisitor::new();
    let n = make_node(0, 0, 10);
    pp.visit_leaf(&n, "a\tb\nc");
    let output = pp.output();
    assert!(output.contains("a\tb\nc"));
}

#[test]
fn pretty_print_unicode_in_leaf() {
    let mut pp = PrettyPrintVisitor::new();
    let n = make_node(0, 0, 10);
    pp.visit_leaf(&n, "日本語");
    assert!(pp.output().contains("日本語"));
}

#[test]
fn pretty_print_emoji_in_leaf() {
    let mut pp = PrettyPrintVisitor::new();
    let n = make_node(0, 0, 10);
    pp.visit_leaf(&n, "🦀🔥");
    assert!(pp.output().contains("🦀🔥"));
}

#[test]
fn pretty_print_backslash_in_leaf() {
    let mut pp = PrettyPrintVisitor::new();
    let n = make_node(0, 0, 4);
    pp.visit_leaf(&n, r"a\b");
    assert!(pp.output().contains(r"a\b"));
}

#[test]
fn pretty_print_quotes_in_leaf() {
    let mut pp = PrettyPrintVisitor::new();
    let n = make_node(0, 0, 10);
    pp.visit_leaf(&n, r#"he said "hi""#);
    assert!(pp.output().contains(r#"he said "hi""#));
}

// =========================================================================
// 6. PrettyPrintVisitor — visit_error
// =========================================================================

#[test]
fn pretty_print_visit_error_contains_error_prefix() {
    let mut pp = PrettyPrintVisitor::new();
    let n = make_error_node(0, 0, 1);
    pp.visit_error(&n);
    assert!(pp.output().contains("ERROR"));
}

#[test]
fn pretty_print_enter_returns_continue() {
    let mut pp = PrettyPrintVisitor::new();
    let n = make_node(0, 0, 1);
    assert_eq!(pp.enter_node(&n), VisitorAction::Continue);
}

// =========================================================================
// 7. SearchVisitor — initial state and predicate matching
// =========================================================================

#[test]
fn search_initial_matches_empty() {
    let sv = SearchVisitor::new(|_n: &Node| false);
    assert!(sv.matches.is_empty());
}

#[test]
fn search_always_false_no_matches() {
    let mut sv = SearchVisitor::new(|_n: &Node| false);
    let n = make_node(5, 0, 10);
    sv.enter_node(&n);
    assert!(sv.matches.is_empty());
}

#[test]
fn search_always_true_matches_everything() {
    let mut sv = SearchVisitor::new(|_n: &Node| true);
    let n = make_node(5, 0, 10);
    sv.enter_node(&n);
    assert_eq!(sv.matches.len(), 1);
}

#[test]
fn search_match_records_byte_range() {
    let mut sv = SearchVisitor::new(|_n: &Node| true);
    let n = make_node(5, 3, 7);
    sv.enter_node(&n);
    assert_eq!(sv.matches[0].0, 3); // start_byte
    assert_eq!(sv.matches[0].1, 7); // end_byte
}

#[test]
fn search_match_records_kind_string() {
    let mut sv = SearchVisitor::new(|_n: &Node| true);
    let n = make_node(0, 0, 1); // kind "end"
    sv.enter_node(&n);
    assert_eq!(sv.matches[0].2, "end");
}

#[test]
fn search_predicate_filters_by_named() {
    let mut sv = SearchVisitor::new(|n: &Node| n.is_named());
    let named = make_node(0, 0, 1);
    let unnamed = make_unnamed_node(0, 0, 1);
    sv.enter_node(&named);
    sv.enter_node(&unnamed);
    assert_eq!(sv.matches.len(), 1);
}

#[test]
fn search_predicate_by_byte_range() {
    let mut sv = SearchVisitor::new(|n: &Node| n.start_byte() < 5);
    let in_range = make_node(0, 2, 4);
    let out_range = make_node(0, 6, 10);
    sv.enter_node(&in_range);
    sv.enter_node(&out_range);
    assert_eq!(sv.matches.len(), 1);
}

#[test]
fn search_enter_returns_continue() {
    let mut sv = SearchVisitor::new(|_n: &Node| true);
    let n = make_node(0, 0, 1);
    assert_eq!(sv.enter_node(&n), VisitorAction::Continue);
}

#[test]
fn search_multiple_matches_accumulate() {
    let mut sv = SearchVisitor::new(|_n: &Node| true);
    for i in 0..10 {
        let n = make_node(0, i, i + 1);
        sv.enter_node(&n);
    }
    assert_eq!(sv.matches.len(), 10);
}

// =========================================================================
// 8. SearchVisitor — case sensitivity
// =========================================================================

#[test]
fn search_kind_exact_match() {
    let mut sv = SearchVisitor::new(|n: &Node| n.kind() == "end");
    let n = make_node(0, 0, 1); // symbol 0 -> "end"
    sv.enter_node(&n);
    assert_eq!(sv.matches.len(), 1);
}

#[test]
fn search_kind_case_mismatch_no_match() {
    let mut sv = SearchVisitor::new(|n: &Node| n.kind() == "End");
    let n = make_node(0, 0, 1); // "end" lowercase
    sv.enter_node(&n);
    assert!(sv.matches.is_empty());
}

#[test]
fn search_kind_case_insensitive_via_predicate() {
    let mut sv = SearchVisitor::new(|n: &Node| n.kind().eq_ignore_ascii_case("END"));
    let n = make_node(0, 0, 1);
    sv.enter_node(&n);
    assert_eq!(sv.matches.len(), 1);
}

// =========================================================================
// 9. Multiple visitors sequentially
// =========================================================================

#[test]
fn sequential_stats_then_pretty_print() {
    let n = make_node(5, 0, 10);
    let mut stats = StatsVisitor::default();
    stats.enter_node(&n);
    stats.leave_node(&n);
    assert_eq!(stats.total_nodes, 1);

    let mut pp = PrettyPrintVisitor::new();
    pp.enter_node(&n);
    pp.leave_node(&n);
    assert!(!pp.output().is_empty());
}

#[test]
fn sequential_stats_then_search() {
    let n = make_node(0, 0, 3);
    let mut stats = StatsVisitor::default();
    stats.enter_node(&n);
    assert_eq!(stats.total_nodes, 1);

    let mut search = SearchVisitor::new(|_n: &Node| true);
    search.enter_node(&n);
    assert_eq!(search.matches.len(), 1);
}

#[test]
fn sequential_search_then_pretty_print() {
    let n = make_node(0, 0, 3);
    let mut search = SearchVisitor::new(|_n: &Node| true);
    search.enter_node(&n);
    assert_eq!(search.matches.len(), 1);

    let mut pp = PrettyPrintVisitor::new();
    pp.enter_node(&n);
    assert!(!pp.output().is_empty());
}

#[test]
fn all_three_visitors_on_same_node() {
    let n = make_node(5, 0, 10);
    let mut stats = StatsVisitor::default();
    let mut pp = PrettyPrintVisitor::new();
    let mut search = SearchVisitor::new(|_n: &Node| true);

    stats.enter_node(&n);
    pp.enter_node(&n);
    search.enter_node(&n);

    assert_eq!(stats.total_nodes, 1);
    assert!(!pp.output().is_empty());
    assert_eq!(search.matches.len(), 1);
}

// =========================================================================
// 10. VisitorAction variants
// =========================================================================

#[test]
fn visitor_action_continue_eq() {
    assert_eq!(VisitorAction::Continue, VisitorAction::Continue);
}

#[test]
fn visitor_action_skip_eq() {
    assert_eq!(VisitorAction::SkipChildren, VisitorAction::SkipChildren);
}

#[test]
fn visitor_action_stop_eq() {
    assert_eq!(VisitorAction::Stop, VisitorAction::Stop);
}

#[test]
fn visitor_action_variants_differ() {
    assert_ne!(VisitorAction::Continue, VisitorAction::SkipChildren);
    assert_ne!(VisitorAction::Continue, VisitorAction::Stop);
    assert_ne!(VisitorAction::SkipChildren, VisitorAction::Stop);
}

#[test]
fn visitor_action_debug() {
    assert_eq!(format!("{:?}", VisitorAction::Continue), "Continue");
    assert_eq!(format!("{:?}", VisitorAction::Stop), "Stop");
}

#[test]
fn visitor_action_clone() {
    let a = VisitorAction::SkipChildren;
    let b = a;
    assert_eq!(a, b);
}

// =========================================================================
// 11. Arena — various capacities
// =========================================================================

#[test]
fn arena_capacity_1() {
    let arena = TreeArena::with_capacity(1);
    assert_eq!(arena.capacity(), 1);
    assert!(arena.is_empty());
}

#[test]
fn arena_capacity_10() {
    let arena = TreeArena::with_capacity(10);
    assert_eq!(arena.capacity(), 10);
}

#[test]
fn arena_capacity_100() {
    let arena = TreeArena::with_capacity(100);
    assert_eq!(arena.capacity(), 100);
}

#[test]
fn arena_default_not_empty_capacity() {
    let arena = TreeArena::new();
    assert!(arena.capacity() > 0);
}

#[test]
fn arena_default_is_empty() {
    let arena = TreeArena::default();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
#[should_panic(expected = "Capacity must be > 0")]
fn arena_capacity_zero_panics() {
    let _arena = TreeArena::with_capacity(0);
}

// =========================================================================
// 12. Arena — alloc and access
// =========================================================================

#[test]
fn arena_alloc_leaf_and_read() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(42));
    assert_eq!(arena.get(h).value(), 42);
}

#[test]
fn arena_alloc_branch_and_read() {
    let mut arena = TreeArena::new();
    let c = arena.alloc(TreeNode::leaf(1));
    let p = arena.alloc(TreeNode::branch(vec![c]));
    assert!(arena.get(p).is_branch());
    assert_eq!(arena.get(p).children().len(), 1);
}

#[test]
fn arena_alloc_increments_len() {
    let mut arena = TreeArena::new();
    assert_eq!(arena.len(), 0);
    arena.alloc(TreeNode::leaf(1));
    assert_eq!(arena.len(), 1);
    arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.len(), 2);
}

#[test]
fn arena_branch_with_symbol() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch_with_symbol(99, vec![]));
    assert_eq!(arena.get(h).symbol(), 99);
}

#[test]
fn arena_leaf_is_leaf() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(7));
    assert!(arena.get(h).is_leaf());
    assert!(!arena.get(h).is_branch());
}

#[test]
fn arena_branch_is_branch() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    assert!(arena.get(h).is_branch());
    assert!(!arena.get(h).is_leaf());
}

#[test]
fn arena_leaf_children_empty() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(1));
    assert!(arena.get(h).children().is_empty());
}

// =========================================================================
// 13. Arena — reset and clear
// =========================================================================

#[test]
fn arena_reset_empties() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    arena.reset();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn arena_clear_empties() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.clear();
    assert!(arena.is_empty());
}

#[test]
fn arena_reuse_after_reset() {
    let mut arena = TreeArena::new();
    let h1 = arena.alloc(TreeNode::leaf(10));
    assert_eq!(arena.get(h1).value(), 10);
    arena.reset();
    let h2 = arena.alloc(TreeNode::leaf(20));
    assert_eq!(arena.get(h2).value(), 20);
}

// =========================================================================
// 14. Arena — chunk growth
// =========================================================================

#[test]
fn arena_grows_beyond_initial_capacity() {
    let mut arena = TreeArena::with_capacity(2);
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.num_chunks(), 1);
    arena.alloc(TreeNode::leaf(3));
    assert_eq!(arena.num_chunks(), 2);
}

#[test]
fn arena_capacity_increases_with_growth() {
    let mut arena = TreeArena::with_capacity(2);
    let cap_before = arena.capacity();
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    arena.alloc(TreeNode::leaf(3)); // triggers new chunk
    assert!(arena.capacity() > cap_before);
}

// =========================================================================
// 15. Arena — metrics
// =========================================================================

#[test]
fn arena_metrics_empty() {
    let arena = TreeArena::new();
    let m = arena.metrics();
    assert!(m.is_empty());
    assert_eq!(m.len(), 0);
    assert!(m.capacity() > 0);
    assert_eq!(m.num_chunks(), 1);
}

#[test]
fn arena_metrics_after_alloc() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    let m = arena.metrics();
    assert_eq!(m.len(), 1);
    assert!(!m.is_empty());
}

#[test]
fn arena_memory_usage_positive() {
    let arena = TreeArena::new();
    assert!(arena.memory_usage() > 0);
}

#[test]
fn arena_metrics_memory_matches_direct() {
    let arena = TreeArena::new();
    let m = arena.metrics();
    assert_eq!(m.memory_usage(), arena.memory_usage());
}

// =========================================================================
// 16. Arena — mutable access
// =========================================================================

#[test]
fn arena_get_mut_set_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    arena.get_mut(h).set_value(99);
    assert_eq!(arena.get(h).value(), 99);
}

#[test]
fn arena_get_mut_on_branch_no_crash() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::branch(vec![]));
    // set_value on a branch is a no-op
    arena.get_mut(h).set_value(42);
    assert_eq!(arena.get(h).value(), 0); // branch default symbol
}

// =========================================================================
// 17. NodeHandle — creation and properties
// =========================================================================

#[test]
fn node_handle_new_copy() {
    let h = NodeHandle::new(0, 0);
    let h2 = h;
    assert_eq!(h, h2);
}

#[test]
fn node_handle_equality() {
    let a = NodeHandle::new(1, 2);
    let b = NodeHandle::new(1, 2);
    assert_eq!(a, b);
}

#[test]
fn node_handle_inequality() {
    let a = NodeHandle::new(0, 0);
    let b = NodeHandle::new(0, 1);
    assert_ne!(a, b);
}

#[test]
fn node_handle_debug() {
    let h = NodeHandle::new(3, 7);
    let dbg = format!("{:?}", h);
    assert!(dbg.contains("NodeHandle"));
}

#[test]
fn node_handle_hash_consistency() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    let h = NodeHandle::new(0, 0);
    set.insert(h);
    set.insert(h); // duplicate
    assert_eq!(set.len(), 1);
}

#[test]
fn node_handle_different_chunks_differ() {
    let a = NodeHandle::new(0, 5);
    let b = NodeHandle::new(1, 5);
    assert_ne!(a, b);
}

// =========================================================================
// 18. TreeNode — construction and queries
// =========================================================================

#[test]
fn tree_node_leaf_value() {
    let n = TreeNode::leaf(42);
    assert_eq!(n.value(), 42);
    assert_eq!(n.symbol(), 42);
}

#[test]
fn tree_node_leaf_is_leaf() {
    let n = TreeNode::leaf(0);
    assert!(n.is_leaf());
    assert!(!n.is_branch());
}

#[test]
fn tree_node_branch_default_symbol() {
    let n = TreeNode::branch(vec![]);
    assert_eq!(n.symbol(), 0);
}

#[test]
fn tree_node_branch_with_symbol_value() {
    let n = TreeNode::branch_with_symbol(55, vec![]);
    assert_eq!(n.symbol(), 55);
    assert_eq!(n.value(), 55);
}

#[test]
fn tree_node_branch_children() {
    let h1 = NodeHandle::new(0, 0);
    let h2 = NodeHandle::new(0, 1);
    let n = TreeNode::branch(vec![h1, h2]);
    assert_eq!(n.children().len(), 2);
}

#[test]
fn tree_node_leaf_children_empty() {
    let n = TreeNode::leaf(1);
    assert!(n.children().is_empty());
}

#[test]
fn tree_node_clone_eq() {
    let n = TreeNode::leaf(7);
    let n2 = n.clone();
    assert_eq!(n, n2);
}

#[test]
fn tree_node_debug() {
    let n = TreeNode::leaf(3);
    let dbg = format!("{:?}", n);
    assert!(dbg.contains("TreeNode"));
}

#[test]
fn tree_node_negative_symbol() {
    let n = TreeNode::leaf(-1);
    assert_eq!(n.value(), -1);
}

#[test]
fn tree_node_zero_symbol() {
    let n = TreeNode::leaf(0);
    assert_eq!(n.value(), 0);
}

// =========================================================================
// 19. TreeNodeRef through arena
// =========================================================================

#[test]
fn tree_node_ref_deref_value() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(10));
    let node_ref = arena.get(h);
    assert_eq!(node_ref.value(), 10);
    assert_eq!(node_ref.symbol(), 10);
}

#[test]
fn tree_node_ref_get_ref() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(5));
    let node_ref = arena.get(h);
    let raw = node_ref.get_ref();
    assert_eq!(raw.value(), 5);
}

#[test]
fn tree_node_ref_as_ref_alias() {
    let mut arena = TreeArena::new();
    let h = arena.alloc(TreeNode::leaf(8));
    let node_ref = arena.get(h);
    let raw = node_ref.as_ref();
    assert_eq!(raw.value(), 8);
}

// =========================================================================
// 20. Visitor default trait methods
// =========================================================================

/// Minimal visitor that overrides nothing — all defaults.
struct NoopVisitor;
impl Visitor for NoopVisitor {}

#[test]
fn noop_visitor_enter_returns_continue() {
    let mut v = NoopVisitor;
    let n = make_node(0, 0, 1);
    assert_eq!(v.enter_node(&n), VisitorAction::Continue);
}

#[test]
fn noop_visitor_leave_does_not_panic() {
    let mut v = NoopVisitor;
    let n = make_node(0, 0, 1);
    v.leave_node(&n); // should be no-op
}

#[test]
fn noop_visitor_visit_leaf_does_not_panic() {
    let mut v = NoopVisitor;
    let n = make_node(0, 0, 1);
    v.visit_leaf(&n, "test");
}

#[test]
fn noop_visitor_visit_error_does_not_panic() {
    let mut v = NoopVisitor;
    let n = make_error_node(0, 0, 1);
    v.visit_error(&n);
}
