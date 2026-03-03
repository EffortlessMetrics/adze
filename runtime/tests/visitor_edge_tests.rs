//! Tests for runtime visitor combinators and edge cases.

use adze::visitor::*;

#[test]
fn stats_visitor_empty_tree() {
    let stats = StatsVisitor::default();
    assert_eq!(stats.total_nodes, 0);
    assert!(stats.node_counts.is_empty());
    assert_eq!(stats.max_depth, 0);
}

#[test]
fn stats_visitor_debug() {
    let stats = StatsVisitor::default();
    let debug = format!("{stats:?}");
    assert!(debug.contains("StatsVisitor"));
}

#[test]
fn tree_walker_creation() {
    let source = b"hello world";
    let _walker = TreeWalker::new(source);
}
