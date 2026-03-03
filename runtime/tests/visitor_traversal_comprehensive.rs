#![allow(clippy::needless_range_loop)]
//! Comprehensive visitor traversal tests for the adze runtime.

use adze::pure_parser::{ParsedNode, Point};
use adze::visitor::{
    BreadthFirstWalker, PrettyPrintVisitor, SearchVisitor, StatsVisitor, TransformVisitor,
    TransformWalker, TreeWalker, Visitor, VisitorAction,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pt(row: u32, col: u32) -> Point {
    Point { row, column: col }
}

/// Create a `ParsedNode` with full control. The `language` field is
/// `pub(crate)`, so we zero-initialize via `MaybeUninit`.
fn make_node(
    symbol: u16,
    children: Vec<ParsedNode>,
    start: usize,
    end: usize,
    is_error: bool,
    is_named: bool,
) -> ParsedNode {
    use std::mem::MaybeUninit;

    let mut uninit = MaybeUninit::<ParsedNode>::uninit();
    let ptr = uninit.as_mut_ptr();
    unsafe {
        std::ptr::write_bytes(ptr, 0, 1);
        std::ptr::addr_of_mut!((*ptr).symbol).write(symbol);
        std::ptr::addr_of_mut!((*ptr).children).write(children);
        std::ptr::addr_of_mut!((*ptr).start_byte).write(start);
        std::ptr::addr_of_mut!((*ptr).end_byte).write(end);
        std::ptr::addr_of_mut!((*ptr).start_point).write(pt(0, start as u32));
        std::ptr::addr_of_mut!((*ptr).end_point).write(pt(0, end as u32));
        std::ptr::addr_of_mut!((*ptr).is_extra).write(false);
        std::ptr::addr_of_mut!((*ptr).is_error).write(is_error);
        std::ptr::addr_of_mut!((*ptr).is_missing).write(false);
        std::ptr::addr_of_mut!((*ptr).is_named).write(is_named);
        std::ptr::addr_of_mut!((*ptr).field_id).write(None);
        uninit.assume_init()
    }
}

fn leaf(symbol: u16, start: usize, end: usize) -> ParsedNode {
    make_node(symbol, vec![], start, end, false, true)
}

fn unnamed_leaf(symbol: u16, start: usize, end: usize) -> ParsedNode {
    make_node(symbol, vec![], start, end, false, false)
}

fn interior(symbol: u16, children: Vec<ParsedNode>) -> ParsedNode {
    let start = children.first().map_or(0, |c| c.start_byte);
    let end = children.last().map_or(0, |c| c.end_byte);
    make_node(symbol, children, start, end, false, true)
}

fn error_node(start: usize, end: usize) -> ParsedNode {
    make_node(0, vec![], start, end, true, false)
}

/// Build:  root(10)( a(1), mid(11)( b(2), c(3) ), d(4) )
/// Source: "abcd"
fn sample_tree() -> (ParsedNode, Vec<u8>) {
    let source = b"abcd".to_vec();
    let a = leaf(1, 0, 1);
    let b = leaf(2, 1, 2);
    let c = unnamed_leaf(3, 2, 3);
    let mid = interior(11, vec![b, c]);
    let d = leaf(4, 3, 4);
    let root = interior(10, vec![a, mid, d]);
    (root, source)
}

/// Single leaf node. Source: "x"
fn single_node_tree() -> (ParsedNode, Vec<u8>) {
    (leaf(1, 0, 1), b"x".to_vec())
}

/// Deep chain:  root -> c1 -> c2 -> ... -> leaf   (depth levels)
fn deep_tree(depth: usize) -> (ParsedNode, Vec<u8>) {
    let source = b"z".to_vec();
    let mut node = leaf(1, 0, 1);
    for i in 0..depth {
        node = interior((i as u16).wrapping_add(2), vec![node]);
    }
    (node, source)
}

/// Wide tree: root with `width` leaf children.
/// Source: "a" repeated `width` times.
fn wide_tree(width: usize) -> (ParsedNode, Vec<u8>) {
    let source: Vec<u8> = (0..width).map(|_| b'a').collect();
    let children: Vec<ParsedNode> = (0..width)
        .map(|i| leaf((i as u16).wrapping_add(1), i, i + 1))
        .collect();
    let root = interior(100, children);
    (root, source)
}

// ---------------------------------------------------------------------------
// Custom visitors used across multiple tests
// ---------------------------------------------------------------------------

/// Records the order of `enter_node` calls via `start_byte`.
struct OrderVisitor {
    enter_order: Vec<usize>,
    leave_order: Vec<usize>,
}

impl OrderVisitor {
    fn new() -> Self {
        Self {
            enter_order: vec![],
            leave_order: vec![],
        }
    }
}

impl Visitor for OrderVisitor {
    fn enter_node(&mut self, node: &ParsedNode) -> VisitorAction {
        self.enter_order.push(node.start_byte());
        VisitorAction::Continue
    }
    fn leave_node(&mut self, node: &ParsedNode) {
        self.leave_order.push(node.start_byte());
    }
}

/// Accumulates leaf text.
struct LeafCollector {
    texts: Vec<String>,
}

impl LeafCollector {
    fn new() -> Self {
        Self { texts: vec![] }
    }
}

impl Visitor for LeafCollector {
    fn enter_node(&mut self, _: &ParsedNode) -> VisitorAction {
        VisitorAction::Continue
    }
    fn visit_leaf(&mut self, _: &ParsedNode, text: &str) {
        self.texts.push(text.to_string());
    }
}

/// Stops after visiting `limit` nodes.
struct LimitVisitor {
    limit: usize,
    count: usize,
}

impl LimitVisitor {
    fn new(limit: usize) -> Self {
        Self { limit, count: 0 }
    }
}

impl Visitor for LimitVisitor {
    fn enter_node(&mut self, _: &ParsedNode) -> VisitorAction {
        self.count += 1;
        if self.count >= self.limit {
            VisitorAction::Stop
        } else {
            VisitorAction::Continue
        }
    }
}

/// Skips children of nodes whose `start_byte` is in `skip_set`.
struct SkipVisitor {
    skip_starts: Vec<usize>,
    visited: Vec<usize>,
}

impl SkipVisitor {
    fn new(skip_starts: Vec<usize>) -> Self {
        Self {
            skip_starts,
            visited: vec![],
        }
    }
}

impl Visitor for SkipVisitor {
    fn enter_node(&mut self, node: &ParsedNode) -> VisitorAction {
        self.visited.push(node.start_byte());
        if self.skip_starts.contains(&node.start_byte()) {
            VisitorAction::SkipChildren
        } else {
            VisitorAction::Continue
        }
    }
}

// ===========================================================================
// DFS TRAVERSAL
// ===========================================================================

#[test]
fn dfs_enter_order() {
    let (tree, source) = sample_tree();
    let walker = TreeWalker::new(&source);
    let mut v = OrderVisitor::new();
    walker.walk(&tree, &mut v);
    // Pre-order DFS: root(0), a(0), mid(1), b(1), c(2), d(3)
    assert_eq!(v.enter_order, vec![0, 0, 1, 1, 2, 3]);
}

#[test]
fn dfs_leave_order() {
    let (tree, source) = sample_tree();
    let walker = TreeWalker::new(&source);
    let mut v = OrderVisitor::new();
    walker.walk(&tree, &mut v);
    // Post-order for leave: a(0), b(1), c(2), mid(1), d(3), root(0)
    assert_eq!(v.leave_order, vec![0, 1, 2, 1, 3, 0]);
}

#[test]
fn dfs_leaf_text_collection() {
    let (tree, source) = sample_tree();
    let walker = TreeWalker::new(&source);
    let mut v = LeafCollector::new();
    walker.walk(&tree, &mut v);
    assert_eq!(v.texts, vec!["a", "b", "c", "d"]);
}

#[test]
fn dfs_stats_visitor_node_counts() {
    let (tree, source) = sample_tree();
    let walker = TreeWalker::new(&source);
    let mut stats = StatsVisitor::default();
    walker.walk(&tree, &mut stats);
    assert_eq!(stats.total_nodes, 6);
    assert_eq!(stats.leaf_nodes, 4);
    assert_eq!(stats.error_nodes, 0);
    // root -> child -> grandchild => depth 3
    assert_eq!(stats.max_depth, 3);
}

// ===========================================================================
// BFS TRAVERSAL
// ===========================================================================

#[test]
fn bfs_enter_order() {
    let (tree, source) = sample_tree();
    let walker = BreadthFirstWalker::new(&source);
    let mut v = OrderVisitor::new();
    walker.walk(&tree, &mut v);
    // Level-order: root(0), a(0), mid(1), d(3), b(1), c(2)
    assert_eq!(v.enter_order, vec![0, 0, 1, 3, 1, 2]);
}

#[test]
fn bfs_leaf_text_collection() {
    let (tree, source) = sample_tree();
    let walker = BreadthFirstWalker::new(&source);
    let mut v = LeafCollector::new();
    walker.walk(&tree, &mut v);
    // BFS visits leaves in level order: a, d come before b, c
    assert_eq!(v.texts, vec!["a", "d", "b", "c"]);
}

#[test]
fn bfs_stats_visitor_counts() {
    let (tree, source) = sample_tree();
    let walker = BreadthFirstWalker::new(&source);
    let mut stats = StatsVisitor::default();
    walker.walk(&tree, &mut stats);
    assert_eq!(stats.total_nodes, 6);
    assert_eq!(stats.leaf_nodes, 4);
}

// ===========================================================================
// SEARCH VISITOR
// ===========================================================================

#[test]
fn search_finds_matching_nodes() {
    let (tree, source) = sample_tree();
    let walker = TreeWalker::new(&source);
    // Search for named nodes whose start_byte == 1 (mid and b both start at 1)
    let mut search = SearchVisitor::new(|n: &ParsedNode| n.start_byte() == 1);
    walker.walk(&tree, &mut search);
    assert_eq!(search.matches.len(), 2);
    assert_eq!(search.matches[0].0, 1); // start_byte
    assert_eq!(search.matches[1].0, 1);
}

#[test]
fn search_finds_named_nodes_only() {
    let (tree, source) = sample_tree();
    let walker = TreeWalker::new(&source);
    let mut search = SearchVisitor::new(|n: &ParsedNode| n.is_named());
    walker.walk(&tree, &mut search);
    // root, a, mid, b, d are named; c is unnamed => 5 matches
    assert_eq!(search.matches.len(), 5);
}

#[test]
fn search_no_matches() {
    let (tree, source) = sample_tree();
    let walker = TreeWalker::new(&source);
    let mut search = SearchVisitor::new(|n: &ParsedNode| n.start_byte() == 999);
    walker.walk(&tree, &mut search);
    assert!(search.matches.is_empty());
}

#[test]
fn search_captures_byte_ranges() {
    let (tree, source) = sample_tree();
    let walker = TreeWalker::new(&source);
    let mut search = SearchVisitor::new(|n: &ParsedNode| n.start_byte() == 3);
    walker.walk(&tree, &mut search);
    assert_eq!(search.matches.len(), 1);
    let (start, end, _kind) = &search.matches[0];
    assert_eq!(*start, 3);
    assert_eq!(*end, 4);
}

// ===========================================================================
// PRETTY PRINT VISITOR
// ===========================================================================

#[test]
fn pretty_print_contains_named_marker() {
    let (tree, source) = sample_tree();
    let walker = TreeWalker::new(&source);
    let mut pp = PrettyPrintVisitor::new();
    walker.walk(&tree, &mut pp);
    let out = pp.output();
    // Named nodes should have [named]
    assert!(out.contains("[named]"), "output:\n{out}");
}

#[test]
fn pretty_print_contains_leaf_text() {
    let (tree, source) = sample_tree();
    let walker = TreeWalker::new(&source);
    let mut pp = PrettyPrintVisitor::new();
    walker.walk(&tree, &mut pp);
    let out = pp.output();
    assert!(out.contains("\"a\""), "output:\n{out}");
    assert!(out.contains("\"b\""), "output:\n{out}");
    assert!(out.contains("\"d\""), "output:\n{out}");
}

#[test]
fn pretty_print_indentation_increases() {
    let (tree, source) = sample_tree();
    let walker = TreeWalker::new(&source);
    let mut pp = PrettyPrintVisitor::new();
    walker.walk(&tree, &mut pp);
    let out = pp.output();
    let lines: Vec<&str> = out.lines().collect();
    // First line (root) has no leading spaces
    assert!(!lines[0].starts_with(' '));
    // Children are indented
    let has_indented = lines.iter().any(|l| l.starts_with("  "));
    assert!(has_indented, "should have indented lines:\n{out}");
}

#[test]
fn pretty_print_default_is_empty() {
    let pp = PrettyPrintVisitor::default();
    assert!(pp.output().is_empty());
}

// ===========================================================================
// TRANSFORM VISITOR
// ===========================================================================

/// Evaluator that counts nodes bottom-up.
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
fn transform_counts_all_nodes() {
    let (tree, source) = sample_tree();
    let walker = TransformWalker::new(&source);
    let mut t = CountTransform;
    let total = walker.walk(&tree, &mut t);
    assert_eq!(total, 6);
}

/// Collects leaf text in post-order.
struct ConcatTransform;

impl TransformVisitor for ConcatTransform {
    type Output = String;

    fn transform_node(&mut self, _: &ParsedNode, children: Vec<String>) -> String {
        children.join("")
    }

    fn transform_leaf(&mut self, _: &ParsedNode, text: &str) -> String {
        text.to_string()
    }

    fn transform_error(&mut self, _: &ParsedNode) -> String {
        "ERR".to_string()
    }
}

#[test]
fn transform_concatenates_leaves() {
    let (tree, source) = sample_tree();
    let walker = TransformWalker::new(&source);
    let mut t = ConcatTransform;
    let result = walker.walk(&tree, &mut t);
    assert_eq!(result, "abcd");
}

#[test]
fn transform_handles_error_node() {
    let source = b"xy".to_vec();
    let err = error_node(0, 1);
    let good = leaf(2, 1, 2);
    let root = interior(10, vec![err, good]);
    let walker = TransformWalker::new(&source);
    let mut t = ConcatTransform;
    let result = walker.walk(&root, &mut t);
    assert_eq!(result, "ERRy");
}

// ===========================================================================
// EMPTY TREE TRAVERSAL
// ===========================================================================

#[test]
fn dfs_empty_tree() {
    // Interior node with no children and zero-width span
    let source = b"".to_vec();
    let root = interior(1, vec![]);
    let walker = TreeWalker::new(&source);
    let mut stats = StatsVisitor::default();
    walker.walk(&root, &mut stats);
    // The root itself is a leaf (0 children) but source is empty so visit_leaf
    // gets an empty string.
    assert_eq!(stats.total_nodes, 1);
    assert_eq!(stats.leaf_nodes, 1);
}

#[test]
fn bfs_empty_tree() {
    let source = b"".to_vec();
    let root = interior(1, vec![]);
    let walker = BreadthFirstWalker::new(&source);
    let mut stats = StatsVisitor::default();
    walker.walk(&root, &mut stats);
    assert_eq!(stats.total_nodes, 1);
    assert_eq!(stats.leaf_nodes, 1);
}

// ===========================================================================
// SINGLE-NODE TREE TRAVERSAL
// ===========================================================================

#[test]
fn dfs_single_node() {
    let (tree, source) = single_node_tree();
    let walker = TreeWalker::new(&source);
    let mut v = OrderVisitor::new();
    walker.walk(&tree, &mut v);
    assert_eq!(v.enter_order, vec![0]);
    assert_eq!(v.leave_order, vec![0]);
}

#[test]
fn bfs_single_node() {
    let (tree, source) = single_node_tree();
    let walker = BreadthFirstWalker::new(&source);
    let mut lc = LeafCollector::new();
    walker.walk(&tree, &mut lc);
    assert_eq!(lc.texts, vec!["x"]);
}

#[test]
fn transform_single_node() {
    let (tree, source) = single_node_tree();
    let walker = TransformWalker::new(&source);
    let mut t = ConcatTransform;
    let result = walker.walk(&tree, &mut t);
    assert_eq!(result, "x");
}

// ===========================================================================
// DEEP TREE TRAVERSAL
// ===========================================================================

#[test]
fn dfs_deep_tree_depth() {
    let (tree, source) = deep_tree(20);
    let walker = TreeWalker::new(&source);
    let mut stats = StatsVisitor::default();
    walker.walk(&tree, &mut stats);
    assert_eq!(stats.total_nodes, 21); // 20 interior + 1 leaf
    assert_eq!(stats.max_depth, 21);
    assert_eq!(stats.leaf_nodes, 1);
}

#[test]
fn transform_deep_tree() {
    let (tree, source) = deep_tree(50);
    let walker = TransformWalker::new(&source);
    let mut t = CountTransform;
    let total = walker.walk(&tree, &mut t);
    assert_eq!(total, 51);
}

// ===========================================================================
// WIDE TREE TRAVERSAL
// ===========================================================================

#[test]
fn dfs_wide_tree() {
    let (tree, source) = wide_tree(100);
    let walker = TreeWalker::new(&source);
    let mut stats = StatsVisitor::default();
    walker.walk(&tree, &mut stats);
    assert_eq!(stats.total_nodes, 101); // root + 100 leaves
    assert_eq!(stats.leaf_nodes, 100);
    assert_eq!(stats.max_depth, 2); // root -> leaf
}

#[test]
fn bfs_wide_tree_leaf_order() {
    let (tree, source) = wide_tree(5);
    let walker = BreadthFirstWalker::new(&source);
    let mut lc = LeafCollector::new();
    walker.walk(&tree, &mut lc);
    // All children are identical "a" leaves
    assert_eq!(lc.texts.len(), 5);
    assert!(lc.texts.iter().all(|t| t == "a"));
}

// ===========================================================================
// VISITOR STATE ACCUMULATION
// ===========================================================================

/// Accumulates a running sum of leaf start bytes.
struct SumVisitor {
    sum: usize,
}

impl SumVisitor {
    fn new() -> Self {
        Self { sum: 0 }
    }
}

impl Visitor for SumVisitor {
    fn enter_node(&mut self, _: &ParsedNode) -> VisitorAction {
        VisitorAction::Continue
    }
    fn visit_leaf(&mut self, node: &ParsedNode, _text: &str) {
        self.sum += node.start_byte();
    }
}

#[test]
fn state_accumulation_sum_of_leaf_positions() {
    let (tree, source) = sample_tree();
    let walker = TreeWalker::new(&source);
    let mut v = SumVisitor::new();
    walker.walk(&tree, &mut v);
    // Leaves at start_bytes: 0, 1, 2, 3 => sum = 6
    assert_eq!(v.sum, 6);
}

#[test]
fn state_accumulation_depth_histogram() {
    let (tree, source) = sample_tree();
    let walker = TreeWalker::new(&source);

    struct DepthHistogramVisitor {
        depth: usize,
        histogram: std::collections::HashMap<usize, usize>,
    }

    impl Visitor for DepthHistogramVisitor {
        fn enter_node(&mut self, _: &ParsedNode) -> VisitorAction {
            self.depth += 1;
            *self.histogram.entry(self.depth).or_insert(0) += 1;
            VisitorAction::Continue
        }
        fn leave_node(&mut self, _: &ParsedNode) {
            self.depth -= 1;
        }
    }

    let mut v = DepthHistogramVisitor {
        depth: 0,
        histogram: std::collections::HashMap::new(),
    };
    walker.walk(&tree, &mut v);
    // depth 1: root(1), depth 2: a, mid, d (3), depth 3: b, c (2)
    assert_eq!(v.histogram[&1], 1);
    assert_eq!(v.histogram[&2], 3);
    assert_eq!(v.histogram[&3], 2);
}

// ===========================================================================
// EARLY TERMINATION
// ===========================================================================

#[test]
fn dfs_early_stop_limits_visits() {
    let (tree, source) = sample_tree();
    let walker = TreeWalker::new(&source);
    let mut v = LimitVisitor::new(2);
    walker.walk(&tree, &mut v);
    // DFS Stop prevents descent into subtrees but siblings still proceed.
    // root(Continue) -> a(Stop) -> mid(Stop) -> d(Stop) = 4 visits
    // Without stop we'd visit all 6 nodes.
    assert_eq!(v.count, 4);
}

#[test]
fn bfs_early_stop_limits_visits() {
    let (tree, source) = sample_tree();
    let walker = BreadthFirstWalker::new(&source);
    let mut v = LimitVisitor::new(3);
    walker.walk(&tree, &mut v);
    assert_eq!(v.count, 3);
}

#[test]
fn dfs_skip_children_prevents_descent() {
    let (tree, source) = sample_tree();
    let walker = TreeWalker::new(&source);
    // Skip children of mid (start_byte == 1)
    let mut v = SkipVisitor::new(vec![1]);
    walker.walk(&tree, &mut v);
    // DFS enters: root(0), a(0), mid(1) [skip], d(3)
    // a and mid share start_byte 0/1; a is at 0, mid at 1
    // b(1) and c(2) should NOT appear
    assert!(!v.visited.contains(&2), "c should be skipped");
    // But d(3) should appear
    assert!(v.visited.contains(&3), "d should still be visited");
}

#[test]
fn bfs_skip_children_prevents_queueing() {
    let (tree, source) = sample_tree();
    let walker = BreadthFirstWalker::new(&source);
    // Skip children of mid (start_byte == 1)
    let mut v = SkipVisitor::new(vec![1]);
    walker.walk(&tree, &mut v);
    // BFS: root(0), a(0), mid(1)[skip], d(3) — b(1), c(2) never queued
    assert!(!v.visited.contains(&2), "c should be skipped");
    assert!(v.visited.contains(&3), "d should still be visited");
}

// ===========================================================================
// ERROR NODE HANDLING
// ===========================================================================

#[test]
fn dfs_error_node_is_reported() {
    let source = b"ab".to_vec();
    let a = leaf(1, 0, 1);
    let err = error_node(1, 2);
    let root = interior(10, vec![a, err]);
    let walker = TreeWalker::new(&source);
    let mut stats = StatsVisitor::default();
    walker.walk(&root, &mut stats);
    assert_eq!(stats.error_nodes, 1);
    // Error nodes are not entered (visit_error is called instead)
    // root + a = 2 entered nodes
    assert_eq!(stats.total_nodes, 2);
}

#[test]
fn bfs_error_node_is_reported() {
    let source = b"ab".to_vec();
    let a = leaf(1, 0, 1);
    let err = error_node(1, 2);
    let root = interior(10, vec![a, err]);
    let walker = BreadthFirstWalker::new(&source);
    let mut stats = StatsVisitor::default();
    walker.walk(&root, &mut stats);
    assert_eq!(stats.error_nodes, 1);
}

// ===========================================================================
// VISITOR ACTION ENUM
// ===========================================================================

#[test]
fn visitor_action_equality() {
    assert_eq!(VisitorAction::Continue, VisitorAction::Continue);
    assert_eq!(VisitorAction::SkipChildren, VisitorAction::SkipChildren);
    assert_eq!(VisitorAction::Stop, VisitorAction::Stop);
    assert_ne!(VisitorAction::Continue, VisitorAction::Stop);
    assert_ne!(VisitorAction::Continue, VisitorAction::SkipChildren);
    assert_ne!(VisitorAction::SkipChildren, VisitorAction::Stop);
}

#[test]
fn visitor_action_debug() {
    let dbg = format!("{:?}", VisitorAction::Continue);
    assert_eq!(dbg, "Continue");
}

#[test]
fn visitor_action_clone() {
    let a = VisitorAction::SkipChildren;
    let b = a;
    assert_eq!(a, b);
}
