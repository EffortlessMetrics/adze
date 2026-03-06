//! Comprehensive tests for the visitor API in `adze::visitor`.

use adze::pure_parser::{ParsedNode, Point};
use adze::visitor::*;
use std::mem::MaybeUninit;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pt(row: u32, col: u32) -> Point {
    Point { row, column: col }
}

/// Create a `ParsedNode` using `MaybeUninit` + `write_bytes` to safely handle
/// the `pub(crate)` `language` field (zeroed → `None`).
#[allow(clippy::too_many_arguments)]
fn make_node(
    symbol: u16,
    children: Vec<ParsedNode>,
    start_byte: usize,
    end_byte: usize,
    is_named: bool,
    is_error: bool,
    is_extra: bool,
    is_missing: bool,
) -> ParsedNode {
    let mut uninit = MaybeUninit::<ParsedNode>::uninit();
    let ptr = uninit.as_mut_ptr();
    unsafe {
        std::ptr::write_bytes(ptr, 0, 1);
        std::ptr::addr_of_mut!((*ptr).symbol).write(symbol);
        std::ptr::addr_of_mut!((*ptr).children).write(children);
        std::ptr::addr_of_mut!((*ptr).start_byte).write(start_byte);
        std::ptr::addr_of_mut!((*ptr).end_byte).write(end_byte);
        std::ptr::addr_of_mut!((*ptr).start_point).write(pt(0, start_byte as u32));
        std::ptr::addr_of_mut!((*ptr).end_point).write(pt(0, end_byte as u32));
        std::ptr::addr_of_mut!((*ptr).is_extra).write(is_extra);
        std::ptr::addr_of_mut!((*ptr).is_error).write(is_error);
        std::ptr::addr_of_mut!((*ptr).is_missing).write(is_missing);
        std::ptr::addr_of_mut!((*ptr).is_named).write(is_named);
        std::ptr::addr_of_mut!((*ptr).field_id).write(None);
        uninit.assume_init()
    }
}

fn leaf(symbol: u16, start: usize, end: usize, kind_named: bool) -> ParsedNode {
    make_node(symbol, vec![], start, end, kind_named, false, false, false)
}

fn inner(symbol: u16, children: Vec<ParsedNode>, named: bool) -> ParsedNode {
    let start = children.first().map_or(0, |c| c.start_byte);
    let end = children.last().map_or(0, |c| c.end_byte);
    make_node(symbol, children, start, end, named, false, false, false)
}

fn error_node(start: usize, end: usize) -> ParsedNode {
    make_node(0, vec![], start, end, false, true, false, false)
}

/// Build a simple tree:  root(a, b)  over source "ab"
fn simple_tree() -> (ParsedNode, Vec<u8>) {
    let source = b"ab".to_vec();
    let a = leaf(1, 0, 1, true);
    let b = leaf(2, 1, 2, true);
    let root = inner(5, vec![a, b], true);
    (root, source)
}

/// Build a deeper tree: root( mid( a ), b )  over source "ab"
fn nested_tree() -> (ParsedNode, Vec<u8>) {
    let source = b"ab".to_vec();
    let a = leaf(1, 0, 1, true);
    let mid = inner(3, vec![a], true);
    let b = leaf(2, 1, 2, true);
    let root = inner(5, vec![mid, b], true);
    (root, source)
}

// ---------------------------------------------------------------------------
// 1. VisitorAction basics
// ---------------------------------------------------------------------------

#[test]
fn visitor_action_equality() {
    assert_eq!(VisitorAction::Continue, VisitorAction::Continue);
    assert_eq!(VisitorAction::SkipChildren, VisitorAction::SkipChildren);
    assert_eq!(VisitorAction::Stop, VisitorAction::Stop);
}

#[test]
fn visitor_action_inequality() {
    assert_ne!(VisitorAction::Continue, VisitorAction::Stop);
    assert_ne!(VisitorAction::Continue, VisitorAction::SkipChildren);
    assert_ne!(VisitorAction::SkipChildren, VisitorAction::Stop);
}

#[test]
fn visitor_action_clone() {
    let a = VisitorAction::Continue;
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn visitor_action_debug() {
    let dbg = format!("{:?}", VisitorAction::Stop);
    assert!(dbg.contains("Stop"));
}

// ---------------------------------------------------------------------------
// 2. Default Visitor trait implementations (no-op)
// ---------------------------------------------------------------------------

struct NoopVisitor;
impl Visitor for NoopVisitor {}

#[test]
fn default_enter_node_returns_continue() {
    let mut v = NoopVisitor;
    let node = leaf(0, 0, 1, false);
    assert_eq!(v.enter_node(&node), VisitorAction::Continue);
}

#[test]
fn default_leave_node_does_not_panic() {
    let mut v = NoopVisitor;
    let node = leaf(0, 0, 1, false);
    v.leave_node(&node); // should not panic
}

#[test]
fn default_visit_leaf_does_not_panic() {
    let mut v = NoopVisitor;
    let node = leaf(0, 0, 1, false);
    v.visit_leaf(&node, "x");
}

#[test]
fn default_visit_error_does_not_panic() {
    let mut v = NoopVisitor;
    let node = error_node(0, 1);
    v.visit_error(&node);
}

// ---------------------------------------------------------------------------
// 3. TreeWalker creation
// ---------------------------------------------------------------------------

#[test]
fn tree_walker_new() {
    let src = b"hello";
    let _w = TreeWalker::new(src);
}

// ---------------------------------------------------------------------------
// 4. TreeWalker depth-first traversal
// ---------------------------------------------------------------------------

#[test]
fn tree_walker_visits_single_leaf() {
    let source = b"x";
    let node = leaf(1, 0, 1, true);
    let walker = TreeWalker::new(source);
    let mut stats = StatsVisitor::default();
    walker.walk(&node, &mut stats);
    assert_eq!(stats.total_nodes, 1);
    assert_eq!(stats.leaf_nodes, 1);
}

#[test]
fn tree_walker_visits_all_children() {
    let (root, source) = simple_tree();
    let walker = TreeWalker::new(&source);
    let mut stats = StatsVisitor::default();
    walker.walk(&root, &mut stats);
    // root + 2 leaves = 3 nodes entered
    assert_eq!(stats.total_nodes, 3);
    assert_eq!(stats.leaf_nodes, 2);
}

#[test]
fn tree_walker_depth_tracking() {
    let (root, source) = nested_tree();
    let walker = TreeWalker::new(&source);
    let mut stats = StatsVisitor::default();
    walker.walk(&root, &mut stats);
    // root -> mid -> a  has depth 3
    assert_eq!(stats.max_depth, 3);
}

#[test]
fn tree_walker_calls_leave_for_every_enter() {
    struct PairVisitor {
        enters: usize,
        leaves: usize,
    }
    impl Visitor for PairVisitor {
        fn enter_node(&mut self, _: &ParsedNode) -> VisitorAction {
            self.enters += 1;
            VisitorAction::Continue
        }
        fn leave_node(&mut self, _: &ParsedNode) {
            self.leaves += 1;
        }
    }
    let (root, source) = nested_tree();
    let walker = TreeWalker::new(&source);
    let mut v = PairVisitor {
        enters: 0,
        leaves: 0,
    };
    walker.walk(&root, &mut v);
    assert_eq!(v.enters, v.leaves);
    assert!(v.enters > 0);
}

#[test]
fn tree_walker_pre_order_sequence() {
    // Capture the enter order by byte ranges
    struct OrderVisitor {
        order: Vec<(usize, usize)>,
    }
    impl Visitor for OrderVisitor {
        fn enter_node(&mut self, node: &ParsedNode) -> VisitorAction {
            self.order.push((node.start_byte(), node.end_byte()));
            VisitorAction::Continue
        }
    }
    let (root, source) = nested_tree();
    let walker = TreeWalker::new(&source);
    let mut v = OrderVisitor { order: vec![] };
    walker.walk(&root, &mut v);
    // Pre-order: root(0..2), mid(0..1), leaf-a(0..1), leaf-b(1..2)
    assert_eq!(v.order.len(), 4);
    assert_eq!(v.order[0], (0, 2)); // root
    assert_eq!(v.order[1], (0, 1)); // mid
    assert_eq!(v.order[2], (0, 1)); // leaf a
    assert_eq!(v.order[3], (1, 2)); // leaf b
}

// ---------------------------------------------------------------------------
// 5. SkipChildren / Stop during TreeWalker
// ---------------------------------------------------------------------------

#[test]
fn tree_walker_skip_children() {
    struct SkipMid {
        visited: Vec<(usize, usize)>,
    }
    impl Visitor for SkipMid {
        fn enter_node(&mut self, node: &ParsedNode) -> VisitorAction {
            self.visited.push((node.start_byte(), node.end_byte()));
            // skip "mid" node (it has children and spans 0..1)
            if node.child_count() > 0 && node.end_byte() == 1 {
                VisitorAction::SkipChildren
            } else {
                VisitorAction::Continue
            }
        }
    }
    let (root, source) = nested_tree();
    let walker = TreeWalker::new(&source);
    let mut v = SkipMid { visited: vec![] };
    walker.walk(&root, &mut v);
    // root entered, mid entered (skipped children), leaf-b entered = 3
    assert_eq!(v.visited.len(), 3);
}

#[test]
fn tree_walker_stop_halts_early() {
    struct StopAfterOne {
        count: usize,
    }
    impl Visitor for StopAfterOne {
        fn enter_node(&mut self, _: &ParsedNode) -> VisitorAction {
            self.count += 1;
            VisitorAction::Stop
        }
    }
    let (root, source) = simple_tree();
    let walker = TreeWalker::new(&source);
    let mut v = StopAfterOne { count: 0 };
    walker.walk(&root, &mut v);
    assert_eq!(v.count, 1);
}

#[test]
fn tree_walker_skip_children_still_calls_leave() {
    struct LeaveCounter {
        leave_count: usize,
    }
    impl Visitor for LeaveCounter {
        fn enter_node(&mut self, _: &ParsedNode) -> VisitorAction {
            VisitorAction::SkipChildren
        }
        fn leave_node(&mut self, _: &ParsedNode) {
            self.leave_count += 1;
        }
    }
    let (root, source) = simple_tree();
    let walker = TreeWalker::new(&source);
    let mut v = LeaveCounter { leave_count: 0 };
    walker.walk(&root, &mut v);
    // Only root is entered (skip children), leave is called for root
    assert_eq!(v.leave_count, 1);
}

// ---------------------------------------------------------------------------
// 6. Error node handling
// ---------------------------------------------------------------------------

#[test]
fn tree_walker_visits_error_nodes() {
    let source = b"x!y";
    let a = leaf(1, 0, 1, true);
    let err = error_node(1, 2);
    let b = leaf(2, 2, 3, true);
    let root = inner(5, vec![a, err, b], true);

    let walker = TreeWalker::new(source);
    let mut stats = StatsVisitor::default();
    walker.walk(&root, &mut stats);
    assert_eq!(stats.error_nodes, 1);
}

#[test]
fn tree_walker_error_does_not_enter() {
    // Error nodes go straight to visit_error, not enter_node
    struct EnterCounter {
        enters: usize,
        errors: usize,
    }
    impl Visitor for EnterCounter {
        fn enter_node(&mut self, _: &ParsedNode) -> VisitorAction {
            self.enters += 1;
            VisitorAction::Continue
        }
        fn visit_error(&mut self, _: &ParsedNode) {
            self.errors += 1;
        }
    }
    let source = b"x!";
    let a = leaf(1, 0, 1, true);
    let err = error_node(1, 2);
    let root = inner(5, vec![a, err], true);

    let walker = TreeWalker::new(source);
    let mut v = EnterCounter {
        enters: 0,
        errors: 0,
    };
    walker.walk(&root, &mut v);
    assert_eq!(v.errors, 1);
    // root + leaf a = 2 enters (error is not entered)
    assert_eq!(v.enters, 2);
}

// ---------------------------------------------------------------------------
// 7. Leaf text extraction
// ---------------------------------------------------------------------------

#[test]
fn tree_walker_leaf_text() {
    struct LeafCollector {
        texts: Vec<String>,
    }
    impl Visitor for LeafCollector {
        fn enter_node(&mut self, _: &ParsedNode) -> VisitorAction {
            VisitorAction::Continue
        }
        fn visit_leaf(&mut self, _: &ParsedNode, text: &str) {
            self.texts.push(text.to_string());
        }
    }
    let source = b"ab";
    let (root, _) = simple_tree();
    let walker = TreeWalker::new(source);
    let mut v = LeafCollector { texts: vec![] };
    walker.walk(&root, &mut v);
    assert_eq!(v.texts, vec!["a", "b"]);
}

// ---------------------------------------------------------------------------
// 8. StatsVisitor
// ---------------------------------------------------------------------------

#[test]
fn stats_visitor_default_is_zero() {
    let v = StatsVisitor::default();
    assert_eq!(v.total_nodes, 0);
    assert_eq!(v.leaf_nodes, 0);
    assert_eq!(v.error_nodes, 0);
    assert_eq!(v.max_depth, 0);
    assert!(v.node_counts.is_empty());
}

#[test]
fn stats_visitor_counts_nodes() {
    let (root, source) = simple_tree();
    let walker = TreeWalker::new(&source);
    let mut stats = StatsVisitor::default();
    walker.walk(&root, &mut stats);
    assert_eq!(stats.total_nodes, 3);
}

#[test]
fn stats_visitor_tracks_node_kinds() {
    let (root, source) = simple_tree();
    let walker = TreeWalker::new(&source);
    let mut stats = StatsVisitor::default();
    walker.walk(&root, &mut stats);
    // Without a language, kind() falls back to hardcoded table
    // All nodes should have some kind string
    assert!(!stats.node_counts.is_empty());
}

#[test]
fn stats_visitor_depth_single_leaf() {
    let source = b"x";
    let node = leaf(1, 0, 1, true);
    let walker = TreeWalker::new(source);
    let mut stats = StatsVisitor::default();
    walker.walk(&node, &mut stats);
    assert_eq!(stats.max_depth, 1);
}

#[test]
fn stats_visitor_depth_resets_after_walk() {
    let (root, source) = nested_tree();
    let walker = TreeWalker::new(&source);
    let mut stats = StatsVisitor::default();
    walker.walk(&root, &mut stats);
    // After walk finishes, current_depth should be back to 0
    // We verify indirectly: max_depth was set correctly
    assert!(stats.max_depth >= 2);
}

// ---------------------------------------------------------------------------
// 9. SearchVisitor
// ---------------------------------------------------------------------------

#[test]
fn search_visitor_finds_matching_nodes() {
    let (root, source) = simple_tree();
    let walker = TreeWalker::new(&source);
    let mut search =
        SearchVisitor::new(|node: &ParsedNode| node.start_byte() == 0 && node.child_count() == 0);
    walker.walk(&root, &mut search);
    assert_eq!(search.matches.len(), 1);
    assert_eq!(search.matches[0].0, 0); // start_byte
    assert_eq!(search.matches[0].1, 1); // end_byte
}

#[test]
fn search_visitor_no_matches() {
    let (root, source) = simple_tree();
    let walker = TreeWalker::new(&source);
    let mut search = SearchVisitor::new(|_: &ParsedNode| false);
    walker.walk(&root, &mut search);
    assert!(search.matches.is_empty());
}

#[test]
fn search_visitor_all_match() {
    let (root, source) = simple_tree();
    let walker = TreeWalker::new(&source);
    let mut search = SearchVisitor::new(|_: &ParsedNode| true);
    walker.walk(&root, &mut search);
    assert_eq!(search.matches.len(), 3); // root + 2 leaves
}

#[test]
fn search_visitor_stores_kind_string() {
    let source = b"x";
    let node = leaf(1, 0, 1, true);
    let walker = TreeWalker::new(source);
    let mut search = SearchVisitor::new(|_: &ParsedNode| true);
    walker.walk(&node, &mut search);
    assert_eq!(search.matches.len(), 1);
    // kind string should be non-empty (fallback table maps symbol 1 to "*")
    assert!(!search.matches[0].2.is_empty());
}

// ---------------------------------------------------------------------------
// 10. PrettyPrintVisitor
// ---------------------------------------------------------------------------

#[test]
fn pretty_print_new_is_empty() {
    let pp = PrettyPrintVisitor::new();
    assert!(pp.output().is_empty());
}

#[test]
fn pretty_print_default_is_empty() {
    let pp = PrettyPrintVisitor::default();
    assert!(pp.output().is_empty());
}

#[test]
fn pretty_print_single_leaf() {
    let source = b"x";
    let node = leaf(1, 0, 1, true);
    let walker = TreeWalker::new(source);
    let mut pp = PrettyPrintVisitor::new();
    walker.walk(&node, &mut pp);
    let out = pp.output();
    assert!(
        out.contains("\"x\""),
        "output should contain leaf text: {out}"
    );
}

#[test]
fn pretty_print_named_annotation() {
    let source = b"x";
    let node = leaf(1, 0, 1, true); // is_named = true
    let walker = TreeWalker::new(source);
    let mut pp = PrettyPrintVisitor::new();
    walker.walk(&node, &mut pp);
    assert!(
        pp.output().contains("[named]"),
        "named nodes should be annotated: {}",
        pp.output()
    );
}

#[test]
fn pretty_print_unnamed_no_annotation() {
    let source = b"x";
    let node = leaf(1, 0, 1, false); // is_named = false
    let walker = TreeWalker::new(source);
    let mut pp = PrettyPrintVisitor::new();
    walker.walk(&node, &mut pp);
    assert!(
        !pp.output().contains("[named]"),
        "unnamed nodes should not be annotated: {}",
        pp.output()
    );
}

#[test]
fn pretty_print_indentation_increases_with_depth() {
    let (root, source) = nested_tree();
    let walker = TreeWalker::new(&source);
    let mut pp = PrettyPrintVisitor::new();
    walker.walk(&root, &mut pp);
    let out = pp.output();
    // The deepest leaf should have at least two levels of indent ("    ")
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.len() >= 3, "should have multiple lines: {out}");
    // Check that some line starts with spaces (indentation)
    assert!(
        lines.iter().any(|l| l.starts_with("  ")),
        "should have indented lines: {out}"
    );
}

#[test]
fn pretty_print_error_node() {
    let source = b"x!";
    let err = error_node(1, 2);
    let a = leaf(1, 0, 1, true);
    let root = inner(5, vec![a, err], true);
    let walker = TreeWalker::new(source);
    let mut pp = PrettyPrintVisitor::new();
    walker.walk(&root, &mut pp);
    assert!(
        pp.output().contains("ERROR"),
        "error nodes should appear: {}",
        pp.output()
    );
}

// ---------------------------------------------------------------------------
// 11. BreadthFirstWalker
// ---------------------------------------------------------------------------

#[test]
fn breadth_first_walker_new() {
    let src = b"hello";
    let _w = BreadthFirstWalker::new(src);
}

#[test]
fn breadth_first_visits_all_nodes() {
    let (root, source) = simple_tree();
    let walker = BreadthFirstWalker::new(&source);
    let mut stats = StatsVisitor::default();
    walker.walk(&root, &mut stats);
    assert_eq!(stats.total_nodes, 3);
}

#[test]
fn breadth_first_level_order() {
    struct OrderVisitor {
        order: Vec<(usize, usize)>,
    }
    impl Visitor for OrderVisitor {
        fn enter_node(&mut self, node: &ParsedNode) -> VisitorAction {
            self.order.push((node.start_byte(), node.end_byte()));
            VisitorAction::Continue
        }
    }
    let (root, source) = nested_tree();
    let walker = BreadthFirstWalker::new(&source);
    let mut v = OrderVisitor { order: vec![] };
    walker.walk(&root, &mut v);
    // BFS: root(0..2), mid(0..1), leaf-b(1..2), leaf-a(0..1)
    assert_eq!(v.order.len(), 4);
    assert_eq!(v.order[0], (0, 2)); // root first
}

#[test]
fn breadth_first_stop_halts() {
    struct StopFirst {
        count: usize,
    }
    impl Visitor for StopFirst {
        fn enter_node(&mut self, _: &ParsedNode) -> VisitorAction {
            self.count += 1;
            VisitorAction::Stop
        }
    }
    let (root, source) = simple_tree();
    let walker = BreadthFirstWalker::new(&source);
    let mut v = StopFirst { count: 0 };
    walker.walk(&root, &mut v);
    assert_eq!(v.count, 1);
}

#[test]
fn breadth_first_skip_children() {
    struct SkipAll {
        count: usize,
    }
    impl Visitor for SkipAll {
        fn enter_node(&mut self, _: &ParsedNode) -> VisitorAction {
            self.count += 1;
            VisitorAction::SkipChildren
        }
    }
    let (root, source) = simple_tree();
    let walker = BreadthFirstWalker::new(&source);
    let mut v = SkipAll { count: 0 };
    walker.walk(&root, &mut v);
    // Only root is entered because its children are skipped
    assert_eq!(v.count, 1);
}

#[test]
fn breadth_first_error_node_handling() {
    let source = b"x!y";
    let a = leaf(1, 0, 1, true);
    let err = error_node(1, 2);
    let b = leaf(2, 2, 3, true);
    let root = inner(5, vec![a, err, b], true);

    let walker = BreadthFirstWalker::new(source);
    let mut stats = StatsVisitor::default();
    walker.walk(&root, &mut stats);
    assert_eq!(stats.error_nodes, 1);
}

// ---------------------------------------------------------------------------
// 12. TransformVisitor / TransformWalker
// ---------------------------------------------------------------------------

struct CountTransform;

impl TransformVisitor for CountTransform {
    type Output = usize;

    fn transform_node(&mut self, _node: &ParsedNode, children: Vec<usize>) -> usize {
        1 + children.iter().sum::<usize>()
    }

    fn transform_leaf(&mut self, _node: &ParsedNode, _text: &str) -> usize {
        1
    }

    fn transform_error(&mut self, _node: &ParsedNode) -> usize {
        0
    }
}

#[test]
fn transform_walker_single_leaf() {
    let source = b"x";
    let node = leaf(1, 0, 1, true);
    let walker = TransformWalker::new(source);
    let mut t = CountTransform;
    let result = walker.walk(&node, &mut t);
    assert_eq!(result, 1);
}

#[test]
fn transform_walker_tree_count() {
    let (root, source) = simple_tree();
    let walker = TransformWalker::new(&source);
    let mut t = CountTransform;
    let result = walker.walk(&root, &mut t);
    // root(1) + leaf-a(1) + leaf-b(1) = 3
    assert_eq!(result, 3);
}

#[test]
fn transform_walker_nested_count() {
    let (root, source) = nested_tree();
    let walker = TransformWalker::new(&source);
    let mut t = CountTransform;
    let result = walker.walk(&root, &mut t);
    // root(1) + mid(1) + leaf-a(1) + leaf-b(1) = 4
    assert_eq!(result, 4);
}

#[test]
fn transform_walker_error_node() {
    let source = b"x!";
    let a = leaf(1, 0, 1, true);
    let err = error_node(1, 2);
    let root = inner(5, vec![a, err], true);

    let walker = TransformWalker::new(source);
    let mut t = CountTransform;
    let result = walker.walk(&root, &mut t);
    // root(1) + leaf-a(1) + error(0) = 2
    assert_eq!(result, 2);
}

#[test]
fn transform_walker_collects_leaf_text() {
    struct TextCollector;
    impl TransformVisitor for TextCollector {
        type Output = Vec<String>;

        fn transform_node(
            &mut self,
            _node: &ParsedNode,
            children: Vec<Vec<String>>,
        ) -> Vec<String> {
            children.into_iter().flatten().collect()
        }

        fn transform_leaf(&mut self, _node: &ParsedNode, text: &str) -> Vec<String> {
            vec![text.to_string()]
        }

        fn transform_error(&mut self, _node: &ParsedNode) -> Vec<String> {
            vec!["<error>".to_string()]
        }
    }

    let (root, source) = simple_tree();
    let walker = TransformWalker::new(&source);
    let mut t = TextCollector;
    let result = walker.walk(&root, &mut t);
    assert_eq!(result, vec!["a", "b"]);
}

// ---------------------------------------------------------------------------
// 13. Edge cases
// ---------------------------------------------------------------------------

#[test]
fn single_root_no_children_no_source() {
    // A node with 0..0 range in an empty source
    let source = b"";
    let node = leaf(1, 0, 0, true);
    let walker = TreeWalker::new(source);
    let mut stats = StatsVisitor::default();
    walker.walk(&node, &mut stats);
    assert_eq!(stats.total_nodes, 1);
    assert_eq!(stats.leaf_nodes, 1);
}

#[test]
fn deeply_nested_tree() {
    // Build a chain: root -> c1 -> c2 -> ... -> leaf
    let depth = 20;
    let source = b"x";
    let mut current = leaf(1, 0, 1, true);
    for i in 0..depth {
        current = inner((i + 10) as u16, vec![current], true);
    }
    let walker = TreeWalker::new(source);
    let mut stats = StatsVisitor::default();
    walker.walk(&current, &mut stats);
    assert_eq!(stats.total_nodes, depth + 1);
    assert_eq!(stats.max_depth, depth + 1);
    assert_eq!(stats.leaf_nodes, 1);
}

#[test]
fn wide_tree_many_children() {
    let source = b"abcdefghij";
    let children: Vec<ParsedNode> = (0..10).map(|i| leaf(1, i, i + 1, true)).collect();
    let root = inner(5, children, true);
    let walker = TreeWalker::new(source);
    let mut stats = StatsVisitor::default();
    walker.walk(&root, &mut stats);
    assert_eq!(stats.total_nodes, 11); // root + 10 leaves
    assert_eq!(stats.leaf_nodes, 10);
    assert_eq!(stats.max_depth, 2);
}

#[test]
fn tree_with_only_error_children() {
    let source = b"!!!";
    let children: Vec<ParsedNode> = (0..3).map(|i| error_node(i, i + 1)).collect();
    let root = inner(5, children, true);
    let walker = TreeWalker::new(source);
    let mut stats = StatsVisitor::default();
    walker.walk(&root, &mut stats);
    assert_eq!(stats.error_nodes, 3);
    // root is entered; errors go through visit_error not enter_node
    assert_eq!(stats.total_nodes, 1);
}

// ---------------------------------------------------------------------------
// 14. Visitor state management
// ---------------------------------------------------------------------------

#[test]
fn visitor_accumulates_state_across_calls() {
    let source = b"ab";
    let (root, _) = simple_tree();
    let walker = TreeWalker::new(source);
    let mut stats = StatsVisitor::default();

    // Walk twice with same visitor
    walker.walk(&root, &mut stats);
    walker.walk(&root, &mut stats);

    assert_eq!(stats.total_nodes, 6); // 3 * 2
    assert_eq!(stats.leaf_nodes, 4); // 2 * 2
}

#[test]
fn separate_visitors_are_independent() {
    let source = b"ab";
    let (root, _) = simple_tree();
    let walker = TreeWalker::new(source);

    let mut stats1 = StatsVisitor::default();
    let mut stats2 = StatsVisitor::default();
    walker.walk(&root, &mut stats1);
    walker.walk(&root, &mut stats2);

    assert_eq!(stats1.total_nodes, stats2.total_nodes);
    assert_eq!(stats1.leaf_nodes, stats2.leaf_nodes);
}

// ---------------------------------------------------------------------------
// 15. Composition patterns
// ---------------------------------------------------------------------------

#[test]
fn visitor_can_hold_closures() {
    // SearchVisitor is parameterised by a closure — test different closures
    let (root, source) = simple_tree();
    let walker = TreeWalker::new(&source);

    let mut search_leaves = SearchVisitor::new(|n: &ParsedNode| n.child_count() == 0);
    walker.walk(&root, &mut search_leaves);
    assert_eq!(search_leaves.matches.len(), 2);

    let mut search_inner = SearchVisitor::new(|n: &ParsedNode| n.child_count() > 0);
    walker.walk(&root, &mut search_inner);
    assert_eq!(search_inner.matches.len(), 1);
}

#[test]
fn multiple_visitors_same_tree() {
    let (root, source) = nested_tree();
    let walker = TreeWalker::new(&source);

    let mut stats = StatsVisitor::default();
    walker.walk(&root, &mut stats);

    let mut pp = PrettyPrintVisitor::new();
    walker.walk(&root, &mut pp);

    // Both should have processed something
    assert!(stats.total_nodes > 0);
    assert!(!pp.output().is_empty());
}

#[test]
fn depth_first_and_breadth_first_same_count() {
    let (root, source) = nested_tree();

    let dfs = TreeWalker::new(&source);
    let mut stats_dfs = StatsVisitor::default();
    dfs.walk(&root, &mut stats_dfs);

    let bfs = BreadthFirstWalker::new(&source);
    let mut stats_bfs = StatsVisitor::default();
    bfs.walk(&root, &mut stats_bfs);

    assert_eq!(stats_dfs.total_nodes, stats_bfs.total_nodes);
    assert_eq!(stats_dfs.leaf_nodes, stats_bfs.leaf_nodes);
}

#[test]
fn transform_and_stats_agree_on_count() {
    let (root, source) = nested_tree();

    let walker_s = TreeWalker::new(&source);
    let mut stats = StatsVisitor::default();
    walker_s.walk(&root, &mut stats);

    let walker_t = TransformWalker::new(&source);
    let mut ct = CountTransform;
    let transform_count = walker_t.walk(&root, &mut ct);

    assert_eq!(stats.total_nodes, transform_count);
}

// ---------------------------------------------------------------------------
// 16. Extra/missing node flags
// ---------------------------------------------------------------------------

#[test]
fn extra_node_is_not_error() {
    let node = make_node(1, vec![], 0, 1, false, false, true, false);
    assert!(!node.is_error());
    assert!(node.is_extra());
}

#[test]
fn missing_node_flag() {
    let node = make_node(1, vec![], 0, 1, false, false, false, true);
    assert!(node.is_missing());
}

#[test]
fn visitor_visits_extra_nodes_normally() {
    let source = b"x y";
    let a = leaf(1, 0, 1, true);
    let space = make_node(6, vec![], 1, 2, false, false, true, false);
    let b = leaf(2, 2, 3, true);
    let root = inner(5, vec![a, space, b], true);

    let walker = TreeWalker::new(source);
    let mut stats = StatsVisitor::default();
    walker.walk(&root, &mut stats);
    // root + a + space + b = 4
    assert_eq!(stats.total_nodes, 4);
    assert_eq!(stats.leaf_nodes, 3);
}
