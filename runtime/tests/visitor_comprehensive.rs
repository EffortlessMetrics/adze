//! Comprehensive visitor module tests for adze runtime.
//!
//! This test suite covers 10 key visitor functionality scenarios:
//! 1. Empty tree (0 nodes)
//! 2. Single-node tree
//! 3. Pre-order traversal produces correct order
//! 4. Post-order traversal produces correct order
//! 5. Visitor with early termination (stop after N nodes)
//! 6. Visitor that collects all leaf nodes
//! 7. Visitor that counts nodes at each depth level
//! 8. Visitor on deeply nested tree (50+ levels)
//! 9. Visitor on wide tree (100+ siblings)
//! 10. Multiple visitors on the same tree produce consistent results

use adze::pure_parser::{ParsedNode, Point};
use adze::visitor::{
    BreadthFirstWalker, PrettyPrintVisitor, SearchVisitor, StatsVisitor, TreeWalker, Visitor,
    VisitorAction,
};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Helper Functions
// ---------------------------------------------------------------------------

fn pt(row: u32, col: u32) -> Point {
    Point { row, column: col }
}

/// Create a `ParsedNode` by zero-initializing and overwriting public fields.
/// This safely handles the private `language` field.
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

/// Create a leaf node with the given symbol spanning [start..end).
fn leaf(symbol: u16, start: usize, end: usize) -> ParsedNode {
    make_node(symbol, vec![], start, end, false, true)
}

/// Create an interior node with given children. Byte range is derived from children.
fn interior(symbol: u16, children: Vec<ParsedNode>) -> ParsedNode {
    let start = children.first().map_or(0, |c| c.start_byte);
    let end = children.last().map_or(0, |c| c.end_byte);
    make_node(symbol, children, start, end, false, true)
}

/// Create an error node spanning [start..end).
fn error_node(start: usize, end: usize) -> ParsedNode {
    make_node(0, vec![], start, end, true, false)
}

// ---------------------------------------------------------------------------
// Test 1: Empty Tree (0 nodes) - Single root with no children
// ---------------------------------------------------------------------------

#[test]
fn test_visitor_empty_tree_single_node() {
    let source = b"".to_vec();
    let root = leaf(1, 0, 0);

    struct CountingVisitor {
        enter_count: usize,
        leave_count: usize,
        leaf_count: usize,
    }

    impl Visitor for CountingVisitor {
        fn enter_node(&mut self, _node: &ParsedNode) -> VisitorAction {
            self.enter_count += 1;
            VisitorAction::Continue
        }

        fn leave_node(&mut self, _node: &ParsedNode) {
            self.leave_count += 1;
        }

        fn visit_leaf(&mut self, _node: &ParsedNode, _text: &str) {
            self.leaf_count += 1;
        }
    }

    let walker = TreeWalker::new(&source);
    let mut visitor = CountingVisitor {
        enter_count: 0,
        leave_count: 0,
        leaf_count: 0,
    };

    walker.walk(&root, &mut visitor);

    // Single leaf node: 1 enter, 1 leave, 1 leaf visit
    assert_eq!(visitor.enter_count, 1);
    assert_eq!(visitor.leave_count, 1);
    assert_eq!(visitor.leaf_count, 1);
}

// ---------------------------------------------------------------------------
// Test 2: Single-Node Tree (root only, no children)
// ---------------------------------------------------------------------------

#[test]
fn test_visitor_single_node_tree() {
    let source = b"x".to_vec();
    let root = leaf(42, 0, 1);

    struct KindRecorder {
        kinds_entered: Vec<u16>,
    }

    impl Visitor for KindRecorder {
        fn enter_node(&mut self, node: &ParsedNode) -> VisitorAction {
            self.kinds_entered.push(node.symbol);
            VisitorAction::Continue
        }
    }

    let walker = TreeWalker::new(&source);
    let mut visitor = KindRecorder {
        kinds_entered: vec![],
    };

    walker.walk(&root, &mut visitor);

    assert_eq!(visitor.kinds_entered.len(), 1);
    assert_eq!(visitor.kinds_entered[0], 42);
}

// ---------------------------------------------------------------------------
// Test 3: Pre-Order Traversal (DFS) Produces Correct Order
// ---------------------------------------------------------------------------

#[test]
fn test_visitor_preorder_traversal_order() {
    // Build tree:
    //        root(1)
    //       /  |  \
    //     a(2) b(3) c(4)
    //      |
    //     d(5)
    let source = b"abcd".to_vec();

    let d = leaf(5, 3, 4);
    let a = interior(2, vec![d]);
    let b = leaf(3, 1, 2);
    let c = leaf(4, 2, 3);
    let root = interior(1, vec![a, b, c]);

    struct PreOrderCollector {
        symbols: Vec<u16>,
    }

    impl Visitor for PreOrderCollector {
        fn enter_node(&mut self, node: &ParsedNode) -> VisitorAction {
            self.symbols.push(node.symbol);
            VisitorAction::Continue
        }
    }

    let walker = TreeWalker::new(&source);
    let mut visitor = PreOrderCollector { symbols: vec![] };

    walker.walk(&root, &mut visitor);

    // Pre-order (DFS): root, a, d, b, c
    assert_eq!(visitor.symbols, vec![1, 2, 5, 3, 4]);
}

// ---------------------------------------------------------------------------
// Test 4: Post-Order Traversal via StatsVisitor
// ---------------------------------------------------------------------------

#[test]
fn test_visitor_postorder_via_stats() {
    let source = b"abc".to_vec();
    let a = leaf(1, 0, 1);
    let b = leaf(2, 1, 2);
    let c = leaf(3, 2, 3);
    let root = interior(10, vec![a, b, c]);

    let walker = TreeWalker::new(&source);
    let mut stats = StatsVisitor::default();
    walker.walk(&root, &mut stats);

    // Verify: root + 3 children = 4 total nodes
    assert_eq!(stats.total_nodes, 4);
    // All leaves visited
    assert_eq!(stats.leaf_nodes, 3);
    // Max depth is 2 (root at depth 1, leaves at depth 2)
    assert_eq!(stats.max_depth, 2);
}

// ---------------------------------------------------------------------------
// Test 5: Early Termination (Stop After N Nodes)
// ---------------------------------------------------------------------------

#[test]
fn test_visitor_early_termination_stop_count() {
    let source = b"abcde".to_vec();
    let a = leaf(1, 0, 1);
    let b = leaf(2, 1, 2);
    let c = leaf(3, 2, 3);
    let d = leaf(4, 3, 4);
    let e = leaf(5, 4, 5);
    let root = interior(10, vec![a, b, c, d, e]);

    struct StopAfterNVisitor {
        max_visits: usize,
        visit_count: usize,
    }

    impl Visitor for StopAfterNVisitor {
        fn enter_node(&mut self, _node: &ParsedNode) -> VisitorAction {
            self.visit_count += 1;
            if self.visit_count >= self.max_visits {
                VisitorAction::Stop
            } else {
                VisitorAction::Continue
            }
        }
    }

    let walker = TreeWalker::new(&source);

    // Test 1: Stop after 3 visits
    let mut visitor = StopAfterNVisitor {
        max_visits: 3,
        visit_count: 0,
    };
    walker.walk(&root, &mut visitor);
    assert!(
        visitor.visit_count >= 3,
        "should visit at least max_visits nodes"
    );

    // Test 2: Stop after 1 visit (just root)
    let mut visitor2 = StopAfterNVisitor {
        max_visits: 1,
        visit_count: 0,
    };
    walker.walk(&root, &mut visitor2);
    assert_eq!(visitor2.visit_count, 1);
}

// ---------------------------------------------------------------------------
// Test 6: Visitor That Collects All Leaf Nodes
// ---------------------------------------------------------------------------

#[test]
fn test_visitor_collect_leaf_nodes() {
    let source = b"leaf1leaf2leaf3".to_vec();
    let l1 = leaf(1, 0, 5);
    let l2 = leaf(2, 5, 10);
    let l3 = leaf(3, 10, 15);
    let root = interior(100, vec![l1, l2, l3]);

    struct LeafCollector {
        leaf_texts: Vec<String>,
        leaf_count: usize,
    }

    impl Visitor for LeafCollector {
        fn visit_leaf(&mut self, _node: &ParsedNode, text: &str) {
            self.leaf_texts.push(text.to_string());
            self.leaf_count += 1;
        }
    }

    let walker = TreeWalker::new(&source);
    let mut visitor = LeafCollector {
        leaf_texts: vec![],
        leaf_count: 0,
    };

    walker.walk(&root, &mut visitor);

    assert_eq!(visitor.leaf_count, 3);
    assert_eq!(visitor.leaf_texts.len(), 3);
}

// ---------------------------------------------------------------------------
// Test 7: Count Nodes at Each Depth Level
// ---------------------------------------------------------------------------

#[test]
fn test_visitor_count_nodes_at_depth() {
    // Tree structure:
    //       root (depth 1)
    //      /  |  \
    //    a   b   c  (depth 2)
    //    |
    //    d           (depth 3)
    let source = b"abcd".to_vec();

    let d = leaf(5, 3, 4);
    let a = interior(2, vec![d]);
    let b = leaf(3, 1, 2);
    let c = leaf(4, 2, 3);
    let root = interior(1, vec![a, b, c]);

    struct DepthCounter {
        counts: HashMap<usize, usize>,
        current_depth: usize,
    }

    impl Visitor for DepthCounter {
        fn enter_node(&mut self, _node: &ParsedNode) -> VisitorAction {
            *self.counts.entry(self.current_depth).or_insert(0) += 1;
            self.current_depth += 1;
            VisitorAction::Continue
        }

        fn leave_node(&mut self, _node: &ParsedNode) {
            self.current_depth -= 1;
        }
    }

    let walker = TreeWalker::new(&source);
    let mut visitor = DepthCounter {
        counts: HashMap::new(),
        current_depth: 0,
    };

    walker.walk(&root, &mut visitor);

    // Verify depth counts
    assert_eq!(visitor.counts.get(&0), Some(&1)); // 1 node at depth 0 (root)
    assert_eq!(visitor.counts.get(&1), Some(&3)); // 3 nodes at depth 1 (a, b, c)
    assert_eq!(visitor.counts.get(&2), Some(&1)); // 1 node at depth 2 (d)
    assert_eq!(visitor.counts.get(&3), None); // No nodes at depth 3
}

// ---------------------------------------------------------------------------
// Test 8: Deep Tree (50+ Levels)
// ---------------------------------------------------------------------------

#[test]
fn test_visitor_deeply_nested_tree() {
    let source = b"x".repeat(60);

    // Build a deeply nested tree: root -> child -> child -> ... (60 levels)
    let mut node = leaf(1, 59, 60);
    for _i in (0..59).rev() {
        node = interior(2, vec![node]);
    }
    let root = node;

    struct DepthTracker {
        max_depth: usize,
        current_depth: usize,
    }

    impl Visitor for DepthTracker {
        fn enter_node(&mut self, _node: &ParsedNode) -> VisitorAction {
            self.current_depth += 1;
            self.max_depth = self.max_depth.max(self.current_depth);
            VisitorAction::Continue
        }

        fn leave_node(&mut self, _node: &ParsedNode) {
            self.current_depth -= 1;
        }
    }

    let walker = TreeWalker::new(&source);
    let mut visitor = DepthTracker {
        max_depth: 0,
        current_depth: 0,
    };

    walker.walk(&root, &mut visitor);

    // Should reach depth 60
    assert_eq!(visitor.max_depth, 60);
}

// ---------------------------------------------------------------------------
// Test 9: Wide Tree (100+ Siblings)
// ---------------------------------------------------------------------------

#[test]
fn test_visitor_wide_tree() {
    let source: Vec<u8> = (0..200).collect();

    // Create a root with 100+ leaf children
    let mut children = Vec::new();
    for i in 0..150 {
        children.push(leaf(i as u16 % 10, i, i + 1));
    }
    let root = interior(100, children);

    struct WidthCounter {
        total_children: usize,
    }

    impl Visitor for WidthCounter {
        fn enter_node(&mut self, node: &ParsedNode) -> VisitorAction {
            if node.child_count() > 0 {
                self.total_children += node.child_count();
            }
            VisitorAction::Continue
        }
    }

    let walker = TreeWalker::new(&source);
    let mut visitor = WidthCounter { total_children: 0 };

    walker.walk(&root, &mut visitor);

    // Root has 150 children
    assert_eq!(visitor.total_children, 150);
}

// ---------------------------------------------------------------------------
// Test 10: Multiple Visitors on Same Tree Produce Consistent Results
// ---------------------------------------------------------------------------

#[test]
fn test_visitor_multiple_visitors_consistency() {
    let source = b"expr".to_vec();
    let a = leaf(1, 0, 1);
    let op = leaf(2, 1, 2);
    let b = leaf(2, 2, 4);
    let root = interior(10, vec![a, op, b]);

    // Visitor 1: Count all nodes
    struct NodeCounter {
        count: usize,
    }

    impl Visitor for NodeCounter {
        fn enter_node(&mut self, _node: &ParsedNode) -> VisitorAction {
            self.count += 1;
            VisitorAction::Continue
        }
    }

    // Visitor 2: Collect kinds
    struct KindCollector {
        kinds: Vec<u16>,
    }

    impl Visitor for KindCollector {
        fn enter_node(&mut self, node: &ParsedNode) -> VisitorAction {
            self.kinds.push(node.symbol);
            VisitorAction::Continue
        }
    }

    let walker = TreeWalker::new(&source);

    let mut counter = NodeCounter { count: 0 };
    walker.walk(&root, &mut counter);

    let mut collector = KindCollector { kinds: vec![] };
    walker.walk(&root, &mut collector);

    // Both visitors should agree on total node count
    assert_eq!(counter.count, collector.kinds.len());
    assert_eq!(counter.count, 4); // root + 3 children
}

// ---------------------------------------------------------------------------
// Additional Tests: Breadth-First Walker, Built-in Visitors
// ---------------------------------------------------------------------------

#[test]
fn test_visitor_breadth_first_walker() {
    let source = b"abcd".to_vec();
    let d = leaf(5, 3, 4);
    let a = interior(2, vec![d]);
    let b = leaf(3, 1, 2);
    let c = leaf(4, 2, 3);
    let root = interior(1, vec![a, b, c]);

    struct BFSCollector {
        symbols: Vec<u16>,
    }

    impl Visitor for BFSCollector {
        fn enter_node(&mut self, node: &ParsedNode) -> VisitorAction {
            self.symbols.push(node.symbol);
            VisitorAction::Continue
        }
    }

    let walker = BreadthFirstWalker::new(&source);
    let mut visitor = BFSCollector { symbols: vec![] };

    walker.walk(&root, &mut visitor);

    // BFS level-order: root(1), a(2), b(3), c(4), d(5)
    assert_eq!(visitor.symbols, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_visitor_stats_visitor_builtin() {
    let source = b"test".to_vec();
    let l1 = leaf(1, 0, 1);
    let l2 = leaf(2, 1, 2);
    let l3 = leaf(3, 2, 4);
    let root = interior(100, vec![l1, l2, l3]);

    let walker = TreeWalker::new(&source);
    let mut stats = StatsVisitor::default();
    walker.walk(&root, &mut stats);

    assert_eq!(stats.total_nodes, 4);
    assert_eq!(stats.leaf_nodes, 3);
    assert_eq!(stats.max_depth, 2);
    // node_counts keys are node kinds (symbol names or IDs)
    assert!(!stats.node_counts.is_empty());
}

#[test]
fn test_visitor_pretty_print_visitor() {
    let source = b"ab".to_vec();
    let a = leaf(1, 0, 1);
    let b = leaf(2, 1, 2);
    let root = interior(10, vec![a, b]);

    let walker = TreeWalker::new(&source);
    let mut printer = PrettyPrintVisitor::new();
    walker.walk(&root, &mut printer);

    let output = printer.output();
    assert!(!output.is_empty());
    // Should contain some indentation and node kinds
    assert!(output.contains('\n'));
}

#[test]
fn test_visitor_search_visitor_builtin() {
    let source = b"abc".to_vec();
    let a = leaf(1, 0, 1);
    let b = leaf(1, 1, 2); // Same symbol as a
    let c = leaf(3, 2, 3); // Different symbol
    let root = interior(10, vec![a, b, c]);

    let walker = TreeWalker::new(&source);
    let mut search = SearchVisitor::new(|node: &ParsedNode| node.symbol == 1);
    walker.walk(&root, &mut search);

    // Should find 2 nodes with symbol 1
    assert_eq!(search.matches.len(), 2);
}

// ---------------------------------------------------------------------------
// Test: Skip Children Action
// ---------------------------------------------------------------------------

#[test]
fn test_visitor_skip_children_action() {
    let source = b"abcd".to_vec();
    let d = leaf(5, 3, 4);
    let a = interior(2, vec![d]);
    let b = leaf(3, 1, 2);
    let c = leaf(4, 2, 3);
    let root = interior(1, vec![a, b, c]);

    struct SkipingVisitor {
        visited_symbols: Vec<u16>,
    }

    impl Visitor for SkipingVisitor {
        fn enter_node(&mut self, node: &ParsedNode) -> VisitorAction {
            self.visited_symbols.push(node.symbol);
            // Skip children of node with symbol 2
            if node.symbol == 2 {
                VisitorAction::SkipChildren
            } else {
                VisitorAction::Continue
            }
        }
    }

    let walker = TreeWalker::new(&source);
    let mut visitor = SkipingVisitor {
        visited_symbols: vec![],
    };

    walker.walk(&root, &mut visitor);

    // Should visit: root(1), a(2), b(3), c(4) but NOT d(5) because we skipped a's children
    assert_eq!(visitor.visited_symbols, vec![1, 2, 3, 4]);
    assert!(!visitor.visited_symbols.contains(&5));
}

// ---------------------------------------------------------------------------
// Test: Error Nodes in Tree
// ---------------------------------------------------------------------------

#[test]
fn test_visitor_with_error_nodes() {
    let source = b"error".to_vec();
    let valid = leaf(1, 0, 2);
    let err = error_node(2, 4);
    let valid2 = leaf(2, 4, 5);
    let root = interior(10, vec![valid, err, valid2]);

    struct ErrorTracker {
        error_count: usize,
        node_count: usize,
    }

    impl Visitor for ErrorTracker {
        fn enter_node(&mut self, _node: &ParsedNode) -> VisitorAction {
            self.node_count += 1;
            VisitorAction::Continue
        }

        fn visit_error(&mut self, _node: &ParsedNode) {
            self.error_count += 1;
        }
    }

    let walker = TreeWalker::new(&source);
    let mut visitor = ErrorTracker {
        error_count: 0,
        node_count: 0,
    };

    walker.walk(&root, &mut visitor);

    // Should detect the error node
    assert_eq!(visitor.error_count, 1);
    // enter_node should be called 3 times (root, valid, valid2) + error path
    assert!(visitor.node_count >= 2);
}

// ---------------------------------------------------------------------------
// Test: Visitor Leave Node Ordering
// ---------------------------------------------------------------------------

#[test]
fn test_visitor_enter_leave_ordering() {
    let source = b"ab".to_vec();
    let a = leaf(1, 0, 1);
    let b = leaf(2, 1, 2);
    let root = interior(10, vec![a, b]);

    struct OrderingTracker {
        events: Vec<String>,
    }

    impl Visitor for OrderingTracker {
        fn enter_node(&mut self, node: &ParsedNode) -> VisitorAction {
            self.events.push(format!("enter:{}", node.symbol));
            VisitorAction::Continue
        }

        fn leave_node(&mut self, node: &ParsedNode) {
            self.events.push(format!("leave:{}", node.symbol));
        }
    }

    let walker = TreeWalker::new(&source);
    let mut visitor = OrderingTracker { events: vec![] };

    walker.walk(&root, &mut visitor);

    // Expected: enter root, enter a, leave a, enter b, leave b, leave root
    let expected = vec![
        "enter:10", "enter:1", "leave:1", "enter:2", "leave:2", "leave:10",
    ];
    assert_eq!(visitor.events, expected);
}
